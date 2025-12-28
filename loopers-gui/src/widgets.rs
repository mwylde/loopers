use crate::{AppData, Controller, GuiEvent, KeyEventKey, KeyEventType, MouseEventType};
use loopers_common::clamp;
use sdl2::mouse::MouseButton;
use skia_safe::paint::Style;
use skia_safe::{Canvas, Color, Contains, Font, Paint, PathBuilder, Point, Rect, Size, TextBlob};
use std::f32::consts::PI;
use std::time::UNIX_EPOCH;

pub fn draw_circle_indicator(canvas: &Canvas, color: Color, p: f32, x: f32, y: f32, r: f32) {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color(color);
    paint.set_alpha_f(0.3);
    canvas.draw_circle(Point::new(x + r, y + r), r, &paint);

    paint.set_alpha_f(1.0);

    let mut path = PathBuilder::new();
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
    let path = path.snapshot();

    paint.set_stroke_width(2.0);
    paint.set_style(Style::StrokeAndFill);
    canvas.draw_path(&path, &paint);
}

pub trait Button {
    fn set_state(&mut self, state: ButtonState);
    fn get_state(&self) -> ButtonState;

    fn handle_event<F: FnOnce(MouseButton)>(
        &mut self,
        canvas: &Canvas,
        bounds: &Rect,
        on_click: F,
        event: Option<GuiEvent>,
    ) {
        if let Some(GuiEvent::MouseEvent(typ, pos)) = event {
            let point = canvas
                .local_to_device_as_3x3()
                .invert()
                .unwrap()
                .map_point((pos.0 as f32, pos.1 as f32));
            if bounds.contains(point) {
                match typ {
                    MouseEventType::MouseDown(MouseButton::Left) => {
                        self.set_state(ButtonState::Pressed);
                    }
                    MouseEventType::MouseUp(button) => {
                        if self.get_state() == ButtonState::Pressed {
                            on_click(button);
                        }
                        self.set_state(ButtonState::Hover);
                    }
                    MouseEventType::Moved => {
                        self.set_state(ButtonState::Hover);
                    }
                    _ => {}
                }
            } else {
                self.set_state(ButtonState::Default);
            }
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ButtonState {
    Default,
    Hover,
    Pressed,
}

pub struct ControlButton {
    state: ButtonState,
    text: TextBlob,
    text_size: Size,
    color: Color,
    width: f32,
    height: f32,
}

impl ControlButton {
    pub fn new(text: &str, color: Color, width: Option<f32>, height: f32) -> Self {
        let font = crate::default_font(16.0);

        let text_size = font.measure_str(text, None).1.size();

        let text = TextBlob::new(text, &font).unwrap();

        ControlButton {
            state: ButtonState::Default,
            text,
            text_size,
            color,
            width: width.unwrap_or(text_size.width + 24.0),
            height,
        }
    }

    pub fn draw<F: FnOnce(MouseButton)>(
        &mut self,
        canvas: &Canvas,
        is_active: bool,
        disabled: bool,
        on_click: F,
        last_event: Option<GuiEvent>,
    ) -> Size {
        self.draw_with_progress(canvas, is_active, disabled, on_click, last_event, 0.0)
    }

    pub fn draw_with_progress<F: FnOnce(MouseButton)>(
        &mut self,
        canvas: &Canvas,
        is_active: bool,
        disabled: bool,
        on_click: F,
        last_event: Option<GuiEvent>,
        progress_percent: f32,
    ) -> Size {
        let bounds = Rect::new(0.0, 0.0, self.width, self.height);

        if !disabled {
            self.handle_event(canvas, &bounds, on_click, last_event);
        }

        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_style(Style::Stroke);

        paint.set_color(match self.state {
            ButtonState::Default => self.color,
            ButtonState::Hover => Color::from_rgb(130, 130, 130),
            ButtonState::Pressed => Color::from_rgb(255, 255, 255),
        });

        if disabled {
            paint.set_alpha_f(0.3);
        }

        paint.set_stroke_width(2.0);

        paint.set_style(if is_active && self.state != ButtonState::Pressed {
            Style::StrokeAndFill
        } else {
            Style::Stroke
        });

        canvas.draw_rect(bounds, &paint);

        if progress_percent > 0.0 {
            let progress = Rect {
                left: bounds.left,
                top: bounds.bottom - progress_percent * bounds.height(),
                right: bounds.right,
                bottom: bounds.bottom,
            };

            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_stroke_width(2.0);
            paint.set_color(self.color);
            paint.set_style(Style::Fill);
            canvas.draw_rect(progress, &paint);
        }

        let mut text_paint = Paint::default();
        text_paint.set_anti_alias(true);
        text_paint.set_color(Color::WHITE);

        if disabled {
            text_paint.set_alpha_f(0.3);
        }

        let x = self.width * 0.5 - self.text_size.width * 0.5;
        let y = self.height * 0.5 + self.text_size.height * 0.5;

        canvas.draw_text_blob(&self.text, (x, y), &text_paint);

        bounds.size()
    }
}

impl Button for ControlButton {
    fn set_state(&mut self, state: ButtonState) {
        self.state = state;
    }

    fn get_state(&self) -> ButtonState {
        self.state
    }
}

#[allow(dead_code)]
pub trait Modal {
    #[allow(clippy::too_many_arguments)]
    fn draw(
        &mut self,
        manager: &mut ModalManager,
        canvas: &Canvas,
        w: f32,
        h: f32,
        data: &AppData,
        sender: &mut Controller,
        last_event: Option<GuiEvent>,
    ) -> Size;
}

#[allow(dead_code)]
pub struct ModalManager {
    current: Option<Box<dyn Modal>>,
}

impl ModalManager {
    pub fn new() -> Self {
        ModalManager { current: None }
    }

    #[allow(dead_code)]
    pub fn set(&mut self, modal: Box<dyn Modal>) {
        self.current = Some(modal);
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.current = None;
    }

    pub fn draw(
        &mut self,
        canvas: &Canvas,
        w: f32,
        h: f32,
        data: &AppData,
        controller: &mut Controller,
        last_event: Option<GuiEvent>,
    ) {
        let mut cur = self.current.take();
        if let Some(modal) = &mut cur {
            modal.draw(self, canvas, w, h, data, controller, last_event);
        }

        self.current = cur;
    }
}

#[derive(Clone, Eq, PartialEq)]
pub enum TextEditState {
    Default,
    Editing(bool, String),
}

pub trait TextEditable {
    fn commit(&mut self, controller: &mut Controller);

    fn start_editing(&mut self, initial: String) {
        *self.get_edit_state() = TextEditState::Editing(true, initial)
    }

    fn get_edit_state(&mut self) -> &mut TextEditState;

    fn is_valid(_s: &str) -> bool {
        true
    }

    fn draw_edit(
        &mut self,
        canvas: &Canvas,
        font: &Font,
        bounds: &Rect,
        sender: &mut Controller,
        last_event: Option<GuiEvent>,
    ) {
        let mut commit = false;
        // if there was a click elsewhere, clear our state
        if let Some(GuiEvent::MouseEvent(MouseEventType::MouseDown(MouseButton::Left), pos)) =
            last_event
        {
            let point = canvas
                .local_to_device_as_3x3()
                .invert()
                .unwrap()
                .map_point((pos.0 as f32, pos.1 as f32));

            if !bounds.contains(point) {
                commit = true;
            }
        }

        if let TextEditState::Editing(selected, edited) = self.get_edit_state() {
            if let Some(GuiEvent::KeyEvent(KeyEventType::Pressed, key)) = last_event {
                match key {
                    KeyEventKey::Char(c) => {
                        if *selected {
                            edited.clear();
                        }

                        edited.push(c);
                        if !Self::is_valid(edited) {
                            edited.pop();
                        }

                        *selected = false;
                    }
                    KeyEventKey::Backspace => {
                        if *selected {
                            edited.clear();
                        } else {
                            edited.pop();
                        }
                    }
                    KeyEventKey::Enter | KeyEventKey::Esc => {
                        commit = true;
                    }
                }
            }

            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_color(Color::WHITE);
            canvas.draw_round_rect(bounds, 4.0, 4.0, &paint);

            let mut text_paint = Paint::default();
            text_paint.set_color(Color::WHITE);
            text_paint.set_anti_alias(true);

            let text_bounds = font
                .measure_str(&edited, Some(&text_paint))
                .1
                .with_offset((15.0, 18.0))
                .with_outset((3.0, 3.0));

            if *selected {
                if !edited.is_empty() {
                    paint.set_color(Color::BLUE);
                    canvas.draw_rect(text_bounds, &paint);
                }
            } else {
                text_paint.set_color(Color::BLACK);
                let mut cursor = PathBuilder::new();
                let x = if edited.is_empty() {
                    20.0
                } else {
                    text_bounds.right + 3.0
                };

                cursor.move_to((x, 2.0));
                cursor.line_to((x, 20.0));
                let cursor = cursor.snapshot();
                let mut paint = Paint::default();
                paint.set_color(Color::BLACK);
                paint.set_style(Style::Stroke);
                paint.set_stroke_width(1.0);
                paint.set_anti_alias(true);

                if UNIX_EPOCH.elapsed().unwrap().as_millis() % 1500 > 500 {
                    canvas.draw_path(&cursor, &paint);
                }
            }

            canvas.draw_str(edited, Point::new(15.0, 18.0), font, &text_paint);
        }

        if commit {
            self.commit(sender);
        }
    }
}

pub struct PotWidget {
    size: f32,
    color: Color,
    last_mouse_value: Option<f32>,
}

impl PotWidget {
    pub fn new(size: f32, color: Color) -> Self {
        Self {
            size,
            color,
            last_mouse_value: None,
        }
    }

    // level is [-1, 1], with 0 centered
    pub fn draw<F: FnOnce(f32)>(
        &mut self,
        canvas: &Canvas,
        level: f32,
        new_level: F,
        last_event: Option<GuiEvent>,
    ) {
        let mut paint = Paint::default();
        paint.set_color(self.color);
        paint.set_anti_alias(true);
        paint.set_stroke_width(2.0);
        paint.set_style(Style::Stroke);

        let offset_angle = 20.0;

        let mut path = PathBuilder::new();
        path.arc_to(
            Rect::from_wh(self.size, self.size),
            180.0 - offset_angle,
            180.0 + offset_angle * 2.0,
            true,
        );
        let path = path.snapshot();
        canvas.draw_path(&path, &paint);

        let mut path = PathBuilder::new();
        let r = self.size / 2.0;
        let c = r;

        let offset_angle_rad = offset_angle * PI / 180.0;
        let angle = level * (PI / 2.0 + offset_angle_rad) - PI / 2.0;

        path.move_to((c, c));
        path.line_to((c + (r + 3.0) * angle.cos(), c + (r + 3.0) * angle.sin()));
        let path = path.snapshot();

        let mut bg_paint = Paint::default();
        bg_paint.set_anti_alias(true);
        bg_paint.set_stroke_width(7.0);
        bg_paint.set_style(Style::Stroke);
        bg_paint.set_color(*crate::skia::BACKGROUND_COLOR);

        canvas.draw_path(&path, &bg_paint);
        canvas.draw_path(&path, &paint);

        let bounds = Rect::from_size((self.size, self.size));

        if let Some(GuiEvent::MouseEvent(MouseEventType::MouseDown(MouseButton::Left), (x, y))) =
            last_event
        {
            let point = canvas
                .local_to_device_as_3x3()
                .invert()
                .unwrap()
                .map_point((x as f32, y as f32));

            if bounds.contains(point) {
                self.last_mouse_value = Some(y as f32);
            }
        }

        if let Some(GuiEvent::MouseEvent(MouseEventType::Moved, (_, y))) = last_event
            && let Some(p_y) = self.last_mouse_value
        {
            let lv = clamp(level + (p_y - y as f32) / 100.0, -1.0, 1.0);
            new_level(lv);
            self.last_mouse_value = Some(y as f32);
        }

        if let Some(GuiEvent::MouseEvent(MouseEventType::MouseUp(_), _)) = last_event {
            self.last_mouse_value = None;
        }
    }
}
