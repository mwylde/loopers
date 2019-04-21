use azul::{prelude::*, widgets::{label::Label, button::Button}};
use std::time::Duration;
use crate::{Message, Command, RecordMode, PlayMode};
use crossbeam_queue::SegQueue;
use std::sync::Arc;

pub struct Gui {
    state: GuiState,
}

#[derive(Clone)]
struct GuiState {
    record_state: RecordMode,
    play_state: PlayMode,
    time: i64,
    length: i64,
    input: Arc<SegQueue<Message>>,
    output: Arc<SegQueue<Command>>,
}

impl Gui {
    pub fn new() -> (Gui, Arc<SegQueue<Message>>, Arc<SegQueue<Command>>) {
        let input = Arc::new(SegQueue::new());
        let output = Arc::new(SegQueue::new());
        let gui = Gui {
            state: GuiState {
                record_state: RecordMode::NONE,
                play_state: PlayMode::PAUSED,
                time: 0,
                length: 0,
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

fn update_counter(app_state: &mut AppState<GuiState>, _info: &mut CallbackInfo<GuiState>) -> UpdateScreen {
    // app_state.data.modify(|state| state.counter += 1);
    Redraw
}

fn process_queue(state: &mut GuiState, _info: &mut AppResources) -> (UpdateScreen, TerminateTimer) {
    let mut update = DontRedraw;
    loop {
        match state.input.pop() {
            Ok(message) =>{
                update = Redraw;
                match message {
                    Message::RecordingStateChanged(mode, _) => {
                        state.record_state = mode;
                        println!("record mode = {:?}", mode);
                    },
                    Message::PlayingStateChanged(mode, _) => {
                        state.play_state = mode;
                        println!("play mode = {:?}", mode);
                    },
                    Message::TimeChanged(t) => state.time = t,
                    Message::LengthChanged(t) => state.length = t,
                }
            }
            Err(_) => break,
        }
    }

    (update, TerminateTimer::Continue)
}

impl Layout for GuiState {
    fn layout(&self, layout_info: LayoutInfo<Self>) -> Dom<Self> where Self: Sized {
        let time_label = Label::new(format!("{}", self.time / 1000)).dom();

        let width = if self.length == 0 {
            0.0
        } else {
            layout_info.window.state.size.dimensions.width * 1.3 * (self.time as f64 / self.length as f64)
        };

        let mut time_progress = Dom::new(NodeType::Div)
            .with_class("progress");

        time_progress.add_child(Dom::new(NodeType::Div)
            .with_class("progress-inner")
            .with_css_override(
                "progress_width", CssProperty::Width(LayoutWidth(PixelValue::const_px(width as isize)))));

        let record_label = Label::new(format!("record: {:?}", self.record_state)).dom();
        let play_label = Label::new(format!("play: {:?}", self.play_state)).dom();
        let button = Button::with_label("Update counter").dom()
            .with_callback(On::MouseUp, Callback(update_counter));

        Dom::new(NodeType::Div)
            .with_id("wrapper")
            .with_child(time_label)
            .with_child(time_progress)
            .with_child(record_label)
            .with_child(play_label)
            .with_child(button)
    }
}
