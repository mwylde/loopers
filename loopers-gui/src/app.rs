use crate::{skia::BACKGROUND_COLOR, AppData, Controller, GuiEvent, LooperData, MouseEventType};

use crate::widgets::{
    draw_circle_indicator, Button, ButtonState, ControlButton, ModalManager, PotWidget,
    TextEditState, TextEditable,
};
use loopers_common::api::{
    get_sample_rate, Command, FrameTime, LooperCommand, LooperMode, LooperSpeed, LooperTarget,
    Part, QuantizationMode, PARTS,
};
use loopers_common::gui_channel::EngineState;
use loopers_common::music::{MetricStructure, TimeSignature};
use regex::Regex;
use sdl2::mouse::MouseButton;
use skia_safe::gpu::SurfaceOrigin;
use skia_safe::paint::Style;
use skia_safe::path::Path;
use skia_safe::Rect;
use skia_safe::*;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use loopers_common::clamp;

const LOOP_ICON: &[u8] = include_bytes!("../resources/icons/loop.png");
const METRONOME_ICON: &[u8] = include_bytes!("../resources/icons/metronome.png");

fn color_for_mode(mode: LooperMode) -> Color {
    match mode {
        LooperMode::Recording => Color::from_rgb(228, 58, 44),
        LooperMode::Overdubbing => Color::from_rgb(101, 191, 171),
        LooperMode::Playing | LooperMode::Soloed => Color::from_rgb(85, 180, 95),
        LooperMode::Muted => Color::from_rgb(178, 178, 178),
    }
}

fn dark_color_for_mode(mode: LooperMode) -> Color {
    match mode {
        LooperMode::Recording => Color::from_rgb(138, 42, 0),
        LooperMode::Overdubbing => Color::from_rgb(0, 138, 138),
        LooperMode::Playing => Color::from_rgb(63, 137, 0),
        LooperMode::Soloed => Color::from_rgb(63, 137, 0),
        LooperMode::Muted => Color::from_rgb(69, 69, 69),
    }
}

#[allow(dead_code)]
enum AnimationFunction {
    Linear,
    EaseInQuad,
    EaseOutQuad,
    EaseInCubic,
    EaseOutCubic,
}

impl AnimationFunction {
    fn value(&self, t: f32) -> f32 {
        match self {
            AnimationFunction::Linear => t,

            AnimationFunction::EaseInQuad => t * t,
            AnimationFunction::EaseOutQuad => t * (2.0 - t),

            AnimationFunction::EaseInCubic => t * t * t,
            AnimationFunction::EaseOutCubic => {
                let t = t - 1.0;
                t * t * t + 1.0
            }
        }
    }
}

struct FrameTimeAnimation {
    start_time: FrameTime,
    length: Duration,
    function: AnimationFunction,
}

impl FrameTimeAnimation {
    fn new(start_time: FrameTime, length: Duration, function: AnimationFunction) -> Self {
        FrameTimeAnimation {
            start_time,
            length,
            function,
        }
    }

    fn value(&self, time: FrameTime) -> f32 {
        let p = (time.to_ms() - self.start_time.to_ms()) as f32 / self.length.as_millis() as f32;
        self.function.value(p).min(1.0).max(0.0)
    }

    #[allow(dead_code)]
    fn done(&self, time: FrameTime) -> bool {
        time.to_ms() - self.start_time.to_ms() > self.length.as_millis() as f64
    }
}

struct ClockTimeAnimation {
    start_time: Instant,
    length: Duration,
    function: AnimationFunction,
}

impl ClockTimeAnimation {
    fn new(start_time: Instant, length: Duration, function: AnimationFunction) -> Self {
        Self {
            start_time,
            length,
            function,
        }
    }

    fn value(&self, time: Instant) -> f32 {
        let p = time
            .checked_duration_since(self.start_time)
            .unwrap_or(Duration::new(0, 0))
            .as_millis() as f32
            / self.length.as_millis() as f32;
        self.function.value(p).min(1.0).max(0.0)
    }

    fn done(&self, time: Instant) -> bool {
        time.checked_duration_since(self.start_time)
            .map(|d| d >= self.length)
            .unwrap_or(false)
    }
}

const LOOPER_MARGIN: f32 = 10.0;
const LOOPER_HEIGHT: f32 = 80.0;
const BOTTOM_MARGIN: f32 = 140.0;
const WAVEFORM_OFFSET_X: f32 = 100.0;
const LOOPER_CIRCLE_INDICATOR_WIDTH: f32 = 50.0;
const WAVEFORM_RIGHT_MARGIN: f32 = 105.0;
const SAMPLES_PER_PIXEL: f32 = 720.0;

fn waveform_zero_offset() -> f32 {
    (2.0 * get_sample_rate() as f32) / SAMPLES_PER_PIXEL
}

struct AddButton {
    state: ButtonState,
}

impl AddButton {
    fn new() -> Self {
        AddButton {
            state: ButtonState::Default,
        }
    }

    fn draw(
        &mut self,
        canvas: &mut Canvas,
        _: &AppData,
        controller: &mut Controller,
        last_event: Option<GuiEvent>,
    ) {
        let mut p = Path::new();
        p.move_to((0.0, 15.0));
        p.line_to((30.0, 15.0));
        p.move_to((15.0, 0.0));
        p.line_to((15.0, 30.0));

        let on_click = |button: MouseButton| {
            if button == MouseButton::Left {
                controller.send_command(Command::AddLooper, "Failed to add looper");
            };
        };

        self.handle_event(canvas, p.bounds(), on_click, last_event);

        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_style(Style::Stroke);

        paint.set_color(match self.state {
            ButtonState::Default => Color::from_rgb(180, 180, 180),
            ButtonState::Hover => Color::from_rgb(255, 255, 255),
            ButtonState::Pressed => Color::from_rgb(30, 255, 30),
        });

        paint.set_stroke_width(5.0);

        canvas.draw_path(&p, &paint);
    }
}

impl Button for AddButton {
    fn set_state(&mut self, state: ButtonState) {
        self.state = state;
    }

    fn get_state(&self) -> ButtonState {
        self.state
    }
}

struct DeleteButton {
    state: ButtonState,
}

impl DeleteButton {
    fn new() -> Self {
        DeleteButton {
            state: ButtonState::Default,
        }
    }

    fn draw(
        &mut self,
        canvas: &mut Canvas,
        looper: &LooperData,
        size: f32,
        controller: &mut Controller,
        last_event: Option<GuiEvent>,
    ) {
        let mut p = Path::new();
        p.move_to((0.0, 0.0));
        p.line_to((size, size));
        p.move_to((size, 0.0));
        p.line_to((0.0, size));

        let on_click = |button: MouseButton| {
            if button == MouseButton::Left {
                controller.send_command(
                    Command::Looper(LooperCommand::Delete, LooperTarget::Id(looper.id)),
                    "Failed to delete looper",
                );
            };
        };

        self.handle_event(
            canvas,
            &Rect::from_size((size, size)).with_outset((5.0, 5.0)),
            on_click,
            last_event,
        );

        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_style(Style::Stroke);

        let mut circle_paint = paint.clone();

        paint.set_color(match self.state {
            ButtonState::Default => Color::from_rgb(150, 0, 00),
            ButtonState::Hover => Color::from_rgb(255, 0, 0),
            ButtonState::Pressed => Color::WHITE,
        });

        circle_paint.set_color(match self.state {
            ButtonState::Default => Color::from_rgb(150, 0, 00),
            ButtonState::Hover | ButtonState::Pressed => Color::from_rgb(255, 0, 0),
        });

        circle_paint.set_color(Color::from_rgb(200, 0, 0));
        circle_paint.set_stroke_width(1.0);
        if self.state == ButtonState::Pressed {
            circle_paint.set_style(Style::Fill);
        }
        canvas.draw_circle((size / 2.0, size / 2.0), size, &circle_paint);

        paint.set_stroke_width(3.0);
        canvas.draw_path(&p, &paint);
    }
}

impl Button for DeleteButton {
    fn set_state(&mut self, state: ButtonState) {
        self.state = state;
    }

    fn get_state(&self) -> ButtonState {
        self.state
    }
}
pub struct MainPage {
    loopers: BTreeMap<u32, LooperView>,
    beat_animation: Option<FrameTimeAnimation>,
    bottom_bar: BottomBarView,
    add_button: AddButton,
    bottom_buttons: BottomButtonView,
    modal_manager: ModalManager,
}

impl MainPage {
    pub fn new() -> Self {
        MainPage {
            loopers: BTreeMap::new(),
            beat_animation: None,
            bottom_bar: BottomBarView::new(),
            add_button: AddButton::new(),
            bottom_buttons: BottomButtonView::new(),
            modal_manager: ModalManager::new(),
        }
    }

    pub fn min_size(&self, data: &AppData) -> Size {
        let mut parts = HashMap::new();
        for l in data.loopers.values() {
            for part in PARTS.iter() {
                if l.parts[*part] {
                    *parts.entry(part).or_insert(0) += 1;
                }
            }
        }

        let max = parts.values().max().unwrap_or(&1);

        Size::new(
            800.0,
            *max as f32 * (LOOPER_HEIGHT + LOOPER_MARGIN) + BOTTOM_MARGIN,
        )
    }

    pub fn draw(
        &mut self,
        canvas: &mut Canvas,
        data: &AppData,
        w: f32,
        h: f32,
        controller: &mut Controller,
        last_event: Option<GuiEvent>,
    ) {
        // add views for new loopers
        for id in data.loopers.keys() {
            self.loopers
                .entry(*id)
                .or_insert_with(|| LooperView::new(*id));
        }

        // remove deleted loopers
        let remove: Vec<u32> = self
            .loopers
            .keys()
            .filter(|id| !data.loopers.contains_key(id))
            .map(|id| *id)
            .collect();

        for id in remove {
            self.loopers.remove(&id);
        }

        self.modal_manager
            .draw(canvas, w as f32, h as f32, data, controller, last_event);

        let mut y = 0.0;
        let mut visible_loopers = 0;
        for (id, looper) in self
            .loopers
            .iter_mut()
            .filter(|(id, _)| data.loopers[id].parts[data.engine_state.part])
        {
            visible_loopers += 1;
            canvas.save();
            canvas.translate(Vector::new(0.0, y));

            let size = looper.draw(canvas, data, &data.loopers[id], w, controller, last_event);

            y += size.height + LOOPER_MARGIN;

            canvas.restore();
        }

        // draw play head
        if visible_loopers > 0 {
            let x = waveform_zero_offset();
            let looper_h = y - 10.0;

            canvas.save();
            canvas.translate(Vector::new(WAVEFORM_OFFSET_X, 0.0));
            let mut path = Path::new();
            {
                path.move_to(Point::new(x - 5.0, 10.0));
                path.line_to(Point::new(x + 5.0, 10.0));
                path.move_to(Point::new(x, 10.0));
                path.line_to(Point::new(x, looper_h));
                path.move_to(Point::new(x - 5.0, looper_h));
                path.line_to(Point::new(x + 5.0, looper_h));
            }
            let mut paint = Paint::default();
            paint.set_anti_alias(true);

            // draw play head bar
            let beat = data
                .engine_state
                .metric_structure
                .tempo
                .beat(data.engine_state.time);
            let bom = data
                .engine_state
                .metric_structure
                .time_signature
                .beat_of_measure(beat);

            if bom == 0 && data.engine_state.time.0 >= 0 {
                if self.beat_animation.is_none() {
                    self.beat_animation = Some(FrameTimeAnimation::new(
                        data.engine_state.time,
                        Duration::from_millis(500),
                        AnimationFunction::EaseOutCubic,
                    ));
                }

                let v = self
                    .beat_animation
                    .as_ref()
                    .unwrap()
                    .value(data.engine_state.time);
                paint.set_stroke_width(3.0 + ((1.0 - v) * 5.0));
            } else {
                self.beat_animation = None;
                paint.set_stroke_width(3.0);
            }
            paint.set_color(Color::from_rgb(255, 255, 255));
            paint.set_style(Style::Stroke);

            if !data.loopers.is_empty() {
                canvas.draw_path(&path, &paint);
            }
            canvas.restore();
        }

        // draw the looper add button if we can fit it
        let max_loopers = ((h - BOTTOM_MARGIN) / (LOOPER_MARGIN + LOOPER_HEIGHT)).floor() as usize;
        if visible_loopers < max_loopers {
            canvas.save();
            canvas.translate((
                35.0,
                (LOOPER_HEIGHT + LOOPER_MARGIN) * visible_loopers as f32 + 50.0,
            ));

            self.add_button.draw(canvas, data, controller, last_event);

            canvas.restore();
        }

        let mut bottom = h as f32;

        // draw the message view if one exists
        canvas.save();
        canvas.translate((0.0, bottom - 90.0));
        LogMessageView::draw(canvas, data).width;
        canvas.restore();

        // draw the bottom bars
        if data.show_buttons {
            canvas.save();
            canvas.translate((10.0, bottom - 30.0));
            self.bottom_buttons
                .draw(canvas, data, controller, last_event);
            canvas.restore();
            bottom -= 30.0;
        };

        canvas.save();
        let bar_height = 30.0;
        canvas.translate(Vector::new(0.0, bottom - bar_height));
        self.bottom_bar.draw(
            data,
            w as f32,
            30.0,
            canvas,
            &mut self.modal_manager,
            controller,
            last_event,
        );
        canvas.restore();
    }
}

struct BottomBarView {
    tempo_view: TempoView,
    metronome_view: MetronomeView,
    metronome_button: MetronomeButton,
    time_view: TimeView,
    peak_view: PeakMeterView,
}

impl BottomBarView {
    fn new() -> Self {
        Self {
            tempo_view: TempoView::new(),
            metronome_view: MetronomeView::new(),
            metronome_button: MetronomeButton::new(),
            time_view: TimeView::new(),
            peak_view: PeakMeterView::new(30),
        }
    }

    fn draw(
        &mut self,
        data: &AppData,
        _w: f32,
        h: f32,
        canvas: &mut Canvas,
        _modal_manager: &mut ModalManager,
        controller: &mut Controller,
        last_event: Option<GuiEvent>,
    ) {
        let size = self.tempo_view.draw(canvas, data, controller, last_event);
        canvas.save();
        canvas.translate((size.width.round() + 20.0, 0.0));

        let size = self
            .metronome_view
            .draw(h, data, canvas, controller, last_event);
        canvas.translate((size.width.round(), 0.0));

        let size = self
            .metronome_button
            .draw(canvas, data, controller, last_event);
        canvas.translate((size.width.round() + 20.0, 0.0));

        let size = self.time_view.draw(h, data, canvas, controller, last_event);
        canvas.translate((size.width.round() + 20.0, 0.0));

        self.peak_view
            .draw(canvas, data.engine_state.input_levels, None, 160.0, h,
                  |_| {}, last_event);

        canvas.restore();
    }
}

struct TempoView {
    button_state: ButtonState,
    edit_state: TextEditState,
}

impl TempoView {
    fn new() -> Self {
        Self {
            button_state: ButtonState::Default,
            edit_state: TextEditState::Default,
        }
    }

    fn draw(
        &mut self,
        canvas: &mut Canvas,
        data: &AppData,
        controller: &mut Controller,
        last_event: Option<GuiEvent>,
    ) -> Size {
        let font = Font::new(Typeface::default(), 20.0);
        let text = &format!(
            "{} bpm",
            data.engine_state.metric_structure.tempo.bpm() as u32
        );
        let text_size = font.measure_str(text, None).1.size();

        let bounds =
            Rect::from_point_and_size(Point::new(15.0, 0.0), text_size).with_outset((10.0, 5.0));

        let mut edit_string = None;
        self.handle_event(
            canvas,
            &bounds,
            |button| {
                if button == MouseButton::Left {
                    edit_string = Some(format!(
                        "{}",
                        data.engine_state.metric_structure.tempo.bpm() as u32
                    ));
                }
            },
            last_event,
        );

        if let Some(s) = edit_string {
            self.start_editing(s);
        }

        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        let mut text_paint = Paint::default();
        text_paint.set_color(Color::WHITE);
        text_paint.set_anti_alias(true);

        if self.button_state != ButtonState::Default {
            match self.button_state {
                ButtonState::Hover => paint.set_color(Color::from_rgb(60, 60, 60)),
                ButtonState::Pressed => paint.set_color(Color::from_rgb(30, 30, 30)),
                ButtonState::Default => unreachable!(),
            };
            canvas.draw_rect(bounds, &paint);
        }

        canvas.draw_str(text, Point::new(15.0, 18.0), &font, &text_paint);

        self.draw_edit(canvas, &font, &bounds, controller, last_event);

        bounds.size()
    }
}

impl Button for TempoView {
    fn set_state(&mut self, state: ButtonState) {
        self.button_state = state;
    }

    fn get_state(&self) -> ButtonState {
        self.button_state
    }
}

impl TextEditable for TempoView {
    fn commit(&mut self, controller: &mut Controller) {
        if let TextEditState::Editing(_, s) = &self.edit_state {
            if let Ok(tempo) = f32::from_str(&s) {
                controller.send_command(Command::SetTempoBPM(tempo), "Failed to set tempo");
            } else if !s.is_empty() {
                controller.log(&format!("Tempo {} is not valid", s));
            }
        }

        self.edit_state = TextEditState::Default;
    }

    fn get_edit_state(&mut self) -> &mut TextEditState {
        &mut self.edit_state
    }

    fn is_valid(s: &str) -> bool {
        s.len() < 4 && u32::from_str(s).is_ok()
    }
}

struct MetronomeView {
    button_state: ButtonState,
    edit_state: TextEditState,
    beat_animation: (u8, Option<FrameTimeAnimation>),
}

impl MetronomeView {
    fn new() -> Self {
        MetronomeView {
            button_state: ButtonState::Default,
            edit_state: TextEditState::Default,
            beat_animation: (255, None),
        }
    }

    fn draw(
        &mut self,
        h: f32,
        data: &AppData,
        canvas: &mut Canvas,
        controller: &mut Controller,
        last_event: Option<GuiEvent>,
    ) -> Size {
        let upper = data.engine_state.metric_structure.time_signature.upper;

        let bounds = Rect::new(
            -15.0,
            -5.0,
            if upper <= 5 {
                upper as f32 * 30.0
            } else {
                70.0
            },
            h - 5.0,
        );

        let mut edit_string = None;
        self.handle_event(
            canvas,
            &bounds,
            |button| {
                if button == MouseButton::Left {
                    edit_string = Some(format!(
                        "{} / {}",
                        data.engine_state.metric_structure.time_signature.upper,
                        data.engine_state.metric_structure.time_signature.lower
                    ));
                }
            },
            last_event,
        );

        if self.button_state != ButtonState::Default {
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            match self.button_state {
                ButtonState::Hover => paint.set_color(Color::from_rgb(60, 60, 60)),
                ButtonState::Pressed => paint.set_color(Color::from_rgb(30, 30, 30)),
                ButtonState::Default => unreachable!(),
            };
            canvas.draw_rect(bounds, &paint);
        }

        if let Some(s) = edit_string {
            self.start_editing(s);
        }

        let current_beat = data
            .engine_state
            .metric_structure
            .tempo
            .beat(data.engine_state.time);
        let beat_of_measure = data
            .engine_state
            .metric_structure
            .time_signature
            .beat_of_measure(current_beat);

        let mut x = 0.0;

        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        let font = Font::new(Typeface::default(), 20.0);

        let beat_color = if beat_of_measure == 0 {
            color_for_mode(LooperMode::Playing)
        } else {
            Color::from_rgb(255, 239, 0)
        };

        if upper <= 5 {
            // if we have fewer than 5 beats per measure, render the full UI with each beat as
            // a circle
            for beat in 0..upper {
                if beat == beat_of_measure {
                    paint.set_color(beat_color);
                } else {
                    paint.set_color(Color::from_rgb(128, 128, 128));
                }

                let radius = 10.0;
                canvas.draw_circle(Point::new(x + radius / 2.0, h / 2.0 - 5.0), radius, &paint);
                x += 30.0;
            }
        } else {
            // otherwise, render a fixed-size UI with a single circle and a beat number
            if self.beat_animation.0 != beat_of_measure {
                self.beat_animation = (
                    beat_of_measure,
                    Some(FrameTimeAnimation::new(
                        data.engine_state.time,
                        Duration::from_millis(300),
                        AnimationFunction::EaseOutCubic,
                    )),
                );
            }

            let radius = 10.0;
            let x = 10.0 + radius / 2.0;
            paint.set_color(Color::from_rgb(128, 128, 128));
            canvas.draw_circle(Point::new(x, h / 2.0 - 5.0), radius, &paint);

            paint.set_color(beat_color);
            if let Some(animation) = &self.beat_animation.1 {
                paint.set_alpha(
                    ((1.0 - animation.value(data.engine_state.time)).min(1.0) * 255.0) as u8,
                );
            }
            canvas.draw_circle(Point::new(x, h / 2.0 - 5.0), radius, &paint);

            let mut text_paint = Paint::default();
            text_paint.set_anti_alias(true);
            text_paint.set_color(Color::WHITE);
            let lower = data.engine_state.metric_structure.time_signature.lower;

            let font = Font::new(Typeface::default(), 12.0);

            let x = x + radius * 2.0 + 10.0;
            canvas.draw_str(&upper.to_string(), (x, 10.0), &font, &text_paint);
            canvas.draw_str(&lower.to_string(), (x, 20.0), &font, &text_paint);
        }

        self.draw_edit(canvas, &font, &bounds, controller, last_event);

        bounds.size()
    }
}

impl Button for MetronomeView {
    fn set_state(&mut self, state: ButtonState) {
        self.button_state = state;
    }

    fn get_state(&self) -> ButtonState {
        self.button_state
    }
}

impl TextEditable for MetronomeView {
    fn commit(&mut self, controller: &mut Controller) {
        if let TextEditState::Editing(_, s) = &self.edit_state {
            let pat = Regex::new(r"(\d\d?)\s*/\s*(\d\d?)").unwrap();
            if let Some(captures) = pat.captures(s) {
                let upper = u8::from_str(captures.get(1).unwrap().as_str()).unwrap();
                let lower = u8::from_str(captures.get(2).unwrap().as_str()).unwrap();

                if TimeSignature::new(upper, lower).is_some() {
                    controller.send_command(
                        Command::SetTimeSignature(upper, lower),
                        "Failed to update time signature",
                    );
                } else {
                    controller.log(&format!("Time signature {} is not valid", s));
                }
            } else {
                controller.log(&format!("Time signature {} is not valid", s));
            }
        }

        self.edit_state = TextEditState::Default;
    }

    fn get_edit_state(&mut self) -> &mut TextEditState {
        &mut self.edit_state
    }
}

struct MetronomeButton {
    button_state: ButtonState,
    icon: Image,
}

impl MetronomeButton {
    fn new() -> Self {
        let icon_data = Data::new_copy(&METRONOME_ICON);
        let icon = Image::from_encoded(icon_data).expect("could not decode metronome icon");

        Self {
            button_state: ButtonState::Default,
            icon,
        }
    }

    fn draw(
        &mut self,
        canvas: &mut Canvas,
        data: &AppData,
        controller: &mut Controller,
        last_event: Option<GuiEvent>,
    ) -> Size {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);

        let bounds = Rect::new(0.0, -5.0, 25.0, 20.0);

        self.handle_event(
            canvas,
            &bounds,
            |button| {
                if button == MouseButton::Left {
                    let vol = if data.engine_state.metronome_volume < 0.5 {
                        100
                    } else {
                        0
                    };

                    controller.send_command(
                        Command::SetMetronomeLevel(vol),
                        "Failed to set metronome volume",
                    );
                }
            },
            last_event,
        );

        if self.button_state != ButtonState::Default {
            let mut paint = Paint::default();
            paint.set_anti_alias(true);

            match self.button_state {
                ButtonState::Hover => paint.set_color(Color::from_rgb(60, 60, 60)),
                ButtonState::Pressed => paint.set_color(Color::from_rgb(30, 30, 30)),
                ButtonState::Default => unreachable!(),
            };

            canvas.draw_rect(&bounds.with_outset((3.0, 3.0)), &paint);
        }

        paint.set_alpha_f(data.engine_state.metronome_volume.min(1.0).max(0.3));

        canvas.draw_image_rect_with_sampling_options(
            &self.icon, None, bounds,
            SamplingOptions::from_filter_quality(FilterQuality::High, None),
            &paint);

        Size::new(25.0, 25.0)
    }
}

impl Button for MetronomeButton {
    fn set_state(&mut self, state: ButtonState) {
        self.button_state = state;
    }

    fn get_state(&self) -> ButtonState {
        self.button_state
    }
}

struct TimeView {
    play_pause_button: PlayPauseButton,
    stop_button: StopButton,
}

impl TimeView {
    fn new() -> Self {
        Self {
            play_pause_button: PlayPauseButton::new(),
            stop_button: StopButton::new(),
        }
    }

    fn draw(
        &mut self,
        h: f32,
        data: &AppData,
        canvas: &mut Canvas,
        controller: &mut Controller,
        last_event: Option<GuiEvent>,
    ) -> Size {
        let mut ms = data.engine_state.time.to_ms();
        let mut negative = "";
        if ms < 0.0 {
            negative = "-";
            ms = -ms;
        }

        ms = (ms / 1000.0).floor();
        let hours = ms as u64 / 60 / 60;
        ms -= (hours * 60 * 60) as f64;
        let minutes = ms as u64 / 60;
        ms -= (minutes * 60) as f64;
        let seconds = ms as u64;

        let font = Font::new(Typeface::default(), 20.0);
        let mut text_paint = Paint::default();
        text_paint.set_color(Color::WHITE);
        text_paint.set_anti_alias(true);

        let time_blob = TextBlob::new(
            &format!("{}{:02}:{:02}:{:02}", negative, hours, minutes, seconds),
            &font,
        )
        .unwrap();

        let mut x = 10.0;
        canvas.draw_text_blob(&time_blob, Point::new(x, h - 12.0), &text_paint);

        x += 110.0;

        let current_beat = data
            .engine_state
            .metric_structure
            .tempo
            .beat(data.engine_state.time);
        let measure = data
            .engine_state
            .metric_structure
            .time_signature
            .measure(current_beat);
        let beat_of_measure = data
            .engine_state
            .metric_structure
            .time_signature
            .beat_of_measure(current_beat);

        let measure_blob =
            TextBlob::new(format!("{:03}.{}", measure, beat_of_measure), &font).unwrap();

        canvas.draw_text_blob(&measure_blob, Point::new(x, h - 12.0), &text_paint);
        x += 80.0;

        // draw play controls
        canvas.save();
        canvas.translate((x, 0.0));

        x += self
            .play_pause_button
            .draw(canvas, data, controller, last_event)
            .width
            + 10.0;
        canvas.restore();

        canvas.save();
        canvas.translate((x, 0.0));

        x += self
            .stop_button
            .draw(canvas, data, controller, last_event)
            .width;

        canvas.restore();

        Size::new(x, h)
    }
}

pub struct PeakMeterView {
    update_time: Duration,
    lines: usize,
    peaks: [(usize, Option<ClockTimeAnimation>); 2],
    levels: [usize; 2],
    image: Option<(Image, Instant)>,
    last_mouse_value: Option<f32>,
}

impl PeakMeterView {
    fn new(lines: usize) -> Self {
        Self {
            update_time: Duration::from_millis(80),
            lines,
            peaks: [(0, None), (0, None)],
            levels: [0, 0],
            image: None,
            last_mouse_value: None,
        }
    }

    fn color(lines: usize, i: usize) -> Color {
        let p = i as f32 / lines as f32;
        if p < 0.8 {
            Color::from_rgb(85, 180, 95)
        } else if p < 0.9 {
            Color::YELLOW
        } else {
            Color::RED
        }
    }

    fn y(i: usize, h: f32) -> f32 {
        2.0 + i as f32 * (h / 2.0 - 3.0)
    }

    fn redraw_if_needed(&mut self, canvas: &mut Canvas, paint: &mut Paint, w: f32, h: f32) {
        if let Some((_, instant)) = &self.image {
            if instant.elapsed() < self.update_time {
                return;
            }
        }

        let image_info = ImageInfo::new_n32((w as i32, h as i32), AlphaType::Premul, None);

        let mut surface = Surface::new_render_target(
            &mut canvas.recording_context().unwrap(),
            Budgeted::Yes,
            &image_info,
            None,
            SurfaceOrigin::TopLeft,
            None,
            None,
        )
        .unwrap();

        // draw level
        for (c, v) in self.levels.iter().enumerate() {
            for i in 0..self.lines {
                let y = Self::y(c, h);
                let mut path = Path::new();
                let x = i as f32 / self.lines as f32 * w;

                if i < *v {
                    paint.set_color(Self::color(self.lines, i));
                } else {
                    paint.set_color(Color::from_rgb(150, 150, 150));
                }

                path.move_to((x, y));
                path.line_to((x, y + h / 2.0 - 7.0));
                surface.canvas().draw_path(&path, &paint);
            }
        }
        self.levels = [0, 0];

        self.image = Some((surface.image_snapshot(), Instant::now()));
    }

    fn draw<F: FnOnce(f32)>(&mut self, canvas: &mut Canvas, levels: [u8; 2], set_level: Option<f32>,
                            w: f32, h: f32, new_level: F, last_event: Option<GuiEvent>) -> Size {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_stroke_width(1.5);
        paint.set_style(Style::Stroke);

        let cur_time = Instant::now();

        for ((now, (peak, animation)), ref mut level) in levels
            .iter()
            .zip(self.peaks.iter_mut())
            .zip(self.levels.iter_mut())
        {
            let v = (*now as f32 / 100.0 * self.lines as f32) as usize;

            // update our peaks (which are persisted for 1.2 seconds)
            if v > *peak {
                *peak = v;
                *animation = Some(ClockTimeAnimation::new(
                    cur_time,
                    Duration::from_millis(1200),
                    AnimationFunction::Linear,
                ));
            } else if animation.is_none() || animation.as_ref().unwrap().done(cur_time) {
                *peak = v;
            }

            // update our levels, which are also peak calculations but on a much shorter time frame
            if v > **level {
                **level = v;
            }
        }

        self.redraw_if_needed(canvas, &mut paint, w, h);

        if let Some((image, _)) = &self.image {
            let mut paint = Paint::default();
            paint.set_anti_alias(true);

            canvas.draw_image_with_sampling_options(
                &image, (0.0, 0.0),
                SamplingOptions::from_filter_quality(FilterQuality::High, None),
                Some(&paint));
        }

        // draw peak
        for (i, (peak, animation)) in self.peaks.iter().enumerate() {
            let y = Self::y(i, h);
            let mut path = Path::new();
            let x = *peak as f32 / self.lines as f32 * w;
            paint.set_color(Self::color(self.lines, *peak));
            if let Some(animation) = animation {
                paint.set_alpha_f(1.0 - animation.value(cur_time));
            }
            path.move_to((x, y));
            path.line_to((x, y + h / 2.0 - 7.0));
            canvas.draw_path(&path, &paint);
        }

        // if we have a level control, draw that over the vis
        if let Some(level) = set_level {
            let level = clamp(level, 0.0, 1.0);
            let mut paint = Paint::default();
            paint.set_color(Color::WHITE);
            paint.set_alpha_f(0.9);
            paint.set_anti_alias(true);
            paint.set_stroke_width(2.0);
            paint.set_style(Style::Stroke);

            let mut path = Path::new();
            path.move_to((w * level, -5.0));
            path.line_to((w * level, h));
            canvas.draw_path(&path, &paint);

            // handle clicks
            let bounds = Rect::from_size((w, h));
            if let Some(GuiEvent::MouseEvent(MouseEventType::MouseDown(MouseButton::Left), (x, y))) = last_event
            {
                let point = canvas
                    .local_to_device_as_3x3()
                    .invert()
                    .unwrap()
                    .map_point((x as f32, y as f32));

                if bounds.contains(point) {
                    new_level(point.x / w);
                    self.last_mouse_value = Some(x as f32);
                }
            } else if let Some(GuiEvent::MouseEvent(MouseEventType::Moved, (x, _))) = last_event {
                if let Some(p_x) = self.last_mouse_value {
                    let lv = clamp(level + (x as f32 - p_x) / w, 0.0, 1.0);
                    new_level(lv);
                    self.last_mouse_value = Some(x as f32);
                }
            }

            if let Some(GuiEvent::MouseEvent(MouseEventType::MouseUp(_), _)) = last_event {
                self.last_mouse_value = None;
            }
        }


        Size::new(w, h)
    }
}

struct PlayPauseButton {
    button_state: ButtonState,
}

impl PlayPauseButton {
    fn new() -> Self {
        Self {
            button_state: ButtonState::Default,
        }
    }

    fn draw(
        &mut self,
        canvas: &mut Canvas,
        data: &AppData,
        controller: &mut Controller,
        last_event: Option<GuiEvent>,
    ) -> Size {
        let bounds = Rect::new(0.0, -5.0, 25.0, 20.0);

        self.handle_event(
            canvas,
            &bounds,
            |button| {
                if button == MouseButton::Left {
                    let command = if data.engine_state.engine_state == EngineState::Active {
                        Command::Pause
                    } else {
                        Command::Start
                    };

                    controller.send_command(command, "Failed to send command to engine");
                }
            },
            last_event,
        );

        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color(Color::WHITE);
        paint.set_style(Style::Fill);

        if self.button_state != ButtonState::Default {
            match self.button_state {
                ButtonState::Hover => paint.set_color(Color::from_rgb(120, 120, 120)),
                ButtonState::Pressed => paint.set_color(Color::from_rgb(60, 60, 60)),
                ButtonState::Default => unreachable!(),
            };
        }

        if data.engine_state.engine_state == EngineState::Stopped {
            // draw play icon
            let mut path = Path::new();
            path.move_to((0.0, 0.0));
            path.line_to((0.0, 20.0));
            path.line_to((20.0, 10.0));
            path.line_to((0.0, 0.0));
            path.close();
            canvas.draw_path(&path, &paint);
        } else {
            // draw pause button
            let rect1 = Rect::new(0.0, 0.0, 7.5, 20.0);
            let rect2 = Rect::new(12.5, 0.0, 20.0, 20.0);
            canvas.draw_rect(&rect1, &paint);
            canvas.draw_rect(&rect2, &paint);
        }

        Size::new(25.0, 25.0)
    }
}

impl Button for PlayPauseButton {
    fn set_state(&mut self, state: ButtonState) {
        self.button_state = state;
    }

    fn get_state(&self) -> ButtonState {
        self.button_state
    }
}

struct StopButton {
    button_state: ButtonState,
}

impl StopButton {
    fn new() -> Self {
        Self {
            button_state: ButtonState::Default,
        }
    }

    fn draw(
        &mut self,
        canvas: &mut Canvas,
        _data: &AppData,
        controller: &mut Controller,
        last_event: Option<GuiEvent>,
    ) -> Size {
        let bounds = Rect::new(0.0, -5.0, 25.0, 20.0);

        self.handle_event(
            canvas,
            &bounds,
            |button| {
                if button == MouseButton::Left {
                    controller.send_command(Command::Stop, "Failed to stop engine");
                }
            },
            last_event,
        );

        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color(Color::WHITE);
        paint.set_style(Style::Fill);

        if self.button_state != ButtonState::Default {
            match self.button_state {
                ButtonState::Hover => paint.set_color(Color::from_rgb(120, 120, 120)),
                ButtonState::Pressed => paint.set_color(Color::from_rgb(60, 60, 60)),
                ButtonState::Default => unreachable!(),
            };
        }

        let rect = Rect::new(0.0, 0.0, 20.0, 20.0);
        canvas.draw_rect(&rect, &paint);

        Size::new(25.0, 25.0)
    }
}

impl Button for StopButton {
    fn set_state(&mut self, state: ButtonState) {
        self.button_state = state;
    }

    fn get_state(&self) -> ButtonState {
        self.button_state
    }
}

#[derive(Copy, Clone, PartialEq)]
enum BottomButtonBehavior {
    Save,
    Load,
    SetSyncMode(QuantizationMode),
    Part(Part),
    Undo,
    Redo,
}

struct LoadWindow {
    active: Arc<AtomicBool>,
}

impl LoadWindow {
    fn activate(&mut self, mut controller: Controller) {
        if !self.active.load(Ordering::Relaxed) {
            let active = self.active.clone();
            std::thread::spawn(move || {
                let dir = dirs::home_dir()
                    .map(|mut dir| {
                        dir.push("looper-sessions/");
                        dir.to_string_lossy().to_string()
                    })
                    .unwrap_or(PathBuf::new().to_string_lossy().to_string());

                // TODO: do this in a separate thread so it doesn't block the channel
                if let Some(file) = tinyfiledialogs::open_file_dialog(
                    "Open",
                    &dir,
                    Some((&["*.loopers"][..], "loopers project files")),
                ) {
                    controller.send_command(
                        Command::LoadSession(Arc::new(PathBuf::from(file))),
                        "Failed to send load command to engine",
                    );
                }

                active.store(false, Ordering::Relaxed);
            });
            self.active.store(true, Ordering::Relaxed);
        }
    }
}

struct BottomButtonView {
    buttons: Vec<(BottomButtonBehavior, ControlButton)>,
    load_window: LoadWindow,
}

impl BottomButtonView {
    fn new() -> Self {
        let c = Color::from_rgb(78, 78, 78);
        BottomButtonView {
            buttons: vec![
                (
                    BottomButtonBehavior::Save,
                    ControlButton::new("save", c, None, 22.0),
                ),
                (
                    BottomButtonBehavior::Load,
                    ControlButton::new("load", c, None, 22.0),
                ),
                (
                    BottomButtonBehavior::SetSyncMode(QuantizationMode::Free),
                    ControlButton::new("free", c, None, 22.0),
                ),
                (
                    BottomButtonBehavior::SetSyncMode(QuantizationMode::Beat),
                    ControlButton::new("beat", c, None, 22.0),
                ),
                (
                    BottomButtonBehavior::SetSyncMode(QuantizationMode::Measure),
                    ControlButton::new("measure", c, None, 22.0),
                ),
                (
                    BottomButtonBehavior::Part(Part::A),
                    ControlButton::new("A", c, None, 22.0),
                ),
                (
                    BottomButtonBehavior::Part(Part::B),
                    ControlButton::new("B", c, None, 22.0),
                ),
                (
                    BottomButtonBehavior::Part(Part::C),
                    ControlButton::new("C", c, None, 22.0),
                ),
                (
                    BottomButtonBehavior::Part(Part::D),
                    ControlButton::new("D", c, None, 22.0),
                ),
                (
                    BottomButtonBehavior::Undo,
                    ControlButton::new("Undo", c, None, 22.0)
                ),
                (
                    BottomButtonBehavior::Redo,
                    ControlButton::new("Redo", c, None, 22.0)
                ),
            ],
            load_window: LoadWindow {
                active: Arc::new(AtomicBool::new(false)),
            },
        }
    }

    fn draw(
        &mut self,
        canvas: &mut Canvas,
        data: &AppData,
        controller: &mut Controller,
        last_event: Option<GuiEvent>,
    ) -> Size {
        let mut x = 0.0;
        let load_window = &mut self.load_window;
        for (behavior, button) in &mut self.buttons {
            canvas.save();
            canvas.translate((x, 0.0));

            let behavior = behavior.clone();

            let on_click = |button: MouseButton| {
                if button == MouseButton::Left {
                    match behavior {
                        BottomButtonBehavior::Save => {
                            if let Some(mut home_dir) = dirs::home_dir() {
                                home_dir.push("looper-sessions");
                                controller.send_command(
                                    Command::SaveSession(Arc::new(home_dir)),
                                    "failed to send save command to engine",
                                );
                            } else {
                                controller.log("Could not determine home dir");
                            }
                        }
                        BottomButtonBehavior::Load => {
                            load_window.activate(controller.clone());
                        }
                        BottomButtonBehavior::Part(part) => {
                            controller
                                .send_command(Command::GoToPart(part), "Failed to change parts");
                        }
                        BottomButtonBehavior::SetSyncMode(mode) => {
                            controller.send_command(
                                Command::SetQuantizationMode(mode),
                                "Failed to set sync mode",
                            );
                        }
                        BottomButtonBehavior::Undo => {
                            controller.send_command(
                                Command::Looper(LooperCommand::Undo, LooperTarget::Selected),
                                "Failed to undo");
                        }
                        BottomButtonBehavior::Redo => {
                            controller.send_command(
                                Command::Looper(LooperCommand::Redo, LooperTarget::Selected),
                                "Failed to redo");
                        }
                    };
                }
            };

            let disabled = match behavior {
                BottomButtonBehavior::Undo => {
                    data.loopers.get(&data.engine_state.active_looper).map(|l| !l.has_undos)
                        .unwrap_or(false)
                }
                BottomButtonBehavior::Redo => {
                    data.loopers.get(&data.engine_state.active_looper).map(|l| !l.has_redos)
                        .unwrap_or(true)
                }
                _ => false,
            };

            let mut progress_percent = 0.0;

            if let BottomButtonBehavior::Part(part) = &behavior {
                progress_percent = data
                    .global_triggers
                    .iter()
                    .rev()
                    .filter(|(_, _, c)| match c {
                        Command::GoToPart(p) => p == part,
                        // TODO: Think about how to support this for previous part / next part
                        _ => false,
                    })
                    .min_by_key(|(_, t1, _)| t1.0)
                    .map(|(t0, t1, _)| {
                        if *t1 == *t0 {
                            0.0
                        } else {
                            (data.engine_state.time.0 as f32 - t0.0 as f32)
                                / (t1.0 as f32 - t0.0 as f32)
                        }
                    })
                    .unwrap_or(0.0);
            }

            let size = button.draw_with_progress(
                canvas,
                match behavior {
                    BottomButtonBehavior::Part(part) => data.engine_state.part == part,
                    BottomButtonBehavior::SetSyncMode(mode) => data.engine_state.sync_mode == mode,
                    _ => false,
                },
                disabled,
                on_click,
                last_event,
                progress_percent,
            );
            x += size.width + 10.0;

            if behavior == BottomButtonBehavior::Load
                || behavior == BottomButtonBehavior::SetSyncMode(QuantizationMode::Measure)
                || behavior == BottomButtonBehavior::Part(Part::D)
            {
                x += 30.0;
            }

            canvas.restore();
        }

        Size::new(x, 40.0)
    }
}

struct LogMessageView {}

impl LogMessageView {
    fn draw(canvas: &mut Canvas, data: &AppData) -> Size {
        let msg = data.messages.cur.as_ref().map(|(_, l)| l.as_str());
        if let Some(msg) = msg.as_ref() {
            let font = Font::new(Typeface::default(), Some(14.0));
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_color(Color::WHITE);

            if msg.len() > 0 {
                let blob = TextBlob::new(msg, &font).unwrap();
                let text_size = font.measure_str(msg, None).1.size();

                canvas.draw_text_blob(blob, Point::new(10.0, 16.0), &paint);

                text_size
            } else {
                Size::new(0.0, 0.03)
            }
        } else {
            Size::new(0.0, 0.0)
        }
    }
}

struct LooperView {
    id: u32,
    waveform_view: WaveformView,
    buttons: Vec<
        Vec<(
            Box<dyn FnMut(&mut Canvas, &LooperData, &mut Controller, Option<GuiEvent>) -> Size>,
            f32,
        )>,
    >,
    state: ButtonState,
    active_button: ActiveButton,
    delete_button: DeleteButton,
    pan: PotWidget,
    peak: PeakMeterView,
}

impl LooperView {
    fn new(id: u32) -> Self {
        let button_height = LOOPER_HEIGHT * 0.5 - 15.0;
        Self {
            id,
            waveform_view: WaveformView::new(),
            buttons: vec![
                vec![
                    // top row
                    (
                        Self::new_state_button(LooperMode::Recording, "record", button_height),
                        15.0,
                    ),
                    (
                        Self::new_state_button(LooperMode::Soloed, "solo", button_height),
                        15.0,
                    ),
                    (
                        Self::new_command_button(
                            "clear",
                            Color::YELLOW,
                            Command::Looper(LooperCommand::Clear, LooperTarget::Id(id)),
                            button_height,
                            100.0,
                        ),
                        15.0,
                    ),
                    (Self::new_part_button(Part::A, button_height), 0.5),
                    (Self::new_part_button(Part::B, button_height), 0.5),
                    (Self::new_part_button(Part::C, button_height), 0.5),
                    (Self::new_part_button(Part::D, button_height), 0.5),
                ],
                vec![
                    (
                        Self::new_state_button(LooperMode::Overdubbing, "overdub", button_height),
                        15.0,
                    ),
                    (
                        Self::new_state_button(LooperMode::Muted, "mute", button_height),
                        15.0,
                    ),
                    (
                        Self::new_speed_button(
                            "½x",
                            LooperSpeed::Half,
                            button_height,
                            45.0
                        ),
                        10.0
                    ),
                    (
                        Self::new_speed_button(
                            "2x",
                            LooperSpeed::Double,
                            button_height,
                            45.0
                        ),
                        15.0
                    )
                ],
            ],
            state: ButtonState::Default,
            active_button: ActiveButton::new(),
            delete_button: DeleteButton::new(),
            pan: PotWidget::new(35.0, Color::WHITE),
            peak: PeakMeterView::new(50),
        }
    }

    fn new_command_button(
        name: &str,
        color: Color,
        command: Command,
        h: f32,
        w: f32,
    ) -> Box<dyn FnMut(&mut Canvas, &LooperData, &mut Controller, Option<GuiEvent>) -> Size> {
        let mut button = ControlButton::new(name, color, Some(w), h);

        Box::new(move |canvas, _, controller, last_event| {
            button.draw(
                canvas,
                false,
                false,
                |button| {
                    if button == MouseButton::Left {
                        controller
                            .send_command(command.clone(), "Failed to send command to engine");
                    }
                },
                last_event,
            )
        })
    }

    fn new_speed_button(
        name: &str,
        speed: LooperSpeed,
        h: f32,
        w: f32,
    ) -> Box<dyn FnMut(&mut Canvas, &LooperData, &mut Controller, Option<GuiEvent>) -> Size> {
        let mut button = ControlButton::new(name, Color::LIGHT_GRAY, Some(w), h);

        Box::new(move |canvas, data, controller, last_event| {
            button.draw(
                canvas,
                data.speed == speed,
                false,
                |button| {
                    if button == MouseButton::Left {
                        let command = Command::Looper(LooperCommand::SetSpeed(if data.speed == speed {
                            LooperSpeed::One
                        } else {
                            speed
                        }), LooperTarget::Id(data.id));

                        controller.send_command(command, "Failed to send command to engine");
                    }
                },
                last_event,
            )
        })
    }

    fn new_part_button(
        part: Part,
        h: f32,
    ) -> Box<dyn FnMut(&mut Canvas, &LooperData, &mut Controller, Option<GuiEvent>) -> Size> {
        let mut button =
            ControlButton::new(part.name(), Color::from_rgb(78, 78, 78), Some(28.0), h);

        Box::new(move |canvas, data, controller, last_event| {
            button.draw(
                canvas,
                data.parts[part],
                false,
                |button| {
                    if button == MouseButton::Left {
                        let lc = if data.parts[part] {
                            LooperCommand::RemoveFromPart(part)
                        } else {
                            LooperCommand::AddToPart(part)
                        };

                        controller.send_command(
                            Command::Looper(lc, LooperTarget::Id(data.id)),
                            "Failed to send command to engine",
                        );
                    }
                },
                last_event,
            )
        })
    }

    fn new_state_button(
        mode: LooperMode,
        name: &str,
        h: f32,
    ) -> Box<dyn FnMut(&mut Canvas, &LooperData, &mut Controller, Option<GuiEvent>) -> Size> {
        let mut button = ControlButton::new(name, color_for_mode(mode), Some(100.0), h);

        Box::new(move |canvas, looper, controller, last_event| {
            button.draw(
                canvas,
                looper.mode == mode,
                false,
                |button| {
                    if button == MouseButton::Left {
                        use LooperMode::*;
                        let command = match (looper.mode, mode) {
                            (Recording, Recording) => Some(LooperCommand::Overdub),
                            (_, Recording) => Some(LooperCommand::Record),
                            (Overdubbing, Overdubbing) => Some(LooperCommand::Play),
                            (_, Overdubbing) => Some(LooperCommand::Overdub),
                            (Muted, Muted) => Some(LooperCommand::Play),
                            (_, Muted) => Some(LooperCommand::Mute),
                            (Soloed, Soloed) => Some(LooperCommand::Play),
                            (_, Soloed) => Some(LooperCommand::Solo),
                            (s, t) => {
                                warn!("unhandled button state ({:?}, {:?})", s, t);
                                None
                            }
                        };

                        if let Some(command) = command {
                            controller.send_command(
                                Command::Looper(command, LooperTarget::Id(looper.id)),
                                "Failed to update looper mode",
                            );
                        }
                    }
                },
                last_event,
            )
        })
    }

    fn draw(
        &mut self,
        canvas: &mut Canvas,
        data: &AppData,
        looper: &LooperData,
        w: f32,
        controller: &mut Controller,
        last_event: Option<GuiEvent>,
    ) -> Size {
        assert_eq!(self.id, looper.id);

        let time = data.engine_state.time - looper.offset;

        let ratio = if looper.length == 0 || looper.mode == LooperMode::Recording {
            0f32
        } else {
            (time.0.rem_euclid(looper.length as i64)) as f32 / looper.length as f32
        };

        // Draw loop completion indicate
        draw_circle_indicator(
            canvas,
            color_for_mode(looper.mode_with_solo(data)),
            ratio,
            LOOPER_CIRCLE_INDICATOR_WIDTH / 2.0,
            LOOPER_CIRCLE_INDICATOR_WIDTH / 2.0,
            LOOPER_CIRCLE_INDICATOR_WIDTH / 2.0,
        );

        if looper.speed != LooperSpeed::One {
            let mut paint = Paint::default();

            let font = Font::new(Typeface::default(), 21.0);
            let (text, x) = match looper.speed {
                LooperSpeed::Half => ("½x", 35.0),
                LooperSpeed::Double => ("2x", 40.0),
                LooperSpeed::One => unreachable!(),
            };

            // draw shadow
            paint.set_color(Color::BLACK);
            paint.set_anti_alias(true);
            paint.set_alpha_f(0.9);
            paint.set_mask_filter(MaskFilter::blur(BlurStyle::Normal, 3.4, None));

            canvas.draw_str(text, Point::new(x + 1.0, 56.0), &font, &paint);

            // draw text
            paint.set_color(Color::WHITE);
            paint.set_alpha_f(1.0);
            paint.set_mask_filter(None);

            canvas.draw_str(text, Point::new(x, 55.0), &font, &paint);
        }


        let waveform_width = w - WAVEFORM_OFFSET_X - WAVEFORM_RIGHT_MARGIN;

        let bounds = Rect::from_size((waveform_width, LOOPER_HEIGHT))
            .with_offset((10.0, 10.0))
            .with_outset((WAVEFORM_OFFSET_X - 7.0, 5.0));

        // sets our state, which tells us if the mouse is hovering
        self.handle_event(canvas, &bounds, |_| {}, last_event);

        // Draw waveform
        canvas.save();
        canvas.translate(Vector::new(WAVEFORM_OFFSET_X, 10.0));
        self.waveform_view
            .draw(canvas, data, looper, waveform_width, LOOPER_HEIGHT);

        // draw pan and level controls
        canvas.save();
        canvas.translate((waveform_width + 15.0, 10.0));
        self.pan.draw(
            canvas,
            looper.pan,
            |pan| {
                controller.send_command(
                    Command::Looper(LooperCommand::SetPan(pan), LooperTarget::Id(looper.id)),
                    "Failed to set pan",
                );
            },
            last_event,
        );
        canvas.translate((0.0, 40.0));
        self.peak.draw(canvas, looper.levels, Some(looper.level), 70.0, 30.0,
                       |level| controller.send_command(
                           Command::Looper(LooperCommand::SetLevel(level), LooperTarget::Id(looper.id)),
                           "Failed to set level"
                       ), last_event);

        canvas.restore();

        // draw active button
        canvas.save();
        canvas.translate((waveform_width + 75.0, 20.0));
        self.active_button.draw(
            canvas,
            data.engine_state.active_looper == looper.id,
            |button| {
                if button == MouseButton::Left {
                    controller.send_command(
                        Command::SelectLooperById(looper.id),
                        "Failed to send select command to engine",
                    );
                }
            },
            last_event,
        );

        {
            // draw id text
            let mut paint = Paint::default();
            if data.engine_state.active_looper == looper.id {
                paint.set_color(Color::WHITE);
            } else {
                paint.set_color(Color::from_rgb(230, 230, 230));
            }
            paint.set_anti_alias(true);

            let font = Font::new(Typeface::default(), 12.0);
            let x = if looper.id > 9 { -8.0 } else { -4.0 };
            canvas.draw_str(&format!("{}", looper.id), Point::new(x, 4.0), &font, &paint);
        }

        canvas.restore();
        canvas.restore();

        if data.show_buttons
            && (self.state == ButtonState::Hover || self.state == ButtonState::Pressed)
        {
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_color(Color::from_argb(200, 0, 0, 0));
            canvas.draw_rect(&bounds, &paint);

            // draw delete button
            let delete_size = 10.0;
            canvas.save();
            canvas.translate((45.0, 10.0 + LOOPER_HEIGHT / 2.0 - delete_size / 2.0));
            self.delete_button
                .draw(canvas, looper, delete_size, controller, last_event);
            canvas.restore();

            // draw
            let mut y = 20.0;
            for row in &mut self.buttons {
                let mut x = WAVEFORM_OFFSET_X + waveform_zero_offset() + 10.0;
                let mut button_height = 0f32;

                for (button, margin_right) in row {
                    canvas.save();
                    canvas.translate((x, y));
                    let bounds = (button)(canvas, looper, controller, last_event);
                    canvas.restore();

                    x += bounds.width + *margin_right;
                    button_height = button_height.max(bounds.height);
                }

                y += button_height + 10.0;
            }
        } else {
            // draw overlay to darken time that is past
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_blend_mode(BlendMode::Darken);
            paint.set_color(BACKGROUND_COLOR.clone().with_a(200));
            canvas.draw_rect(
                Rect::new(
                    WAVEFORM_OFFSET_X,
                    10.0,
                    WAVEFORM_OFFSET_X + waveform_zero_offset(),
                    LOOPER_HEIGHT + 10.0,
                ),
                &paint,
            );
        }

        canvas.restore();

        bounds.size()
    }
}

impl Button for LooperView {
    fn set_state(&mut self, state: ButtonState) {
        self.state = state;
    }

    fn get_state(&self) -> ButtonState {
        self.state
    }
}

const IMAGE_SCALE: f32 = 4.0;

type CacheUpdaterFn = fn(
    data: &AppData,
    looper: &LooperData,
    w: f32,
    h: f32,
    scale: f32,
    canvas: &mut Canvas,
) -> Size;

struct DrawCache<T: Eq + Copy> {
    image: Option<(Image, Size)>,
    key: Option<T>,
    draw_fn: CacheUpdaterFn,
    draw_count: usize,
}

impl<T: Eq + Copy> DrawCache<T> {
    fn new(draw_fn: CacheUpdaterFn) -> DrawCache<T> {
        DrawCache {
            image: None,
            key: None,
            draw_fn,
            draw_count: 0,
        }
    }

    fn draw_with_cache(
        &mut self,
        key: T,
        data: &AppData,
        looper: &LooperData,
        w: f32,
        h: f32,
        canvas: &mut Canvas,
    ) -> Option<Size> {
        let size = ((w * IMAGE_SCALE) as i32, (h * IMAGE_SCALE) as i32);

        let (image, size) = if self.key? != key
            || self.image.is_none()
            // this is a hack to get around textures being cleared from GPU memory after sleep
            // there's probably a better way to detect this, but I'm not sure what it is
            || self.draw_count > 300
            || self
            .image
            .as_ref()
            .map(|(i, _)| (i.width(), i.height()))?
            != size
        {
            let image_info = ImageInfo::new_n32(size, AlphaType::Premul, None);
            let mut surface = Surface::new_render_target(
                &mut canvas.recording_context()?,
                Budgeted::Yes,
                &image_info,
                None,
                SurfaceOrigin::TopLeft,
                None,
                None,
            )?;

            let draw_size = (self.draw_fn)(
                data,
                looper,
                w * IMAGE_SCALE,
                h * IMAGE_SCALE,
                IMAGE_SCALE,
                &mut surface.canvas(),
            );

            let image = surface.image_snapshot();
            self.image = Some((image, draw_size));
            self.key = Some(key);
            self.draw_count = 0;

            self.image.as_ref()?
        } else {
            self.draw_count += 1;
            self.image.as_ref()?
        };

        canvas.save();
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color(Color::from_rgb(255, 255, 0));
        canvas.scale((1.0 / IMAGE_SCALE, 1.0 / IMAGE_SCALE));
        canvas.draw_image_with_sampling_options(
            image, (0.0, 0.0),
            SamplingOptions::from_filter_quality(FilterQuality::High, None),
            Some(&paint));
        canvas.restore();

        Some(*size)
    }

    fn draw(
        &mut self,
        key: T,
        data: &AppData,
        looper: &LooperData,
        w: f32,
        h: f32,
        use_cache: bool,
        canvas: &mut Canvas,
    ) -> Size {
        if use_cache {
            if let Some(size) = self.draw_with_cache(key, data, looper, w, h, canvas) {
                return size;
            }
        }

        return (self.draw_fn)(data, looper, w, h, 1.0, canvas);
    }
}

struct ActiveButton {
    state: ButtonState,
}

impl ActiveButton {
    fn new() -> Self {
        Self {
            state: ButtonState::Default,
        }
    }

    fn draw<F: FnOnce(MouseButton) -> ()>(
        &mut self,
        canvas: &mut Canvas,
        is_active: bool,
        on_click: F,
        last_event: Option<GuiEvent>,
    ) {
        let bounds = Rect {
            left: -10.0,
            top: -10.0,
            right: 10.0,
            bottom: 10.0,
        };

        self.handle_event(canvas, &bounds, on_click, last_event);

        let mut active_paint = Paint::default();
        active_paint.set_anti_alias(true);
        if is_active {
            active_paint.set_color(color_for_mode(LooperMode::Recording));
            active_paint.set_style(Style::Fill);
        } else {
            active_paint.set_color(Color::from_rgb(230, 230, 230));
            if self.state == ButtonState::Default {
                active_paint.set_style(Style::Stroke);
            } else {
                active_paint.set_style(Style::Fill);
            }
        };

        canvas.draw_circle(Point::new(0.0, 0.0), 10.0, &active_paint);
    }
}

impl Button for ActiveButton {
    fn set_state(&mut self, state: ButtonState) {
        self.state = state;
    }

    fn get_state(&self) -> ButtonState {
        self.state
    }
}

struct WaveformView {
    waveform: DrawCache<(u64, FrameTime, LooperMode)>,
    beats: DrawCache<MetricStructure>,
    loop_icon: Image,
}

impl WaveformView {
    fn new() -> Self {
        let loop_icon_data = Data::new_copy(&LOOP_ICON);
        let loop_icon = Image::from_encoded(loop_icon_data).expect("could not decode loop icon");

        Self {
            waveform: DrawCache::new(Self::draw_waveform),
            beats: DrawCache::new(Self::draw_beats),
            loop_icon,
        }
    }

    fn time_to_pixels(&self, time: FrameTime) -> f64 {
        time.0 as f64 / SAMPLES_PER_PIXEL as f64
    }

    fn time_to_x(&self, time: FrameTime) -> f64 {
        let t_in_pixels = self.time_to_pixels(time);
        t_in_pixels - waveform_zero_offset() as f64
    }

    fn channel_transform(t: usize, d_t: f32, len: usize) -> (f32, f32) {
        let v = (d_t * 3.0).abs().min(1.0);

        let x = (t as f32) / len as f32;
        let y = v;

        (x, y)
    }

    fn path_for_waveform(waveform: [&[f32]; 2], w: f32, h: f32) -> Path {
        let mut p = Path::new();
        p.move_to(Point::new(0.0, h / 2.0));

        let len = waveform[0].len();
        for (x, y) in waveform[0]
            .iter()
            .enumerate()
            .map(|(t, d_t)| Self::channel_transform(t, *d_t, len))
        {
            p.line_to(Point::new(x * w, (-y + 1.0) / 2.0 * h));
        }

        for (x, y) in waveform[1]
            .iter()
            .enumerate()
            .rev()
            .map(|(t, d_t)| Self::channel_transform(t, *d_t, len))
        {
            p.line_to(Point::new(x * w, (y + 1.0) / 2.0 * h));
        }

        p.close();

        p
    }

    fn draw_waveform(
        data: &AppData,
        looper: &LooperData,
        w: f32,
        h: f32,
        _: f32,
        canvas: &mut Canvas,
    ) -> Size {
        let p = Self::path_for_waveform([&looper.waveform[0], &looper.waveform[1]], w, h);

        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color(color_for_mode(looper.mode_with_solo(data)));
        paint.set_style(Style::Fill);
        canvas.draw_path(&p, &paint);

        // this actually isn't right probably?
        Size::new(w, h)
    }

    fn draw_beats(
        data: &AppData,
        _: &LooperData,
        w: f32,
        h: f32,
        scale: f32,
        canvas: &mut Canvas,
    ) -> Size {
        let mut beat_p = Path::new();
        let mut bar_p = Path::new();

        let ms = data.engine_state.metric_structure;

        let samples_per_beat = FrameTime(ms.tempo.samples_per_beat() as i64);

        let beat_width = samples_per_beat.0 as f32 / SAMPLES_PER_PIXEL;
        let number_of_beats = (w / beat_width).ceil() as usize;

        // make sure we get a full number of measures
        let number_of_beats = (number_of_beats / ms.time_signature.upper as usize + 1)
            * ms.time_signature.upper as usize;

        let mut x = 0.0;
        for i in 0..number_of_beats as i64 {
            if i % ms.time_signature.upper as i64 == 0 {
                bar_p.move_to(Point::new(x, 5.0));
                bar_p.line_to(Point::new(x, h - 5.0));
            } else {
                beat_p.move_to(Point::new(x, 20.0));
                beat_p.line_to(Point::new(x, h - 20.0));
            }

            x += beat_width;
        }

        let mut beat_paint = Paint::default();
        beat_paint
            .set_color(Color::from_argb(170, 200, 200, 255))
            .set_anti_alias(true)
            .set_stroke_width(1.0 * scale)
            .set_style(Style::Stroke)
            .set_blend_mode(BlendMode::Lighten);

        let mut bar_paint = Paint::default();
        bar_paint
            .set_color(Color::from_argb(255, 255, 255, 255))
            .set_anti_alias(true)
            .set_stroke_width(3.0 * scale)
            .set_style(Style::Stroke);
        let mut bar_outer_paint = bar_paint.clone();
        bar_outer_paint.set_color(Color::from_argb(130, 0, 0, 0));
        bar_outer_paint.set_stroke_width(4.0);

        canvas.draw_path(&beat_p, &beat_paint);
        canvas.draw_path(&bar_p, &bar_outer_paint);
        canvas.draw_path(&bar_p, &bar_paint);

        Size::new(x, h)
    }

    fn draw(
        &mut self,
        canvas: &mut Canvas,
        data: &AppData,
        looper: &LooperData,
        w: f32,
        h: f32,
    ) -> Size {
        let full_w = looper.length as f64 / SAMPLES_PER_PIXEL as f64;

        canvas.save();

        let mut loop_icons = vec![];

        canvas.clip_rect(
            Rect::new(0.0, 0.0, w, h),
            Some(ClipOp::Intersect),
            Some(false),
        );

        // draw waveform
        if looper.length > 0 {
            if looper.mode == LooperMode::Recording {
                let pre_width = FrameTime((waveform_zero_offset() * SAMPLES_PER_PIXEL) as i64)
                    .to_waveform() as f32;
                // we're only going to render the part of the waveform that's in the past
                let len = (pre_width as usize).min(looper.waveform[0].len());
                let start = looper.waveform[0].len() - len;

                let width = (len as f32 / pre_width) * waveform_zero_offset();

                canvas.save();
                canvas.translate((waveform_zero_offset() - width, 0.0));
                let path = Self::path_for_waveform(
                    [&looper.waveform[0][start..], &looper.waveform[1][start..]],
                    width,
                    h,
                );
                let mut paint = Paint::default();
                paint.set_anti_alias(true);
                paint.set_color(dark_color_for_mode(LooperMode::Recording));
                canvas.draw_path(&path, &paint);
                canvas.restore();
            } else {
                let mut time = data.engine_state.time.0 - looper.offset.0;

                if time < 0 {
                    time = time.rem_euclid(looper.length as i64);
                }

                let first_loop_iteration = data.engine_state.time.0 < looper.length as i64;

                let start_time = if time < looper.length as i64 {
                    0
                } else {
                    // The second smallest multiple of length < time
                    ((time / looper.length as i64) - 1) * (looper.length as i64)
                };

                let mut x = -self.time_to_x(FrameTime(time - start_time));

                let mut first = true;

                canvas.save();
                let clip_x = -self.time_to_x(data.engine_state.time) as f32;
                if clip_x > 0.0 {
                    canvas.clip_rect(
                        Rect::new(clip_x, 0.0, w, h),
                        Some(ClipOp::Intersect),
                        Some(false),
                    );
                }

                while x < w as f64 * 2.0 {
                    canvas.save();
                    canvas.translate(Vector::new(x as f32, 0.0));

                    if (!first_loop_iteration || !first) && clip_x < x as f32 {
                        loop_icons.push(x);
                    }

                    self.waveform.draw(
                        (looper.length, looper.last_time, looper.mode_with_solo(data)),
                        data,
                        looper,
                        full_w as f32,
                        h,
                        looper.mode != LooperMode::Recording
                            && looper.mode != LooperMode::Overdubbing,
                        canvas,
                    );

                    canvas.restore();
                    x += full_w;
                    first = false;
                }

                canvas.restore();
            }
        }

        canvas.clip_rect(
            Rect::new(0.0, 0.0, w, h),
            Some(ClipOp::Intersect),
            Some(false),
        );

        // draw bar and beat lines
        {
            canvas.save();
            // draw the first at the previous measure start before time
            let ms = data.engine_state.metric_structure;
            let next_beat = ms.tempo.next_full_beat(data.engine_state.time);
            let mut beat_of_measure = ms.time_signature.beat_of_measure(ms.tempo.beat(next_beat));
            if beat_of_measure == 0 {
                beat_of_measure = ms.time_signature.upper;
            }

            // we need to make sure that we go back far enough that the start is off of the screen
            // so we just subtract measures until we are
            // there's an analytical solution to this but I'm too lazy to figure it out right now
            let mut start_time =
                next_beat - FrameTime(beat_of_measure as i64 * ms.tempo.samples_per_beat() as i64);
            let mut x = -self.time_to_x(data.engine_state.time - start_time);
            while x > 0.0 {
                start_time = start_time
                    - FrameTime(
                        ms.time_signature.upper as i64 * ms.tempo.samples_per_beat() as i64,
                    );
                x = -self.time_to_x(data.engine_state.time - start_time);
            }

            canvas.translate((x as f32, 0.0));
            let size = self.beats.draw(
                ms, data, looper, w, h,
                // TODO: turning on the cache currently causes rendering issues
                false, canvas,
            );
            canvas.translate((size.width, 0.0));
            self.beats.draw(ms, data, looper, w, h, false, canvas);
            canvas.restore();
        }

        // draw loop icons
        for x in loop_icons {
            canvas.save();
            canvas.translate((x as f32, 0.0));
            let s = 48.0;
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            canvas.draw_image_rect_with_sampling_options(
                &self.loop_icon,
                None,
                Rect::new(-s / 2.0, (h - s) / 2.0, s / 2.0, (h + s) / 2.0),
                SamplingOptions::from_filter_quality(FilterQuality::High, None),
                &paint,
            );

            canvas.restore();
        }

        // draw trigger if present and in future
        if let Some((time, lc)) = looper.trigger {
            if time > data.engine_state.time {
                let mut paint = Paint::default();
                paint.set_anti_alias(true);

                let mut text = None;
                match lc {
                    LooperCommand::Record => {
                        paint.set_color(color_for_mode(LooperMode::Recording));
                        text = Some("recording");
                    }
                    LooperCommand::Overdub => {
                        paint.set_color(color_for_mode(LooperMode::Overdubbing));
                        text = Some("overdubbing");
                    }
                    LooperCommand::Play => {
                        paint.set_color(color_for_mode(LooperMode::Playing));
                        text = Some("playing");
                    }
                    LooperCommand::Mute => {
                        paint.set_color(color_for_mode(LooperMode::Muted));
                        text = Some("muting");
                    }
                    LooperCommand::Solo => {
                        paint.set_color(color_for_mode(LooperMode::Soloed));
                        text = Some("soloing");
                    }
                    LooperCommand::RecordOverdubPlay => {
                        if looper.length == 0 {
                            paint.set_color(color_for_mode(LooperMode::Recording));
                            text = Some("recording");
                        } else if looper.mode == LooperMode::Recording
                            || looper.mode == LooperMode::Playing
                        {
                            paint.set_color(color_for_mode(LooperMode::Overdubbing));
                            text = Some("overdubbing");
                        } else {
                            paint.set_color(color_for_mode(LooperMode::Playing));
                            text = Some("playing");
                        }
                    }
                    _ => {}
                }

                paint.set_alpha_f(0.9);

                let x = -self.time_to_x(data.engine_state.time - time) as f32;
                let rect = Rect::new(x, 15.0, w, h - 15.0);
                canvas.draw_rect(&rect, &paint);

                if let Some(text) = text {
                    let font = Font::new(Typeface::default(), 24.0);
                    let mut text_paint = Paint::default();
                    text_paint.set_color(Color::BLACK);
                    text_paint.set_anti_alias(true);

                    let time_blob = TextBlob::new(&text, &font).unwrap();
                    canvas.draw_text_blob(
                        &time_blob,
                        Point::new(x + 10.0, h / 2.0 + 6.0),
                        &text_paint,
                    );
                }
            }
        }

        canvas.restore();

        Size::new(w, h)
    }
}
