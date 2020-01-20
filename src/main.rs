extern crate jack;
extern crate crossbeam_queue;
extern crate prost;
extern crate tokio;
extern crate tower_grpc;
extern crate tower_hyper;
extern crate futures;
extern crate bytes;
extern crate serde;
extern crate serde_yaml;

use std::{io, thread, fs};
use std::fs::File;
use crate::protos::{Config, MidiMapping, LooperCommand, LooperCommandType, TargetNumber, Command, GlobalCommandType};
use crate::protos::command::CommandOneof;
use crate::protos::looper_command::TargetOneof;
use crate::config::LooperCommandTarget;

mod sample;
mod engine;
mod gui;
mod protos;
mod config;

fn main() {
    let (gui, output, input) = gui::Gui::new();
    thread::spawn(move|| {
        gui.run();
        println!("window exited... shutting down");
        std::process::exit(0);
    });

    // read config
    let mut config_path = std::env::home_dir().unwrap();
    config_path.push(".config/loopers/config.yaml");
    let mut config : config::Config = fs::read_to_string(config_path)
        .map(|s| serde_yaml::from_str(&s).expect("Failed to parse config file"))
        .unwrap_or(config::Config { midi_mappings: vec![] });

    let mut mapping_path = std::env::home_dir().unwrap();
    mapping_path.push(".config/loopers/midi_mappings.tsv");
    let mappings = fs::read_to_string(mapping_path)
        .map(|s| {
            s.lines()
                .filter(|l| !l.trim().is_empty())
                .map(config::MidiMapping::from_line)
                .map(|m| m.expect("Failed to map line"))
                .collect::<Vec<config::MidiMapping>>()
        });

    if let Ok(mappings) = mappings {
        config.midi_mappings.extend(&mappings);
    }

    println!("Config: {:#?}", config);

    // read wav files
    let mut reader = hound::WavReader::open("resources/sine_normal.wav").unwrap();
    let beat_normal: Vec<f32> = reader.into_samples().into_iter()
        .map(|x| x.unwrap()).collect();

    let mut reader = hound::WavReader::open("resources/sine_emphasis.wav").unwrap();
    let beat_empahsis: Vec<f32> = reader.into_samples().into_iter()
        .map(|x| x.unwrap()).collect();

    // Create client
    let (client, _status) =
        jack::Client::new("loopers", jack::ClientOptions::NO_START_SERVER).unwrap();

    // Register ports. They will be used in a callback that will be
    // called when new data is available.
    let in_a = client
        .register_port("rust_in_l", jack::AudioIn::default()).unwrap();
    let in_b = client
        .register_port("rust_in_r", jack::AudioIn::default()).unwrap();
    let mut out_a = client
        .register_port("rust_out_l", jack::AudioOut::default()).unwrap();
    let mut out_b = client
        .register_port("rust_out_r", jack::AudioOut::default()).unwrap();

    let mut met_out_a = client
        .register_port("metronome_out_l",jack::AudioOut::default()).unwrap();
    let mut met_out_b = client
        .register_port("metronome_out_r", jack::AudioOut::default()).unwrap();

    let controller = client
        .register_port("rust_midi_in", jack::MidiIn::default()).unwrap();


    let mut engine = engine::Engine::new(
        config.to_config(),
        in_a, in_b, out_a, out_b, met_out_a, met_out_b, controller,
        output, input, beat_normal, beat_empahsis);

    let process_callback = move |client: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
        engine.process(client, ps)
    };
    let process = jack::ClosureProcessHandler::new(process_callback);

    // Activate the client, which starts the processing.
    let active_client = client.activate_async(Notifications, process).unwrap();

    // Wait for user input to quit
    println!("Press enter/return to quit...");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();

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
