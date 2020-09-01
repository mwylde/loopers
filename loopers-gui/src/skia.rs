use skia_safe::gpu::gl::FramebufferInfo;
use skia_safe::gpu::{BackendRenderTarget, Context, SurfaceOrigin};
use skia_safe::{
    Color, ColorType, Font, Paint, PictureRecorder, Point, Rect, Surface, TextBlob, Typeface,
};
use std::convert::TryInto;

use glutin::event::ElementState;
use glutin::event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::{ContextBuilder, GlProfile, PixelFormat};

use crate::{Gui, GuiEvent, KeyEventKey, KeyEventType, MouseEventType};
use gl::types::*;
use gl_rs as gl;

use chrono::Local;
use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::{Duration, Instant};
use winit::dpi::PhysicalSize;

const INITIAL_WIDTH: i32 = 800;
const INITIAL_HEIGHT: i32 = 600;

lazy_static! {
    pub static ref BACKGROUND_COLOR: Color = Color::from_rgb(29, 30, 39);
}

const FPS: u64 = 60;

fn create_surface(
    gr_context: &mut Context,
    pixel_format: &PixelFormat,
    fb_info: FramebufferInfo,
    size: PhysicalSize<u32>,
    scale_factor: f32,
) -> Surface {
    let backend_render_target = BackendRenderTarget::new_gl(
        (
            size.width.try_into().unwrap(),
            size.height.try_into().unwrap(),
        ),
        pixel_format.multisampling.map(|s| s.try_into().unwrap()),
        pixel_format.stencil_bits.try_into().unwrap(),
        fb_info,
    );

    let mut surface = Surface::from_backend_render_target(
        gr_context,
        &backend_render_target,
        SurfaceOrigin::BottomLeft,
        ColorType::RGBA8888,
        None,
        None,
    )
    .unwrap();

    surface.canvas().scale((scale_factor, scale_factor));

    surface
}

pub fn skia_main(mut gui: Gui) {
    let el = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("loopers-gui")
        .with_inner_size(glutin::dpi::LogicalSize {
            width: INITIAL_WIDTH,
            height: INITIAL_HEIGHT,
        });

    let cb = ContextBuilder::new()
        .with_depth_buffer(0)
        .with_stencil_buffer(8)
        .with_pixel_format(24, 8)
        .with_double_buffer(Some(true))
        .with_gl_profile(GlProfile::Core);

    let windowed_context = cb.build_windowed(wb, &el).unwrap();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };
    let pixel_format = windowed_context.get_pixel_format();

    debug!(
        "Pixel format of the window's GL context: {:?}",
        pixel_format
    );

    gl::load_with(|s| windowed_context.get_proc_address(&s));

    let mut gr_context = Context::new_gl(None).unwrap();

    let mut fboid: GLint = 0;
    unsafe { gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fboid) };

    let fb_info = FramebufferInfo {
        fboid: fboid.try_into().unwrap(),
        format: skia_safe::gpu::gl::Format::RGBA8.into(),
    };

    let size = windowed_context.window().inner_size();
    let sf = windowed_context.window().scale_factor() as f32;

    let mut surface = create_surface(&mut gr_context, &pixel_format, fb_info, size, sf);

    let inter_frame_time = Duration::from_micros(1_000_000 / FPS);

    let mut last_time = Instant::now();

    let mut frame_times = vec![0; 60];
    let mut frame_counter = 0;

    let mut mouse_position = None;
    let mut capture_debug_frame = false;

    let mut last_event = None;

    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        gui.update();

        #[allow(deprecated)]
        match event {
            Event::LoopDestroyed => {}
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    windowed_context.resize(physical_size);
                    surface =
                        create_surface(&mut gr_context, &pixel_format, fb_info, physical_size, sf);
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode,
                            state,
                            modifiers,
                            ..
                        },
                    ..
                } => {
                    if state == ElementState::Pressed {
                        if virtual_keycode == Some(VirtualKeyCode::Slash) && modifiers.ctrl() {
                            capture_debug_frame = true;
                        } else {
                            match virtual_keycode {
                                Some(key) => {
                                    if let Some(c) = char_from_key(key) {
                                        last_event = Some(GuiEvent::KeyEvent(
                                            KeyEventType::Pressed,
                                            KeyEventKey::Char(c),
                                        ));
                                    } else {
                                        let key = match key {
                                            VirtualKeyCode::Back | VirtualKeyCode::Delete => {
                                                Some(KeyEventKey::Backspace)
                                            }
                                            VirtualKeyCode::Escape => Some(KeyEventKey::Esc),
                                            VirtualKeyCode::Return => Some(KeyEventKey::Enter),
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
                }
                WindowEvent::CursorMoved { position, .. } => {
                    mouse_position = Some(position);
                    last_event = Some(GuiEvent::MouseEvent(MouseEventType::Moved, position));
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    if let Some(pos) = mouse_position {
                        let typ = match state {
                            ElementState::Pressed => MouseEventType::MouseDown(button),
                            ElementState::Released => MouseEventType::MouseUp(button),
                        };
                        last_event = Some(GuiEvent::MouseEvent(typ, pos));
                    };
                }
                _ => (),
            },
            Event::MainEventsCleared => {
                {
                    let mut canvas = surface.canvas();
                    canvas.clear(BACKGROUND_COLOR.clone());

                    let size = windowed_context.window().inner_size();

                    if capture_debug_frame {
                        let mut recorder = PictureRecorder::new();
                        let mut recording_canvas = recorder.begin_recording(
                            Rect::from_iwh(size.width as i32, size.height as i32),
                            None,
                            None,
                        );

                        recording_canvas.clear(Color::BLACK);

                        gui.draw(
                            &mut recording_canvas,
                            size.width as f32,
                            size.height as f32,
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
                        size.width as f32,
                        size.height as f32,
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

                    if frame_counter > frame_times.len() {
                        canvas.draw_text_blob(
                            &text,
                            Point::new(
                                size.width as f32 - text.bounds().width() + 10.0,
                                size.height as f32 - 10.0,
                            ),
                            &paint,
                        );
                    }
                }
                surface.canvas().flush();

                let diff = Instant::now() - last_time;
                if inter_frame_time > diff {
                    thread::sleep(inter_frame_time - diff);
                }

                windowed_context.swap_buffers().unwrap();

                let min_size = gui.min_size();
                windowed_context
                    .window()
                    .set_min_inner_size(Some(PhysicalSize::new(
                        min_size.width as i32,
                        min_size.height as i32,
                    )));

                let frame_len = frame_times.len();
                frame_times[frame_counter % frame_len] =
                    (Instant::now() - last_time).as_micros() as u64;
                frame_counter += 1;

                last_time = Instant::now();
            }
            _ => (),
        }
    });
}

fn char_from_key(key: VirtualKeyCode) -> Option<char> {
    Some(match key {
        VirtualKeyCode::Key1 => '1',
        VirtualKeyCode::Key2 => '2',
        VirtualKeyCode::Key3 => '3',
        VirtualKeyCode::Key4 => '4',
        VirtualKeyCode::Key5 => '5',
        VirtualKeyCode::Key6 => '6',
        VirtualKeyCode::Key7 => '7',
        VirtualKeyCode::Key8 => '8',
        VirtualKeyCode::Key9 => '9',
        VirtualKeyCode::Key0 => '0',
        VirtualKeyCode::A => 'a',
        VirtualKeyCode::B => 'b',
        VirtualKeyCode::C => 'c',
        VirtualKeyCode::D => 'd',
        VirtualKeyCode::E => 'e',
        VirtualKeyCode::F => 'f',
        VirtualKeyCode::G => 'g',
        VirtualKeyCode::H => 'h',
        VirtualKeyCode::I => 'i',
        VirtualKeyCode::J => 'j',
        VirtualKeyCode::K => 'k',
        VirtualKeyCode::L => 'l',
        VirtualKeyCode::M => 'm',
        VirtualKeyCode::N => 'n',
        VirtualKeyCode::O => 'o',
        VirtualKeyCode::P => 'p',
        VirtualKeyCode::Q => 'q',
        VirtualKeyCode::R => 'r',
        VirtualKeyCode::S => 's',
        VirtualKeyCode::T => 't',
        VirtualKeyCode::U => 'u',
        VirtualKeyCode::V => 'v',
        VirtualKeyCode::W => 'w',
        VirtualKeyCode::X => 'x',
        VirtualKeyCode::Y => 'y',
        VirtualKeyCode::Z => 'z',
        VirtualKeyCode::Slash => '/',
        _ => return None,
    })
}
