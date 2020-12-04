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
use jack::{AudioOut, Client, Port, ProcessScope};
use loopers_common::gui_channel::GuiSender;
use loopers_common::Host;
use loopers_engine::midi::MidiEvent;
use loopers_engine::Engine;
use loopers_gui::Gui;
use std::collections::HashMap;
use std::io;

// metronome sounds; included in the binary for now to ease usage of cargo install
const SINE_NORMAL: &[u8] = include_bytes!("../resources/sine_normal.wav");
const SINE_EMPHASIS: &[u8] = include_bytes!("../resources/sine_emphasis.wav");

fn setup_logger(debug_log: bool) -> Result<(), fern::InitError> {
    let stdout_config = fern::Dispatch::new()
        .chain(io::stdout())
        .level(log::LevelFilter::Warn);

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

pub struct JackHost<'a> {
    client: &'a Client,
    looper_ports: &'a mut HashMap<u32, [Port<AudioOut>; 2]>,
    ps: Option<&'a ProcessScope>,
}

impl<'a> Host<'a> for JackHost<'a> {
    fn add_looper(&mut self, id: u32) -> Result<(), String> {
        if !self.looper_ports.contains_key(&id) {
            let l = self
                .client
                .register_port(&format!("loop{}_out_l", id), jack::AudioOut::default())
                .map_err(|e| format!("could not create jack port: {:?}", e))?;
            let r = self
                .client
                .register_port(&format!("loop{}_out_r", id), jack::AudioOut::default())
                .map_err(|e| format!("could not create jack port: {:?}", e))?;

            self.looper_ports.insert(id, [l, r]);
        }

        Ok(())
    }

    fn remove_looper(&mut self, id: u32) -> Result<(), String> {
        if let Some([l, r]) = self.looper_ports.remove(&id) {
            self.client
                .unregister_port(l)
                .map_err(|e| format!("could not remove jack port: {:?}", e))?;
            self.client
                .unregister_port(r)
                .map_err(|e| format!("could not remove jack port: {:?}", e))?;
        }

        Ok(())
    }

    fn output_for_looper<'b>(&'b mut self, id: u32) -> Option<[&'b mut [f32]; 2]>
    where
        'a: 'b,
    {
        let ps = self.ps?;
        let [l, r] = self.looper_ports.get_mut(&id)?;
        Some([l.as_mut_slice(ps), r.as_mut_slice(ps)])
    }
}

fn main() {
    let matches = App::new("loopers-jack")
        .version("0.1.0")
        .author("Micah Wylde <micah@micahw.com>")
        .about(
            "Loopers is a graphical live looper, designed for ease of use and rock-solid stability",
        )
        .arg(
            Arg::with_name("restore")
                .long("restore")
                .help("Automatically restores the last saved session"),
        )
        .arg(
            Arg::with_name("no-gui")
                .long("no-gui")
                .help("Launches in headless mode (without the gui)"),
        )
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

    let (new_gui, gui_sender) = if !matches.is_present("no-gui") {
        let (sender, receiver) = GuiSender::new();
        (
            Some(Gui::new(receiver, gui_to_engine_sender, sender.clone())),
            sender,
        )
    } else {
        (None, GuiSender::disconnected())
    };

    // read wav files
    let reader = hound::WavReader::new(SINE_NORMAL).unwrap();
    let beat_normal: Vec<f32> = reader
        .into_samples()
        .into_iter()
        .map(|x| x.unwrap())
        .collect();

    let reader = hound::WavReader::new(SINE_EMPHASIS).unwrap();
    let beat_empahsis: Vec<f32> = reader
        .into_samples()
        .into_iter()
        .map(|x| x.unwrap())
        .collect();

    // Create client
    let (client, _status) = jack::Client::new("loopers", jack::ClientOptions::NO_START_SERVER)
        .expect("Jack server is not running");

    // Register ports. They will be used in a callback that will be
    // called when new data is available.
    let in_a = client
        .register_port("in_l", jack::AudioIn::default())
        .unwrap();
    let in_b = client
        .register_port("in_r", jack::AudioIn::default())
        .unwrap();
    let mut out_a = client
        .register_port("main_out_l", jack::AudioOut::default())
        .unwrap();
    let mut out_b = client
        .register_port("main_out_r", jack::AudioOut::default())
        .unwrap();

    let mut met_out_a = client
        .register_port("metronome_out_l", jack::AudioOut::default())
        .unwrap();
    let mut met_out_b = client
        .register_port("metronome_out_r", jack::AudioOut::default())
        .unwrap();

    let midi_in = client
        .register_port("loopers_midi_in", jack::MidiIn::default())
        .unwrap();

    let mut looper_ports: HashMap<u32, [Port<AudioOut>; 2]> = HashMap::new();

    let mut host = JackHost {
        client: &client,
        looper_ports: &mut looper_ports,
        ps: None,
    };

    let mut engine = Engine::new(
        &mut host,
        gui_sender,
        gui_to_engine_receiver,
        beat_normal,
        beat_empahsis,
        restore,
        client.sample_rate(),
    );

    let process_callback =
        move |_client: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            let in_bufs = [in_a.as_slice(ps), in_b.as_slice(ps)];
            let out_l = out_a.as_mut_slice(ps);
            let out_r = out_b.as_mut_slice(ps);
            for b in &mut *out_l {
                *b = 0f32;
            }
            for b in &mut *out_r {
                *b = 0f32;
            }

            for l in looper_ports.values_mut() {
                for c in l {
                    for v in c.as_mut_slice(ps) {
                        *v = 0f32;
                    }
                }
            }

            let mut met_bufs = [met_out_a.as_mut_slice(ps), met_out_b.as_mut_slice(ps)];
            for buf in &mut met_bufs {
                for b in &mut **buf {
                    *b = 0f32
                }
            }

            let mut host = JackHost {
                client: &_client,
                looper_ports: &mut looper_ports,
                ps: Some(ps),
            };

            let midi_events: Vec<MidiEvent> = midi_in
                .iter(ps)
                .map(|e| MidiEvent {
                    bytes: e.bytes.to_vec(),
                })
                .collect();

            engine.process(
                &mut host,
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
    std::process::exit(0);
}

struct Notifications;

impl jack::NotificationHandler for Notifications {
    fn thread_init(&self, _: &jack::Client) {
        debug!("JACK: thread init");
    }

    fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        debug!(
            "JACK: shutdown with status {:?} because \"{}\"",
            status, reason
        );
    }

    fn freewheel(&mut self, _: &jack::Client, is_enabled: bool) {
        debug!(
            "JACK: freewheel mode is {}",
            if is_enabled { "on" } else { "off" }
        );
    }

    fn buffer_size(&mut self, _: &jack::Client, sz: jack::Frames) -> jack::Control {
        debug!("JACK: buffer size changed to {}", sz);
        jack::Control::Continue
    }

    fn sample_rate(&mut self, _: &jack::Client, _: jack::Frames) -> jack::Control {
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
        debug!(
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
