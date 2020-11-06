use skia_safe::gpu::gl::FramebufferInfo;
use skia_safe::gpu::{BackendRenderTarget, Context, SurfaceOrigin};
use skia_safe::{
    Color, ColorType, Font, Paint, PictureRecorder, Point, Rect, Surface, TextBlob, Typeface,
};
use std::convert::TryInto;

use crate::{Gui, GuiEvent, KeyEventKey, KeyEventType, MouseEventType};
use gl::types::*;
use gl_rs as gl;

use chrono::Local;
use std::fs::File;
use std::io::Write;
use std::time::Instant;
use sdl2::video::GLProfile;
use sdl2::pixels::PixelFormatEnum;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::{Keycode, Mod};

const INITIAL_WIDTH: i32 = 800;
const INITIAL_HEIGHT: i32 = 600;

lazy_static! {
    pub static ref BACKGROUND_COLOR: Color = Color::from_rgb(29, 30, 39);
}

fn create_surface(
    gr_context: &mut Context,
    pixel_format: &PixelFormatEnum,
    fb_info: FramebufferInfo,
    size: (u32, u32),
    scale_factor: f32,
) -> Surface {

    let backend_render_target = BackendRenderTarget::new_gl(
        (size.0 as i32, size.1 as i32),
        0,
        8,
        fb_info,
    );

    let color_type = match pixel_format {
        PixelFormatEnum::RGBA8888 => ColorType::RGBA8888,
        PixelFormatEnum::BGRA8888 => ColorType::BGRA8888,
        PixelFormatEnum::RGB888 => ColorType::RGBA8888,
        ct => {
            warn!("Unexpected color type {:?}", ct);
            ColorType::RGBA8888
        }
    };

    let mut surface = Surface::from_backend_render_target(
        gr_context,
        &backend_render_target,
        SurfaceOrigin::BottomLeft,
        color_type,
        None,
        None,
    )
    .expect("Unable to create surface");

    surface.canvas().scale((scale_factor, scale_factor));

    surface
}

pub fn skia_main(mut gui: Gui) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_context_version(3, 3);
    gl_attr.set_red_size(8);
    gl_attr.set_green_size(8);
    gl_attr.set_blue_size(8);
    gl_attr.set_double_buffer(true);
    gl_attr.set_depth_size(0);
    gl_attr.set_stencil_size(8);
    gl_attr.set_accelerated_visual(true);

    let mut window = video_subsystem
        .window("loopers",INITIAL_WIDTH as u32, INITIAL_HEIGHT as u32)
        .opengl()
        .resizable()
        .build()
        .unwrap();

    // must live until window is destroyed
    let _ctx = window.gl_create_context().unwrap();
    gl::load_with(|name| video_subsystem.gl_get_proc_address(&name) as *const _);

    let debug = std::env::var("DEBUG").is_ok();

    let mut gr_context = Context::new_gl(None).unwrap();

    let mut fboid: GLint = 0;
    unsafe { gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fboid) };

    let fb_info = FramebufferInfo {
        fboid: fboid.try_into().unwrap(),
        format: skia_safe::gpu::gl::Format::RGBA8.into(),
    };

    let size = window.drawable_size();
    let sf = size.0 as f32 / window.size().0 as f32;

    let pixel_format = window.window_pixel_format();

    let mut surface = create_surface(&mut gr_context, &pixel_format, fb_info, size, sf);

    let mut last_time = Instant::now();

    let mut frame_times = vec![0; 60];
    let mut frame_counter = 0;

    let mut capture_debug_frame = false;

    let mut last_event = None;
    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        window.gl_swap_window();

        gui.update();

        for event in event_pump.poll_iter() {
            match event {
                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::Resized(w, h) => {
                        surface =
                            create_surface(&mut gr_context, &pixel_format, fb_info,
                                           (w as u32, h as u32), sf);
                    }
                    WindowEvent::Close => break 'running,
                    _ => (),
                },
                Event::KeyDown {
                    keycode,
                    keymod,
                    ..
                } => {
                    if keycode == Some(Keycode::Question) && keymod.contains(Mod::LCTRLMOD) {
                        capture_debug_frame = true;
                    } else {
                        match keycode {
                            Some(key) => {
                                if let Some(c) = char_from_key(key) {
                                    last_event = Some(GuiEvent::KeyEvent(
                                        KeyEventType::Pressed,
                                        KeyEventKey::Char(c),
                                    ));
                                } else {
                                    let key = match key {
                                        Keycode::Backspace | Keycode::Delete => {
                                            Some(KeyEventKey::Backspace)
                                        }
                                        Keycode::Escape => Some(KeyEventKey::Esc),
                                        Keycode::Return => Some(KeyEventKey::Enter),
                                        _ => None,
                                    };

                                    if let Some(key) = key {
                                        last_event = Some(GuiEvent::KeyEvent(
                                            KeyEventType::Pressed,
                                            key,
                                        ));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Event::MouseMotion { x, y, .. } => {
                    last_event = Some(GuiEvent::MouseEvent(MouseEventType::Moved, (x, y)));
                }
                Event::MouseButtonDown { x, y, mouse_btn, .. } => {
                    last_event = Some(GuiEvent::MouseEvent(MouseEventType::MouseDown(mouse_btn), (x, y)));
                }
                Event::MouseButtonUp { x, y, mouse_btn, .. } => {
                    last_event = Some(GuiEvent::MouseEvent(MouseEventType::MouseUp(mouse_btn), (x, y)));
                }
                Event::Quit { .. } => {
                    break 'running;
                }
                _ => {}
            }
        }

        let mut canvas = surface.canvas();
        canvas.clear(BACKGROUND_COLOR.clone());

        let size = window.drawable_size();

        if debug && capture_debug_frame {
            let mut recorder = PictureRecorder::new();
            let mut recording_canvas = recorder.begin_recording(
                Rect::from_iwh(size.0 as i32, size.1 as i32),
                None,
                None,
            );

            canvas.clear(BACKGROUND_COLOR.clone());

            gui.draw(
                &mut recording_canvas,
                size.0 as f32,
                size.1 as f32,
                last_event,
            );

            let picture = recorder.finish_recording_as_picture(None).unwrap();
            let data = picture.serialize();
            let now = Local::now();

            let path =
                format!("/tmp/skia_dump_{}.skp", now.format("%Y-%m-%d_%H:%M:%S"));
            let mut file = File::create(&path).unwrap();

            info!("Captured debug frame to {}", path);

            file.write_all(data.as_bytes()).unwrap();
            capture_debug_frame = false;
        }

        gui.draw(
            &mut canvas,
            size.0 as f32,
            size.1 as f32,
            last_event,
        );

        last_event = None;

        let mut paint = Paint::default();
        paint.set_color(Color::from_rgb(255, 255, 255));
        paint.set_anti_alias(true);

        let avg_frame_time =
            frame_times.iter().sum::<u64>() as f32 / frame_times.len() as f32;
        let fps = 1.0 / (avg_frame_time / 1_000_000.0);

        let text = TextBlob::new(
            &format!("{:.1} fps", fps),
            &Font::new(Typeface::default(), 12.0),
        )
            .unwrap();

        if debug && frame_counter > frame_times.len() {
            canvas.draw_text_blob(
                &text,
                Point::new(
                    size.0 as f32 - text.bounds().width() + 10.0,
                    size.1 as f32 - 10.0,
                ),
                &paint,
            );
        }
        surface.canvas().flush();

        let min_size = gui.min_size();
        if let Err(e) = window.set_minimum_size(
            min_size.width as u32,
            min_size.height as u32,
        ) {
            warn!("Failed to set minimum window size: {:?}", e);
        }

        let frame_len = frame_times.len();
        frame_times[frame_counter % frame_len] =
            (Instant::now() - last_time).as_micros() as u64;
        frame_counter += 1;

        last_time = Instant::now();
    }
}

fn char_from_key(key: Keycode) -> Option<char> {
    Some(match key {
        Keycode::Num1 => '1',
        Keycode::Num2 => '2',
        Keycode::Num3 => '3',
        Keycode::Num4 => '4',
        Keycode::Num5 => '5',
        Keycode::Num6 => '6',
        Keycode::Num7 => '7',
        Keycode::Num8 => '8',
        Keycode::Num9 => '9',
        Keycode::Num0 => '0',
        Keycode::A => 'a',
        Keycode::B => 'b',
        Keycode::C => 'c',
        Keycode::D => 'd',
        Keycode::E => 'e',
        Keycode::F => 'f',
        Keycode::G => 'g',
        Keycode::H => 'h',
        Keycode::I => 'i',
        Keycode::J => 'j',
        Keycode::K => 'k',
        Keycode::L => 'l',
        Keycode::M => 'm',
        Keycode::N => 'n',
        Keycode::O => 'o',
        Keycode::P => 'p',
        Keycode::Q => 'q',
        Keycode::R => 'r',
        Keycode::S => 's',
        Keycode::T => 't',
        Keycode::U => 'u',
        Keycode::V => 'v',
        Keycode::W => 'w',
        Keycode::X => 'x',
        Keycode::Y => 'y',
        Keycode::Z => 'z',
        Keycode::Slash => '/',
        _ => return None,
    })
}
