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

mod loopers_jack;

#[cfg(target_os = "macos")]
mod looper_coreaudio;

use crate::loopers_jack::jack_main;
use clap::{arg, Command};
use crossbeam_channel::bounded;
use loopers_common::gui_channel::GuiSender;
use loopers_gui::Gui;
use std::io;
use std::process::exit;

// metronome sounds; included in the binary for now to ease usage of cargo install
const SINE_NORMAL: &[u8] = include_bytes!("../resources/sine_normal.wav");
const SINE_EMPHASIS: &[u8] = include_bytes!("../resources/sine_emphasis.wav");

fn setup_logger(debug_log: bool) -> Result<(), fern::InitError> {
    let stdout_config = fern::Dispatch::new()
        .chain(io::stdout())
        .level(log::LevelFilter::Info);

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
        let file_config = fern::Dispatch::new()
            .chain(fern::log_file("output.log")?)
            .level(log::LevelFilter::Debug);

        d = d.chain(file_config);
    };

    d.apply()?;

    Ok(())
}

#[cfg(target_os = "macos")]
const DEFAULT_DRIVER: &str = "coreaudio";

#[cfg(not(target_os = "macos"))]
const DEFAULT_DRIVER: &str = "jack";

fn main() {
    let drivers = if cfg!(target_os = "macos") {
        "coreaudio, jack"
    } else {
        "jack"
    };

    let matches = Command::new("loopers")
        .version("0.1.2")
        .author("Micah Wylde <micah@micahw.com>")
        .about(
            "Loopers is a graphical live looper, designed for ease of use and rock-solid stability",
        )
        .arg(arg!(--restore "Automatically restores the last saved session"))
        .arg(arg!(--"no-gui" "Launches in headless mode (without the gui)"))
        .arg(
            arg!(--driver <VALUE>)
                .default_value(DEFAULT_DRIVER)
                .help(format!(
                    "Controls which audio driver to use (included drivers: {})",
                    drivers
                )),
        )
        .arg(arg!(--debug))
        .get_matches();

    if let Err(e) = setup_logger(matches.get_flag("debug")) {
        eprintln!("Unable to set up logging: {:?}", e);
    }

    let restore = matches.get_flag("restore");

    if restore {
        info!("Restoring previous session");
    }

    let (gui_to_engine_sender, gui_to_engine_receiver) = bounded(100);

    let (gui, gui_sender) = if !matches.get_flag("no-gui") {
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
    let beat_normal: Vec<f32> = reader.into_samples().map(|x| x.unwrap()).collect();

    let reader = hound::WavReader::new(SINE_EMPHASIS).unwrap();
    let beat_emphasis: Vec<f32> = reader.into_samples().map(|x| x.unwrap()).collect();

    match matches
        .get_one("driver")
        .unwrap_or(&DEFAULT_DRIVER.to_string())
        .as_str()
    {
        "jack" => {
            jack_main(
                gui,
                gui_sender,
                gui_to_engine_receiver,
                beat_normal,
                beat_emphasis,
                restore,
            );
        }
        "coreaudio" => {
            if cfg!(target_os = "macos") {
                #[cfg(target_os = "macos")]
                crate::looper_coreaudio::coreaudio_main(
                    gui,
                    gui_sender,
                    gui_to_engine_receiver,
                    beat_normal,
                    beat_emphasis,
                    restore,
                )
                .expect("failed to set up coreaudio");
            } else {
                eprintln!("Coreaudio is not supported on this system; choose another driver");
                exit(1);
            }
        }
        driver => {
            eprintln!("Unknown driver '{}'", driver);
            exit(1);
        }
    }
}
