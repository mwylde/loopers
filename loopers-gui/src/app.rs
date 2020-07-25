use skia_safe::*;

use crate::{AppData, LooperData, GuiEvent, MouseEventType};

use crate::skia::{HEIGHT, WIDTH};
use skia_safe::paint::Style;
use std::time::Duration;
use loopers_common::music::{FrameTime, MetricStructure};
use std::collections::{BTreeMap};
use loopers_common::protos::{LooperMode, Command, GlobalCommand, GlobalCommandType};
use skia_safe::gpu::SurfaceOrigin;
use winit::event::MouseButton;
use crossbeam_channel::Sender;
use loopers_common::protos::command::CommandOneof;


fn color_for_mode(mode: LooperMode) -> Color {
    match mode {
        LooperMode::Record => Color::from_rgb(255, 0, 0),
        LooperMode::Ready => Color::from_rgb(255, 50, 0), // TODO: fixme
        LooperMode::Overdub => Color::from_rgb(0, 255, 255),
        LooperMode::Playing => Color::from_rgb(0, 255, 0),
        LooperMode::None => Color::from_rgb(135, 135, 135),
    }
}

fn dark_color_for_mode(mode: LooperMode) -> Color {
    match mode {
        LooperMode::Record => Color::from_rgb(210, 45, 45),
        LooperMode::Ready => Color::from_rgb(150, 30, 255), // TODO: fixme
        LooperMode::Overdub => Color::from_rgb(0, 255, 255),
        LooperMode::Playing => Color::from_rgb(0, 213, 0),
        LooperMode::None => Color::from_rgb(65, 65, 65),
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

struct Animation {
    start_time: FrameTime,
    length: Duration,
    function: AnimationFunction,
}

impl Animation {
    fn new(start_time: FrameTime, length: Duration, function: AnimationFunction) -> Self {
        Animation {
            start_time,
            length,
            function,
        }
    }

    fn value(&self, time: FrameTime) -> f32 {
        let p = (time.to_ms() - self.start_time.to_ms()) as f32 / self.length.as_millis() as f32;
        self.function.value(p)
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum ButtonState {
    Default,
    Hover,
    Pressed,
}

trait Button {
    fn clicked(&mut self, button: MouseButton, sender: &mut Sender<Command>);

    fn set_state(&mut self, state: ButtonState);

    fn handle_event(&mut self, canvas: &Canvas, bounds: &Rect,
                    sender: &mut Sender<Command>, event: Option<GuiEvent>) {
        if let Some(event) = event {
            match event {
                GuiEvent::MouseEvent(typ, pos) => {
                    let point = canvas.total_matrix().invert().unwrap().map_point((pos.x as f32, pos.y as f32));
                    if bounds.contains(point) {
                        match typ {
                            MouseEventType::MouseDown(MouseButton::Left) => {
                                self.set_state(ButtonState::Pressed);
                            },
                            MouseEventType::MouseUp(button) => {
                                self.clicked(button, sender);
                                self.set_state(ButtonState::Default);
                            },
                            MouseEventType::Moved => {
                                self.set_state(ButtonState::Hover);
                            },
                            _ => {}
                        }
                    } else {
                        self.set_state(ButtonState::Default);
                    }
                },
            }
        }

    }
}

pub struct MainPage {
    loopers: BTreeMap<u32, LooperView>,
    beat_animation: Option<Animation>,
    bottom_bar: BottomBarView,
    add_button: AddButton,
}

const LOOPER_MARGIN: f32 = 10.0;
const LOOPER_HEIGHT: f32 = 80.0;
const WAVEFORM_OFFSET_X: f32 = 100.0;
const WAVEFORM_WIDTH: f32 = 650.0;
const WAVEFORM_ZERO_RATIO: f32 = 0.25;

struct AddButton {
    state: ButtonState,
}

impl AddButton {
    fn new() -> Self {
        AddButton {
            state: ButtonState::Default,
        }
    }

    fn draw(&mut self, canvas: &mut Canvas, data: &AppData, sender: &mut Sender<Command>, last_event: Option<GuiEvent>) {
        canvas.save();
        canvas.translate((35.0, (LOOPER_HEIGHT + LOOPER_MARGIN) * data.loopers.len() as f32 + 50.0));

        let mut p = Path::new();
        p.move_to((0.0, 15.0));
        p.line_to((30.0, 15.0));
        p.move_to((15.0, 0.0));
        p.line_to((15.0, 30.0));

        self.handle_event(canvas, p.bounds(), sender, last_event);

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
        canvas.restore();
    }
}

impl Button for AddButton {
    fn clicked(&mut self, button: MouseButton, sender: &mut Sender<Command>) {
        if button == MouseButton::Left {
            sender.send(Command {
                command_oneof: Some(CommandOneof::GlobalCommand(GlobalCommand {
                    command: GlobalCommandType::AddLooper as i32,
                }))
            }).unwrap();
        }
    }

    fn set_state(&mut self, state: ButtonState) {
        self.state = state;
    }
}

impl MainPage {
    pub fn new() -> Self {
        MainPage {
            loopers: BTreeMap::new(),
            beat_animation: None,
            bottom_bar: BottomBarView::new(),
            add_button: AddButton::new(),
        }
    }

    pub fn draw(&mut self, canvas: &mut Canvas, data: &AppData, sender: &mut Sender<Command>,
                last_event: Option<GuiEvent>) {
        // add views for new loopers
        for id in data.loopers.keys() {
            self.loopers.entry(*id)
                .or_insert_with(|| LooperView::new(*id));
        }

        // remove deleted loopers
        let remove: Vec<u32> = self.loopers.keys()
            .filter(|id| !data.loopers.contains_key(id))
            .map(|id| *id)
            .collect();

        for id in remove {
            self.loopers.remove(&id);
        }

        for (i, (id, looper)) in self.loopers.iter_mut().enumerate() {
            canvas.save();
            canvas.translate(Vector::new(0.0, i as f32 * (LOOPER_HEIGHT + LOOPER_MARGIN)));

            looper.draw(canvas, data, &data.loopers[id]);

            canvas.restore();
        }

        // draw play head
        let x = WAVEFORM_WIDTH * WAVEFORM_ZERO_RATIO;
        let h = self.loopers.len() as f32 * (LOOPER_HEIGHT + LOOPER_MARGIN);

        canvas.save();
        canvas.translate(Vector::new(WAVEFORM_OFFSET_X, 0.0));
        let mut path = Path::new();
        {
            path.move_to(Point::new(x - 5.0, 10.0));
            path.line_to(Point::new(x + 5.0, 10.0));
            path.move_to(Point::new(x, 10.0));
            path.line_to(Point::new(x, h));
            path.move_to(Point::new(x - 5.0, h));
            path.line_to(Point::new(x + 5.0, h));
        }
        let mut paint = Paint::default();
        paint.set_anti_alias(true);

        // draw overlay to darken time that is past
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color(Color::from_argb(120, 0, 0, 0));
        canvas.draw_rect(Rect::new(0.0, 10.0, x, h), &paint);

        // draw play head bar
        let beat = data.engine_state.metric_structure.tempo.beat(data.engine_state.time);
        let bom = data.engine_state.metric_structure.time_signature.beat_of_measure(beat);

        if bom == 0 {
            if self.beat_animation.is_none() {
                self.beat_animation = Some(Animation::new(
                    data.engine_state.time,
                    Duration::from_millis(500),
                    AnimationFunction::EaseOutCubic,
                ));
            }

            let v = self.beat_animation.as_ref().unwrap().value(data.engine_state.time);
            paint.set_stroke_width(3.0 + ((1.0 - v) * 5.0));
        } else {
            self.beat_animation = None;
            paint.set_stroke_width(3.0);
        }
        paint.set_color(Color::from_rgb(255, 255, 255));
        paint.set_style(Style::Stroke);

        canvas.draw_path(&path, &paint);
        canvas.restore();

        // draw the looper add button if we have fewer than 5 loopers
        if self.loopers.len() < 5 {
            self.add_button.draw(canvas, data, sender, last_event);
        }

        // draw the bottom bar
        canvas.save();
        let bar_height = 30.0;
        canvas.translate(Vector::new(0.0, HEIGHT as f32 - bar_height));
        self.bottom_bar.draw(canvas, WIDTH as f32, 30.0, data);
        canvas.restore();
    }
}

struct BottomBarView {}

impl BottomBarView {
    fn new() -> Self {
        Self {}
    }

    fn draw(&mut self, canvas: &mut Canvas, _w: f32, h: f32, data: &AppData) {
        let font = Font::new(Typeface::default(), 20.0);

        // let mut background = Paint::default();
        // background.set_color(Color::from_argb(100, 255, 255, 255));
        // canvas.draw_rect(Rect::new(0.0, 0.0, w, h), &background);

        let mut text_paint = Paint::default();
        text_paint.set_color(Color::WHITE);
        text_paint.set_anti_alias(true);
        canvas.draw_str(
            &format!("{} bpm", data.engine_state.metric_structure.tempo.bpm() as u32),
            Point::new(10.0, h - 12.0),
            &font,
            &text_paint,
        );

        let mut x = 130.0;

        let current_beat = data.engine_state.metric_structure.tempo.beat(data.engine_state.time);
        let beat_of_measure = data.engine_state.metric_structure.time_signature.beat_of_measure(current_beat);
        let measure = data.engine_state.metric_structure.time_signature.measure(current_beat);
        for beat in 0..data.engine_state.metric_structure.time_signature.upper {
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            if beat == beat_of_measure {
                paint.set_color(Color::from_rgb(0, 255, 0));
            } else {
                paint.set_color(Color::from_rgb(128, 128, 128));
            }

            let radius = 10.0;
            canvas.draw_circle(Point::new(x, h / 2.0 - 5.0), radius, &paint);
            x += 30.0;
        }

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

        let time_blob = TextBlob::new(
            &format!("{}{:02}:{:02}:{:02}", negative, hours, minutes, seconds),
            &font,
        )
        .unwrap();

        canvas.draw_text_blob(&time_blob, Point::new(x, h - 12.0), &text_paint);

        // TODO: should probably figure out what this bounds actually represents, since it does
        //       not seem to be a bounding box of the text as I would expect
        x += time_blob.bounds().width() - 30.0;

        let measure_blob =
            TextBlob::new(format!("{:03}.{}", measure, beat_of_measure), &font).unwrap();

        canvas.draw_text_blob(&measure_blob, Point::new(x, h - 12.0), &text_paint);
    }
}

struct LooperView {
    id: u32,
    waveform_view: WaveformView,
}

impl LooperView {
    fn new(id: u32) -> Self {
        Self {
            id,
            waveform_view: WaveformView::new(),
        }
    }

    fn draw(&mut self, canvas: &mut Canvas, data: &AppData, looper: &LooperData) {
        assert_eq!(self.id, looper.id);

        let ratio = if looper.length == 0 || looper.state == LooperMode::Record {
            0f32
        } else {
            (data.engine_state.time.0 % looper.length as i64) as f32 / looper.length as f32
        };

        draw_circle_indicator(canvas, color_for_mode(looper.state), ratio, 25.0, 25.0, 25.0);

        canvas.save();
        canvas.translate(Vector::new(WAVEFORM_OFFSET_X, 10.0));
        self.waveform_view
            .draw(canvas, data, looper, WAVEFORM_WIDTH, LOOPER_HEIGHT);

        canvas.restore();
    }
}

const IMAGE_SCALE: f32 = 4.0;

type CacheUpdaterFn = fn(data: &AppData, looper: &LooperData, time_width: FrameTime, w: f32, h: f32, canvas: &mut Canvas);

struct DrawCache<T: Eq + Copy> {
    image: Option<Image>,
    key: Option<T>,
    draw_fn: CacheUpdaterFn,
}

impl <T: Eq + Copy> DrawCache<T> {
    fn new(draw_fn: CacheUpdaterFn) -> DrawCache<T> {
        DrawCache {
            image: None,
            key: None,
            draw_fn,
        }
    }

    fn draw(&mut self, key: T, data: &AppData, looper: &LooperData, time_width: FrameTime,
            w: f32, h: f32, use_cache: bool, canvas: &mut Canvas) {
        if !use_cache {
            (self.draw_fn)(data, looper, time_width, w, h, canvas);
            return;
        }

        let size = ((w * IMAGE_SCALE) as i32, (h * IMAGE_SCALE) as i32);

        if self.key.is_none() || self.key.unwrap() != key ||
            self.image.is_none() || self.image.as_ref().map(|i| (i.width(), i.height())).unwrap() != size {

            let image_info = ImageInfo::new_n32(size, AlphaType::Premul, None);
            let mut surface =
                Surface::new_render_target(canvas.gpu_context().as_mut().unwrap(),
                                           Budgeted::Yes, &image_info,
                                           None, SurfaceOrigin::TopLeft,
                                           None, None).unwrap();

            (self.draw_fn)(data, looper, time_width, w * IMAGE_SCALE, h * IMAGE_SCALE, &mut surface.canvas());

            let image = surface.image_snapshot();
            self.image = Some(image);
            self.key = Some(key);
        }

        if let Some(image) = self.image.as_ref() {
            canvas.save();
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            canvas.scale((1.0/IMAGE_SCALE, 1.0/IMAGE_SCALE));
            canvas.draw_image(image, (0.0, 0.0), Some(&paint));
            canvas.restore();
        }
    }
}

struct WaveformView {
    waveform: DrawCache<(u64, FrameTime, LooperMode)>,
    beats: DrawCache<MetricStructure>,
    time_width: FrameTime,
}

impl WaveformView {
    fn new() -> Self {
        Self {
            waveform: DrawCache::new(Self::draw_waveform),
            beats: DrawCache::new(Self::draw_beats),
            time_width: FrameTime::from_ms(12_000.0),
        }
    }

    fn time_to_x(&self, time: FrameTime, w: f32) -> f32 {
        // offset time back so that time 0 is at the play head
        let offset_time = time.0 as f32 - (self.time_width.0 as f32 * WAVEFORM_ZERO_RATIO);
        1.0 / (self.time_width.0 as f32 / w) * offset_time
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
        for (x, y) in waveform[0].iter().enumerate()
            .map(|(t, d_t)|  Self::channel_transform(t, *d_t, len)) {
            p.line_to(Point::new(x * w, (-y + 1.0) / 2.0 * h));
        }

        for (x, y) in waveform[1].iter().enumerate().rev()
            .map(|(t, d_t)|  Self::channel_transform(t, *d_t, len)) {
            p.line_to(Point::new(x * w, (y + 1.0) / 2.0 * h));
        }

        p.close();

        p
    }

    fn draw_waveform(_: &AppData, looper: &LooperData, _: FrameTime, w: f32, h: f32, canvas: &mut Canvas) {
        let p = Self::path_for_waveform([&looper.waveform[0], &looper.waveform[1]], w, h);

        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color(dark_color_for_mode(looper.state));
        paint.set_style(Style::Fill);
        canvas.draw_path(&p, &paint);

        // paint.set_color(Color::from_argb(150, 255, 255, 255));
        // paint.set_stroke_width(2.0);
        // paint.set_style(Style::Stroke);
        // paint.set_path_effect(PathEffect::discrete(5.0, 2.0, None));
        // let mut p = Path::new();
        // p.move_to((w - 3.0, 0.0));
        // p.line_to((w - 3.0, h));
        // canvas.draw_path(&p, &paint);
        //
        // paint.set_color(color_for_mode(looper.state));
        // paint.set_style(Style::Stroke);
        // paint.set_stroke_width(4.0);
        // canvas.draw_path(&p, &paint);
    }

    fn draw_beats(data: &AppData, _: &LooperData, time_width: FrameTime, w: f32, h: f32, canvas: &mut Canvas) {
        let mut beat_p = Path::new();
        let mut bar_p = Path::new();

        let samples_per_beat = FrameTime::from_ms(1000.0 /
            (data.engine_state.metric_structure.tempo.bpm() / 60.0) as f64);
        let number_of_beats = (time_width.0 as f32 / samples_per_beat.0 as f32).ceil() as usize;
        for i in 0..number_of_beats as i64 {
            let x = i as f32 * w / number_of_beats as f32;

            if i % data.engine_state.metric_structure.time_signature.upper as i64 == 0 {
                bar_p.move_to(Point::new(x, 5.0));
                bar_p.line_to(Point::new(x, h - 5.0));
            } else {
                beat_p.move_to(Point::new(x, 20.0));
                beat_p.line_to(Point::new(x, h - 20.0));
            }
        }

        let mut beat_paint = Paint::default();
        beat_paint
            .set_color(Color::from_argb(170, 200, 200, 255))
            .set_anti_alias(true)
            .set_stroke_width(1.0)
            .set_style(Style::Stroke)
            .set_blend_mode(BlendMode::Lighten);

        let mut bar_paint = Paint::default();
        bar_paint
            .set_color(Color::from_argb(255, 255, 255, 255))
            .set_anti_alias(true)
            .set_stroke_width(3.0)
            .set_style(Style::Stroke);
        let mut bar_outer_paint = bar_paint.clone();
        bar_outer_paint.set_color(Color::from_argb(130, 0, 0, 0));
        bar_outer_paint.set_stroke_width(4.0);

        canvas.draw_path(&beat_p, &beat_paint);
        canvas.draw_path(&bar_p, &bar_outer_paint);
        canvas.draw_path(&bar_p, &bar_paint);
    }

    fn draw(&mut self, canvas: &mut Canvas, data: &AppData, looper: &LooperData, w: f32, h: f32) {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color(Color::from_rgb(0, 65, 122));

        canvas.draw_rect(Rect::new(0.0, 0.0, w, h), &paint);

        let full_w = (looper.length as f32 / self.time_width.0 as f32) * w;

        canvas.save();

        canvas.clip_rect(
            Rect::new(0.0, 0.0, w, h),
            Some(ClipOp::Intersect),
            Some(false));

        // draw waveform
        if looper.length > 0 {
            if looper.state == LooperMode::Record {
                let pre_width = self.time_width.to_waveform() as f32 * WAVEFORM_ZERO_RATIO;
                // we're only going to render the part of the waveform that's in the past
                let len = (pre_width as usize).min(looper.waveform[0].len());
                let start = looper.waveform[0].len() - len;

                let width = (len as f32 / pre_width) * w * WAVEFORM_ZERO_RATIO;

                canvas.save();
                canvas.translate((w * WAVEFORM_ZERO_RATIO - width, 0.0));
                let path = Self::path_for_waveform(
                    [&looper.waveform[0][start..], &looper.waveform[1][start..]],
                    width, h);
                let mut paint = Paint::default();
                paint.set_anti_alias(true);
                paint.set_color(dark_color_for_mode(LooperMode::Record));
                canvas.draw_path(&path, &paint);
                canvas.restore();
            } else {
                let mut x = -self.time_to_x(data.engine_state.time, w)
                    .max(-full_w);
                while full_w > 0.0 && x < w * 2.0 {
                    if x + full_w > 0.0 && x < w {
                        canvas.save();
                        canvas.translate(Vector::new(x, 0.0));

                        self.waveform.draw((looper.length, looper.last_time, looper.state),
                                           data, looper, self.time_width, full_w, h,
                                           looper.state != LooperMode::Record &&
                                               looper.state != LooperMode::Overdub, canvas);

                        canvas.restore();
                    }

                    if looper.state == LooperMode::Record {
                        break;
                    }

                    x += full_w;
                }
            }
        }

        // draw bar and beat lines
        {
            let x = -self.time_to_x(data.engine_state.time, w)
                .rem_euclid(w);
            canvas.translate((x, 0.0));
            self.beats.draw(data.engine_state.metric_structure,
                            data, looper, self.time_width, w, h, false, canvas);
            canvas.translate((w, 0.0));
            self.beats.draw(data.engine_state.metric_structure,
                            data, looper, self.time_width, w, h, false, canvas);
        }


        canvas.restore();
    }
}

fn draw_circle_indicator(canvas: &mut Canvas, color: Color, p: f32, x: f32, y: f32, r: f32) {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color(color);
    paint.set_alpha_f(0.3);
    canvas.draw_circle(Point::new(x + r, y + r), r, &paint);

    paint.set_alpha_f(1.0);

    let mut path = Path::new();
    path.move_to(Point::new(x + r, y + r));
    path.line_to(Point::new(x + r, y));
    path.arc_to(
        Rect::new(x, y, x + 2.0 * r, y + 2.0 * r),
        270.0,
        270.0 + (p + 0.25) * 360.0,
        true,
    );
    path.line_to(Point::new(x + r, y + r));
    path.close();

    paint.set_stroke_width(2.0);
    paint.set_style(Style::StrokeAndFill);
    canvas.draw_path(&path, &paint);
}
