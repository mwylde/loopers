#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]

extern crate bytes;
extern crate chrono;
extern crate crossbeam_queue;
extern crate dirs;
extern crate futures;
extern crate jack;
extern crate prost;
extern crate serde;
extern crate serde_yaml;
extern crate tokio;
extern crate tower_grpc;
extern crate tower_hyper;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

use crate::midi::MidiEvent;
use clap::{App, Arg};
use std::{fs, io, thread};

#[allow(dead_code)]
mod protos;

mod config;
mod engine;
mod error;
mod gui;
mod looper;
mod metronome;
mod midi;
mod music;
mod sample;
mod session;

fn setup_logger() -> Result<(), fern::InitError> {
    let stdout_config = fern::Dispatch::new()
        .chain(io::stdout())
        .level(log::LevelFilter::Error);

    let file_config = fern::Dispatch::new()
        .chain(fern::log_file("output.log")?)
        .level(log::LevelFilter::Info);

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .chain(stdout_config)
        .chain(file_config)
        .apply()?;

    Ok(())
}

fn main() {
    if let Err(e) = setup_logger() {
        eprintln!("Unable to set up logging: {:?}", e);
    }

    let matches = App::new("loopers")
        .version("0.0.1")
        .author("Micah Wylde <micah@micahw.com>")
        .arg(Arg::with_name("restore").long("restore"))
        .get_matches();

    let restore = matches.is_present("restore");

    if restore {
        info!("Restoring previous session");
    }

    let (gui, output, input) = gui::Gui::new();
    thread::spawn(move || {
        gui.run();
        info!("window exited... shutting down");
        std::process::exit(0);
    });

    // read config
    let mut config_path = dirs::config_dir().unwrap();
    config_path.push("loopers/config.yaml");
    let mut config: config::Config = fs::read_to_string(config_path)
        .map(|s| serde_yaml::from_str(&s).expect("Failed to parse config file"))
        .unwrap_or(config::Config {
            midi_mappings: vec![],
        });

    let mut mapping_path = dirs::config_dir().unwrap();
    mapping_path.push("loopers/midi_mappings.tsv");
    let mappings = fs::read_to_string(mapping_path).map(|s| {
        s.lines()
            .filter(|l| !l.trim().is_empty())
            .map(config::MidiMapping::from_line)
            .map(|m| m.expect("Failed to map line"))
            .collect::<Vec<config::MidiMapping>>()
    });

    if let Ok(mappings) = mappings {
        config.midi_mappings.extend(&mappings);
    }

    info!("Config: {:#?}", config);

    // read wav files
    let reader = hound::WavReader::open("resources/sine_normal.wav").unwrap();
    let beat_normal: Vec<f32> = reader
        .into_samples()
        .into_iter()
        .map(|x| x.unwrap())
        .collect();

    let reader = hound::WavReader::open("resources/sine_emphasis.wav").unwrap();
    let beat_empahsis: Vec<f32> = reader
        .into_samples()
        .into_iter()
        .map(|x| x.unwrap())
        .collect();

    // Create client
    let (client, _status) =
        jack::Client::new("loopers", jack::ClientOptions::NO_START_SERVER).unwrap();

    // Register ports. They will be used in a callback that will be
    // called when new data is available.
    let in_a = client
        .register_port("rust_in_l", jack::AudioIn::default())
        .unwrap();
    let in_b = client
        .register_port("rust_in_r", jack::AudioIn::default())
        .unwrap();
    let mut out_a = client
        .register_port("rust_out_l", jack::AudioOut::default())
        .unwrap();
    let mut out_b = client
        .register_port("rust_out_r", jack::AudioOut::default())
        .unwrap();

    let mut met_out_a = client
        .register_port("metronome_out_l", jack::AudioOut::default())
        .unwrap();
    let mut met_out_b = client
        .register_port("metronome_out_r", jack::AudioOut::default())
        .unwrap();

    let midi_in = client
        .register_port("rust_midi_in", jack::MidiIn::default())
        .unwrap();

    let mut engine = engine::Engine::new(
        config.to_config(),
        output,
        input,
        beat_normal,
        beat_empahsis,
        restore,
    );

    let process_callback =
        move |_client: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            let in_bufs = [in_a.as_slice(ps), in_b.as_slice(ps)];
            let mut out_bufs = [out_a.as_mut_slice(ps), out_b.as_mut_slice(ps)];
            for buf in &mut out_bufs {
                for b in &mut **buf {
                    *b = 0f32
                }
            }

            let mut met_bufs = [met_out_a.as_mut_slice(ps), met_out_b.as_mut_slice(ps)];
            for buf in &mut met_bufs {
                for b in &mut **buf {
                    *b = 0f32
                }
            }

            let midi_events: Vec<MidiEvent> = midi_in
                .iter(ps)
                .map(|e| MidiEvent {
                    bytes: e.bytes.to_vec(),
                })
                .collect();

            engine.process(
                in_bufs,
                &mut out_bufs,
                &mut met_bufs,
                ps.n_frames() as u64,
                &midi_events,
            );

            jack::Control::Continue
        };
    let process = jack::ClosureProcessHandler::new(process_callback);

    // Activate the client, which starts the processing.
    let active_client = client.activate_async(Notifications, process).unwrap();

    // Wait for user input to quit
    println!("Type q to quit...");
    loop {
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input).ok();
        if user_input == "q" {
            break;
        }
    }

    active_client.deactivate().unwrap();
}

struct Notifications;

impl jack::NotificationHandler for Notifications {
    fn thread_init(&self, _: &jack::Client) {
        println!("JACK: thread init");
    }

    fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        println!(
            "JACK: shutdown with status {:?} because \"{}\"",
            status, reason
        );
    }

    fn freewheel(&mut self, _: &jack::Client, is_enabled: bool) {
        println!(
            "JACK: freewheel mode is {}",
            if is_enabled { "on" } else { "off" }
        );
    }

    fn buffer_size(&mut self, _: &jack::Client, sz: jack::Frames) -> jack::Control {
        println!("JACK: buffer size changed to {}", sz);
        jack::Control::Continue
    }

    fn sample_rate(&mut self, _: &jack::Client, srate: jack::Frames) -> jack::Control {
        println!("JACK: sample rate changed to {}", srate);
        jack::Control::Continue
    }

    fn client_registration(&mut self, _: &jack::Client, name: &str, is_reg: bool) {
        println!(
            "JACK: {} client with name \"{}\"",
            if is_reg { "registered" } else { "unregistered" },
            name
        );
    }

    fn port_registration(&mut self, _: &jack::Client, port_id: jack::PortId, is_reg: bool) {
        println!(
            "JACK: {} port with id {}",
            if is_reg { "registered" } else { "unregistered" },
            port_id
        );
    }

    fn port_rename(
        &mut self,
        _: &jack::Client,
        port_id: jack::PortId,
        old_name: &str,
        new_name: &str,
    ) -> jack::Control {
        println!(
            "JACK: port with id {} renamed from {} to {}",
            port_id, old_name, new_name
        );
        jack::Control::Continue
    }

    fn ports_connected(
        &mut self,
        _: &jack::Client,
        port_id_a: jack::PortId,
        port_id_b: jack::PortId,
        are_connected: bool,
    ) {
        println!(
            "JACK: ports with id {} and {} are {}",
            port_id_a,
            port_id_b,
            if are_connected {
                "connected"
            } else {
                "disconnected"
            }
        );
    }

    fn graph_reorder(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: graph reordered");
        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: xrun occurred");
        jack::Control::Continue
    }

    fn latency(&mut self, _: &jack::Client, mode: jack::LatencyType) {
        println!(
            "JACK: {} latency has changed",
            match mode {
                jack::LatencyType::Capture => "capture",
                jack::LatencyType::Playback => "playback",
            }
        );
    }
}
