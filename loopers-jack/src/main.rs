#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]

extern crate bytes;
extern crate chrono;
extern crate crossbeam_queue;
extern crate dirs;
extern crate futures;
extern crate jack;
extern crate serde;
#[macro_use]
extern crate log;

use clap::{App, Arg};
use crossbeam_channel::bounded;
use loopers_common::config;
use loopers_common::config::MidiMapping;
use loopers_common::gui_channel::GuiSender;
use loopers_engine::midi::MidiEvent;
use loopers_engine::Engine;
use loopers_gui::Gui;
use std::fs::File;
use std::{fs, io};

fn setup_logger(debug_log: bool) -> Result<(), fern::InitError> {
    let stdout_config = fern::Dispatch::new()
        .chain(io::stdout())
        .level(log::LevelFilter::Error);

    let file_config = fern::Dispatch::new()
        .chain(fern::log_file("output.log")?)
        .level(log::LevelFilter::Debug);

    let mut d = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .chain(stdout_config);

    if debug_log {
        d = d.chain(file_config);
    };

    d.apply()?;

    Ok(())
}

fn main() {
    let matches = App::new("loopers")
        .version("0.0.1")
        .author("Micah Wylde <micah@micahw.com>")
        .arg(Arg::with_name("restore").long("restore"))
        .arg(Arg::with_name("gui").long("gui"))
        .arg(Arg::with_name("debug").long("debug"))
        .get_matches();

    if let Err(e) = setup_logger(matches.is_present("debug")) {
        eprintln!("Unable to set up logging: {:?}", e);
    }

    let restore = matches.is_present("restore");

    if restore {
        info!("Restoring previous session");
    }

    let (gui_to_engine_sender, gui_to_engine_receiver) = bounded(100);

    let (new_gui, gui_sender) = if matches.is_present("gui") {
        let (sender, receiver) = GuiSender::new();
        (Some(Gui::new(receiver, gui_to_engine_sender)), sender)
    } else {
        (None, GuiSender::disconnected())
    };

    // read config
    let mut config_path = dirs::config_dir().unwrap();
    config_path.push("loopers/config.toml");
    let mut config: config::Config = fs::read_to_string(config_path)
        .map(|s| toml::from_str(&s).expect("Failed to parse config file"))
        .unwrap_or(config::Config {
            midi_mappings: vec![],
        });

    let mut mapping_path = dirs::config_dir().unwrap();
    mapping_path.push("loopers/midi_mappings.tsv");
    if let Ok(file) = File::open(&mapping_path) {
        match MidiMapping::from_file(&mapping_path.to_string_lossy(), &file) {
            Ok(mms) => config.midi_mappings.extend(mms),
            Err(e) => {
                error!("Failed to load midi mappings: {:?}", e);
            }
        }
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
        .register_port("loopers_in_l", jack::AudioIn::default())
        .unwrap();
    let in_b = client
        .register_port("loopers_in_r", jack::AudioIn::default())
        .unwrap();
    let mut out_a = client
        .register_port("loopers_out_l", jack::AudioOut::default())
        .unwrap();
    let mut out_b = client
        .register_port("loopers_out_r", jack::AudioOut::default())
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

    let mut engine = Engine::new(
        config,
        gui_sender,
        gui_to_engine_receiver,
        beat_normal,
        beat_empahsis,
        restore,
    );

    let process_callback =
        move |_client: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            let in_bufs = [in_a.as_slice(ps), in_b.as_slice(ps)];
            let out_l = out_a.as_mut_slice(ps);
            let out_r = out_b.as_mut_slice(ps);
            for b in &mut *out_l {
                *b = 0f32
            }
            for b in &mut *out_r {
                *b = 0f32
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
                out_l,
                out_r,
                met_bufs,
                ps.n_frames() as u64,
                &midi_events,
            );

            jack::Control::Continue
        };
    let process = jack::ClosureProcessHandler::new(process_callback);

    // Activate the client, which starts the processing.
    let active_client = client.activate_async(Notifications, process).unwrap();

    // start the gui
    if let Some(gui) = new_gui {
        gui.start();
    } else {
        loop {
            let mut user_input = String::new();
            io::stdin().read_line(&mut user_input).ok();
            if user_input == "q" {
                break;
            }
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
        warn!("JACK: sample rate changed to {}. This is not supported yet.", srate);
        jack::Control::Quit
    }

    fn client_registration(&mut self, _: &jack::Client, name: &str, is_reg: bool) {
        info!(
            "JACK: {} client with name \"{}\"",
            if is_reg { "registered" } else { "unregistered" },
            name
        );
    }

    fn port_registration(&mut self, _: &jack::Client, port_id: jack::PortId, is_reg: bool) {
        info!(
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
        info!(
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
        info!("JACK: graph reordered");
        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        warn!("JACK: xrun occurred");
        jack::Control::Continue
    }

    fn latency(&mut self, _: &jack::Client, mode: jack::LatencyType) {
        info!(
            "JACK: {} latency has changed",
            match mode {
                jack::LatencyType::Capture => "capture",
                jack::LatencyType::Playback => "playback",
            }
        );
    }
}
