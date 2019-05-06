use std::time::Duration;
use crate::{Message, Command, RecordMode, PlayMode};
use crossbeam_queue::SegQueue;
use std::sync::Arc;
use std::collections::HashMap;
use tui::Terminal;
use tui::backend::TermionBackend;
use termion::raw::{IntoRawMode, RawTerminal};
use std::io;
use tui::widgets::{Widget, Block, Borders, Gauge};
use std::thread::sleep;
use std::thread;
use termion::input::{Keys, TermRead};
use termion::event::{Key, MouseButton};
use std::io::{Stdout, Write};
use tui::layout::{Layout, Direction, Constraint};
use termion::screen::AlternateScreen;
use tui::style::{Style, Color};


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

    pub fn run(&mut self) {
        let mut screen = AlternateScreen::from(io::stdout());

        let stdout = io::stdout().into_raw_mode().unwrap();
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.hide_cursor().unwrap();
        terminal.clear().unwrap();

        let tick_rate = 1000 / 60;
        self.render(&mut terminal);

        let (tx, rx) = crossbeam_channel::unbounded();
        thread::spawn(move|| {
            loop {
                let stdin = io::stdin();
                for evt in stdin.keys() {
                    match evt {
                        Ok(key) => {
                            tx.send(key).unwrap();
                        }
                        Err(_) => {}
                    }
                }

            }
        });

        loop {
            if process_queue(&mut self.state) {
                self.render(&mut terminal);
            }

            for key in rx.clone().try_iter() {
                info!("got key {:?}", key);
                match key {
                    Key::Char('q') => {
                        screen.flush().unwrap();
                        return
                    }
                    _ => {}
                }
            }

            sleep(Duration::from_millis(tick_rate));
        }
    }

    fn render(&mut self, terminal: &mut Terminal<TermionBackend<RawTerminal<Stdout>>>) {
        terminal.draw(|mut f| {
            let mut constraints: Vec<Constraint> = self.state.loopers.iter().map(|_| Constraint::Max(15)).collect();
            constraints.push(Constraint::Max(0));
            let global_layout = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(constraints)
                .split(f.size());

            for (i, looper) in self.state.loopers.iter().enumerate() {
                let mut border_style = Style::default();
                if self.state.active == looper.uuid {
                    border_style = border_style.fg(Color::Green);
                }

                Block::default()
                    .title(&format!("looper {}", i))
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .render(&mut f, global_layout[i]);

                let time_label = format!("{}", looper.time / 1000);

                let progress = if looper.length == 0 {
                    0
                } else {
                    ((looper.time as f64 / looper.length as f64) * 100.0) as u16
                };

                let layout = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(2)
                    .constraints([
                        Constraint::Percentage(50),
                        Constraint::Percentage(50)
                    ].as_ref())
                    .split(global_layout[i]);

                Gauge::default()
                    // .block(Block::default().title("Gauge1").borders(Borders::ALL))
                    .style(Style::default().fg(Color::Yellow))
                    .percent(progress)
                    .label(&time_label)
                    .render(&mut f, layout[0]);

                let controls =  Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                    ].as_ref())
                    .split(layout[1]);

                fn button(title: &str, active: bool) -> Block {
                    let mut b = Block::default()
                        .borders(Borders::ALL)
                        .title(title);

                    if active {
                        b = b.style(Style::default().bg(Color::Red));
                    }

                    b
                }


            button("record", looper.record_state == RecordMode::RECORD)
                .render(&mut f, controls[0]);
            button("overdub", looper.record_state == RecordMode::OVERDUB)
                .render(&mut f, controls[1]);
            button("play", looper.play_state == PlayMode::PLAYING)
                .render(&mut f, controls[2]);
            }
//            let looper = self.state.loopers[0];
//            let looper_layout = Layout::default()
//                .direction(Direction::Vertical)
//                .margin(2)
//                .split(global_layout[0].)
        });
    }
}

fn process_queue(state: &mut GuiState) -> bool {
    let mut update = false;
    loop {
        match state.input.pop() {
            Ok(message) =>{
                update = true;
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

    update
}

//fn button<T: Sized>(label: &'static str, active: bool) -> Dom<T> {
//    let mut button = Button::with_label(label).dom().with_class("control");
//    if active {
//        button.add_class("active");
//    }
//    button
//}
//
//impl Layout for GuiState {
//    fn layout(&self, layout_info: LayoutInfo<Self>) -> Dom<Self> where Self: Sized {
//        let mut loopers = Dom::new(NodeType::Div).with_id("loopers");
//
//        for looper in &self.loopers {
//
//            let time_label = Label::new(format!("{}", looper.time / 1000)).dom();
//
//            let width = if looper.length == 0 {
//                0.0
//            } else {
//                layout_info.window.state.size.dimensions.width * 1.3 * (looper.time as f64 / looper.length as f64)
//            };
//
//            let mut time_progress = Dom::new(NodeType::Div)
//                .with_class("progress");
//
//            time_progress.add_child(Dom::new(NodeType::Div)
//                .with_class("progress-inner")
//                .with_css_override(
//                    "progress_width", CssProperty::Width(LayoutWidth(PixelValue::const_px(width as isize)))));
//
////            let record_label = Label::new(format!("record: {:?}", looper.record_state)).dom();
////            let play_label = Label::new(format!("play: {:?}", looper.play_state)).dom();
//
//            let mut buttons = Dom::new(NodeType::Div)
//                .with_class("controls");
//            buttons.add_child(button("record", looper.record_state == RecordMode::RECORD));
//            buttons.add_child(button("overdub", looper.record_state == RecordMode::OVERDUB));
//            buttons.add_child(button("play", looper.play_state == PlayMode::PLAYING));
//
//            let mut parent = Dom::new(NodeType::Div)
//                .with_class("looper")
//                .with_child(time_label)
//                .with_child(time_progress)
//                .with_child(buttons);
//
//            if self.active == looper.uuid {
//                parent.add_class("active");
//            }
//
//            loopers.add_child(parent)
//        }
//
//        Dom::new(NodeType::Div)
//            .with_id("wrapper")
//            .with_child(loopers)
//    }
//}
