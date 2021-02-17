#[macro_use]
extern crate vst;

use crossbeam_channel::{bounded, Sender};

use vst::plugin::{Plugin, Info, Category, CanDo};
use vst::api::{Events, Supported};
use vst::buffer::AudioBuffer;
use loopers_engine::Engine;
use loopers_common::Host;
use loopers_common::gui_channel::{GuiSender, GuiReceiver};
use loopers_common::api::{set_sample_rate, Command};
use loopers_common::midi::MidiEvent;
use std::collections::VecDeque;
use vst::event::Event;
use std::ffi::c_void;
use vst::editor::Editor;
use loopers_gui::Gui;

struct LoopersVstHost {}

struct LoopersVst {
    engine: Engine,
    host: LoopersVstHost,
    events: VecDeque<MidiEvent>,
    gui_receiver: GuiReceiver,
    gui_to_engine_sender: Sender<Command>,
    gui_sender: GuiSender,

}

impl Default for LoopersVst {
    fn default() -> Self {
        let mut host = LoopersVstHost {};

        let (gui_to_engine_sender, gui_to_engine_receiver) = bounded(100);

        let (gui_sender, gui_receiver) = GuiSender::new();

        let mut engine = Engine::new(
            &mut host,
            gui_sender.clone() ,
            gui_to_engine_receiver,
            vec![],
            vec![],
            false,
            44100,
        );

        LoopersVst {
            engine,
            host,
            events: VecDeque::with_capacity(100),
            gui_receiver,
            gui_to_engine_sender,
            gui_sender,
        }
    }
}

impl <'a> Host<'a> for LoopersVstHost {
    fn add_looper(&mut self, _: u32) -> Result<(), String> {
        Ok(())
    }

    fn remove_looper(&mut self, _: u32) -> Result<(), String> {
        Ok(())
    }

    fn output_for_looper<'b>(&'b mut self, _: u32) -> Option<[&'b mut [f32]; 2]> where
        'a: 'b {
        None
    }
}

impl Plugin for LoopersVst {
    fn get_info(&self) -> Info {
        Info {
            name: "loopers".to_string(),
            vendor: "micahw".to_string(),
            parameters: 0,
            inputs: 2,
            outputs: 2,
            midi_inputs: 1,
            midi_outputs: 0,
            unique_id: 789149909,
            version: 0,
            category: Category::Unknown,
            ..Default::default()
        }
    }

    fn set_sample_rate(&mut self, rate: f32) {
        println!("Setting sample rate {}", rate);
        set_sample_rate(rate as usize);
    }

    fn can_do(&self, can_do: CanDo) -> Supported {
        match can_do {
            CanDo::ReceiveMidiEvent => Supported::Yes,
            _ => Supported::Maybe,
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let samples = buffer.samples();

        let (inputs, mut outputs) = buffer.split();


        let events: Vec<MidiEvent> = self.events.drain(..).collect();

        self.engine.process(
            &mut self.host,
            [inputs.get(0), inputs.get(1)],
            outputs.get_mut(0),
            outputs.get_mut(1),
            None,
            samples as u64,
            &events,
        )
    }

    fn process_events(&mut self, events: &Events) {
        for event in events.events() {
            while self.events.len() >= self.events.capacity() {
                self.events.pop_front();
            }

            match event {
                Event::Midi(ev) => {
                    println!("Got event {:?}", ev.data);
                    if let Some(ev) = MidiEvent::from_bytes(&ev.data) {
                        println!("Processing event {:?}", ev);
                        self.events.push_back(ev);
                    }
                }
                _ => {}
            }
        }
    }

    fn get_editor(&mut self) -> Option<Box<dyn Editor>> {
        Some(Box::new(LoopersVstGui {
            gui: Some(Gui::new(
                self.gui_receiver.clone(),
                self.gui_to_engine_sender.clone(),
                self.gui_sender.clone(),
            ))
        }))
    }
}

struct LoopersVstGui {
    gui: Option<Gui>,
}

impl vst::editor::Editor for LoopersVstGui {
    fn size(&self) -> (i32, i32) {
        (800, 600)
    }

    fn position(&self) -> (i32, i32) {
        (400, 400)
    }

    fn close(&mut self)  {
    }

    fn open(&mut self, parent: *mut c_void) -> bool {
        let gui = self.gui.take();
        if let Some(gui) = gui {
            gui.start();
            true
        } else {
            false
        }
    }

    fn is_open(&mut self) -> bool {
        true
    }
}

plugin_main!(LoopersVst);
