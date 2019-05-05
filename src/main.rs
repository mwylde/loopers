extern crate jack;
extern crate crossbeam_queue;
extern crate azul;
use std::{io, thread};

mod engine;
mod gui;

#[derive(Ord, PartialOrd, PartialEq, Eq, Debug, Copy, Clone)]
pub enum RecordMode {
    NONE, READY, RECORD, OVERDUB
}

#[derive(Ord, PartialOrd, PartialEq, Eq, Debug, Copy, Clone)]
pub enum PlayMode {
    PAUSED, PLAYING
}

// Messages are sent from the audio thread to the gui
#[derive(Ord, PartialOrd, PartialEq, Eq, Debug)]
pub enum Message {
    LoopCreated(u128),
    LoopDestroyed(u128),
    RecordingStateChanged(RecordMode, u128),
    PlayingStateChanged(PlayMode, u128),
    TimeChanged(i64, u128),
    LengthChanged(i64, u128),
    ActiveChanged(u128),
}

// Commands are sent from the Gui to the audio thread
pub enum Command {

}

fn main() {
    let (gui, output, input) = gui::Gui::new();
    thread::spawn(move|| {
        gui.run();
        println!("window exited... shutting down");
        std::process::exit(0);
    });

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

    let controller = client
        .register_port("rust_midi_in", jack::MidiIn::default()).unwrap();


    let mut engine = engine::Engine::new(in_a, in_b, out_a, out_b, controller,
                                     output, input);

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
