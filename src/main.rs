extern crate gl;
extern crate nalgebra as na;
extern crate sdl2;

pub mod gl_shaders;
pub mod line;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::video::GLProfile;
use sdl2::video::Window;
use std::time::Duration;

use line::Line;

pub trait Drawable {
    fn draw(&self, projection: &na::Matrix4<f32>);
    fn process_event(&mut self, e: &Event);
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    let window = video_subsystem
        .window("explain", 800, 600)
        .opengl()
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let ctx = window.gl_create_context().unwrap();
    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);

    debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
    debug_assert_eq!(gl_attr.context_version(), (3, 3));

    // array of items that dynamically expands as user creates more items with the various tools
    // available
    let mut items: Vec<Box<dyn Drawable>> = Vec::new();
    let mut item_currently_creating: Option<Box<dyn Drawable>> = None;

    let mut event_pump = sdl_context.event_pump().unwrap();

    // gl stuff
    let mut projection = nalgebra::Orthographic3::new(0.0, 800.0, 600.0, 0.0, -1.0, 1.0);
    let mut drawing_wireframe = false;
    unsafe {
        gl::Viewport(0, 0, 800, 600);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }
    'running: loop {
        unsafe {
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        for event in event_pump.poll_iter() {
            if let Some(item) = &mut item_currently_creating {
                item.process_event(&event);
            }
            use sdl2::mouse::MouseButton;
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Left,
                    ..
                } => {
                    item_currently_creating = Some(Box::new(Line::new()));
                }
                Event::MouseButtonUp {
                    mouse_btn: MouseButton::Left,
                    ..
                } => {
                    if item_currently_creating.is_some() {
                        items.push(item_currently_creating.unwrap());
                        item_currently_creating = None;
                    }
                }
                #[cfg(debug_assertions)]
                Event::KeyDown {
                    keycode: Some(Keycode::Z),
                    ..
                } => {
                    drawing_wireframe = !drawing_wireframe;
                    unsafe {
                        if drawing_wireframe {
                            gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
                        } else {
                            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
                        }
                    }
                }
                Event::Window { win_event, .. } => match win_event {
                    sdl2::event::WindowEvent::Resized(x, y) => unsafe {
                        gl::Viewport(0, 0, x, y);
                        projection.set_right(x as f32);
                        projection.set_bottom(y as f32);
                    },
                    _ => {}
                },
                _ => {}
            }
        }

        let mat = projection.as_matrix();
        for i in items.iter() {
            i.draw(mat);
        }
        if let Some(item) = &item_currently_creating {
            item.draw(mat);
        }

        window.gl_swap_window();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
