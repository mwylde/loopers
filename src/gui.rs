use azul::{prelude::*, widgets::{label::Label, button::Button}};
use std::time::Duration;
use crate::{Message, Command, RecordMode, PlayMode};
use crossbeam_queue::SegQueue;
use std::sync::Arc;
use std::collections::HashMap;
use azul::dom::NodeType::Div;

#[derive(Clone)]
struct LooperState {
    uuid: u128,
    record_state: RecordMode,
    play_state: PlayMode,
    time: i64,
    length: i64,
}

impl LooperState {
    fn new(uuid: u128) -> LooperState {
        LooperState {
            uuid,
            record_state: RecordMode::NONE,
            play_state: PlayMode::PAUSED,
            time: 0,
            length: 0,
        }
    }
}

pub struct Gui {
    state: GuiState,
}


#[derive(Clone)]
struct GuiState {
    loopers: Vec<LooperState>,
    active: u128,
    input: Arc<SegQueue<Message>>,
    output: Arc<SegQueue<Command>>,
}

impl GuiState {
    fn apply_to_loop(&mut self, uuid: u128, f: &Fn(&mut LooperState) -> ()) {
        if let Some(looper) = self.loopers.iter_mut().find(|f| f.uuid == uuid) {
            f(looper)
        } else {
            println!("could not update unknown loop {}", uuid)
        }
    }
}

impl Gui {
    pub fn new() -> (Gui, Arc<SegQueue<Message>>, Arc<SegQueue<Command>>) {
        let input = Arc::new(SegQueue::new());
        let output = Arc::new(SegQueue::new());
        let gui = Gui {
            state: GuiState {
                loopers: vec![],
                active: 0,
                input: input.clone(),
                output: output.clone(),
            },
        };
        (gui, input, output)
    }

    pub fn run(&self) {
        let mut app = App::new(self.state.clone(), AppConfig::default()).unwrap();
        let mut window_options = WindowCreateOptions::default();
        window_options.state.title = "Loopers".to_string();
        window_options.state.size.dimensions.width = 600.0;
        window_options.state.size.dimensions.height = 400.0;

        macro_rules! CSS_PATH { () => (concat!(env!("CARGO_MANIFEST_DIR"), "/styles/main.css")) }

        println!("Loading CSS from {}", CSS_PATH!());

        #[cfg(debug_assertions)]
        let window = {
            let hot_reloader = css::hot_reload(CSS_PATH!(), Duration::from_millis(500));
            app.create_hot_reload_window(window_options, hot_reloader).unwrap()
        };

        #[cfg(not(debug_assertions))]
        let window = {
            let css = css::from_str(include_str!(CSS_PATH!())).unwrap();
            app.create_window(window_options, css).unwrap()
        };

        let timer = Timer::new(process_queue).with_interval(Duration::from_millis(5));

        app.app_state.add_timer(TimerId::new(), timer);
        app.run(window).unwrap();
    }
}

fn process_queue(state: &mut GuiState, _info: &mut AppResources) -> (UpdateScreen, TerminateTimer) {
    let mut update = DontRedraw;
    loop {
        match state.input.pop() {
            Ok(message) =>{
                update = Redraw;
                match message {
                    Message::LoopCreated(uuid) => {
                        state.loopers.push(LooperState::new(uuid))
                    }
                    Message::LoopDestroyed(uuid) => {
                        state.loopers.retain(|s| s.uuid != uuid)
                    }
                    Message::RecordingStateChanged(mode, uuid) => {
                        state.apply_to_loop(uuid, &|l| l.record_state = mode)
                    },
                    Message::PlayingStateChanged(mode, uuid) => {
                        state.apply_to_loop(uuid, &|l| l.play_state = mode)
                    },
                    Message::TimeChanged(t, uuid) => {
                        state.apply_to_loop(uuid, &|l| l.time = t)
                    },
                    Message::LengthChanged(t, uuid) => {
                        state.apply_to_loop(uuid, &|l| l.length = t)
                    },
                    Message::ActiveChanged(uuid) => {
                        state.active = uuid
                    }
                }
            }
            Err(_) => break,
        }
    }

    (update, TerminateTimer::Continue)
}

fn button<T: Sized>(label: &'static str, active: bool) -> Dom<T> {
    let mut button = Button::with_label(label).dom().with_class("control");
    if active {
        button.add_class("active");
    }
    button
}

impl Layout for GuiState {
    fn layout(&self, layout_info: LayoutInfo<Self>) -> Dom<Self> where Self: Sized {
        let mut loopers = Dom::new(NodeType::Div).with_id("loopers");

        for looper in &self.loopers {

            let time_label = Label::new(format!("{}", looper.time / 1000)).dom();

            let width = if looper.length == 0 {
                0.0
            } else {
                layout_info.window.state.size.dimensions.width * 1.3 * (looper.time as f64 / looper.length as f64)
            };

            let mut time_progress = Dom::new(NodeType::Div)
                .with_class("progress");

            time_progress.add_child(Dom::new(NodeType::Div)
                .with_class("progress-inner")
                .with_css_override(
                    "progress_width", CssProperty::Width(LayoutWidth(PixelValue::const_px(width as isize)))));

//            let record_label = Label::new(format!("record: {:?}", looper.record_state)).dom();
//            let play_label = Label::new(format!("play: {:?}", looper.play_state)).dom();

            let mut buttons = Dom::new(NodeType::Div)
                .with_class("controls");
            buttons.add_child(button("record", looper.record_state == RecordMode::RECORD));
            buttons.add_child(button("overdub", looper.record_state == RecordMode::OVERDUB));
            buttons.add_child(button("play", looper.play_state == PlayMode::PLAYING));

            let mut parent = Dom::new(NodeType::Div)
                .with_class("looper")
                .with_child(time_label)
                .with_child(time_progress)
                .with_child(buttons);

            if self.active == looper.uuid {
                parent.add_class("active");
            }

            loopers.add_child(parent)
        }

        Dom::new(NodeType::Div)
            .with_id("wrapper")
            .with_child(loopers)
    }
}
