extern crate gl;
extern crate nalgebra as na;
extern crate sdl2;

#[macro_use]
pub mod gl_shaders;
pub mod gl_vertices;
mod line;
mod text;

use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;
use sdl2::{event::Event, mouse};
use std::time::Duration;

// TODO put all these type aliases into util mod
type P2 = na::Point2<f32>;
type V2 = na::Vector2<f32>;

pub struct Movement {
    wrt_point: P2,
    zoom: f32,
    pan: V2,
}

impl Movement {
    fn new() -> Self {
        Self {
            wrt_point: P2::new(0.0, 0.0),
            zoom: 1.0,
            pan: V2::new(0.0, 0.0),
        }
    }
}

pub trait Drawable {
    fn camera_move(&mut self, m: &Movement);
    fn draw(&self, projection: &na::Matrix4<f32>);
    fn process_event(&mut self, e: &Event) -> bool;
}

// https://www.khronos.org/opengl/wiki/OpenGL_Error
extern "system" fn message_callback(
    source: gl::types::GLenum,
    t: gl::types::GLenum,
    id: gl::types::GLuint,
    severity: gl::types::GLenum,
    length: gl::types::GLsizei,
    message: *const gl::types::GLchar,
    user_param: *mut gl::types::GLvoid,
) {
    unsafe {
        let is_error = t == gl::DEBUG_TYPE_ERROR;

        let type_name = if is_error {
            String::from("ERROR")
        } else {
            format!("Type {}", t)
        };
        if is_error {
            println!(
                "GL {}: {}",
                type_name,
                std::ffi::CStr::from_ptr(message).to_str().unwrap()
            );
        }
    }
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::GLES);
    gl_attr.set_context_major_version(2);
    gl_attr.set_context_minor_version(0);

    let window = video_subsystem
        .window("explain", 800, 600)
        .opengl()
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let _ctx = window.gl_create_context().unwrap();
    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);

    debug_assert_eq!(gl_attr.context_profile(), GLProfile::GLES);
    debug_assert_eq!(gl_attr.context_version(), (2, 0));

    // ui state
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
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::DebugMessageCallback(Some(message_callback), std::ptr::null());
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }
    'running: loop {
        unsafe {
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        let ms = event_pump.mouse_state();
        let middle_down = ms.middle();
        let mouse_pos = P2::new(ms.x() as f32, ms.y() as f32);
        drop(ms);
        // commits the item if it's there, if it's not does nothing.
        fn commit_item(
            items: &mut Vec<Box<dyn Drawable>>,
            item_currently_creating: Option<Box<dyn Drawable>>,
        ) -> Option<Box<dyn Drawable>> {
            if item_currently_creating.is_some() {
                items.push(item_currently_creating.unwrap());
                None
            } else {
                item_currently_creating
            }
        }

        let mut cur_movement = Movement::new();
        for event in event_pump.poll_iter() {
            use sdl2::mouse::MouseButton;
            // pass event on through trait
            let mut consumed_event = false;
            if let Some(item) = &mut item_currently_creating {
                consumed_event = item.process_event(&event);
            }
            if !consumed_event {
                match event {
                    // TODO refactor thing creation to put whether element is created or not into element's file
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,

                    // creation and deletion
                    Event::MouseButtonDown {
                        mouse_btn: MouseButton::Left,
                        ..
                    } => {
                        item_currently_creating = commit_item(&mut items, item_currently_creating);
                        item_currently_creating = Some(Box::new(line::Line::new()));
                    }
                    Event::MouseButtonUp {
                        mouse_btn: MouseButton::Left,
                        ..
                    } => {
                        item_currently_creating = commit_item(&mut items, item_currently_creating);
                    }

                    // text
                    Event::KeyDown {
                        keycode: Some(Keycode::T),
                        ..
                    } => {
                        item_currently_creating = commit_item(&mut items, item_currently_creating);
                        let global_pos = mouse_pos;
                        item_currently_creating = Some(Box::new(text::Text::new(P2::new(
                            global_pos.x,
                            global_pos.y,
                        ))));
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Return),
                        ..
                    } => {
                        item_currently_creating = commit_item(&mut items, item_currently_creating);
                    }

                    // zooming
                    Event::MouseWheel { y, .. } => {
                        let scale_delta = 1.0 + (y as f32) * 0.04;
                        cur_movement.zoom = scale_delta;
                        cur_movement.wrt_point = mouse_pos;
                    }

                    // panning
                    Event::MouseMotion { xrel, yrel, .. } => {
                        if middle_down {
                            cur_movement.pan -= V2::new(xrel as f32, yrel as f32);
                        }
                    }

                    // debug wireframe mode
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

                    // resize the gl canvas with the window
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
        }

        let mat = projection.as_matrix();
        if let Some(item) = &mut item_currently_creating {
            item.camera_move(&cur_movement);
            item.draw(mat);
        }
        for i in items.iter_mut() {
            i.camera_move(&cur_movement);
            i.draw(mat);
        }

        window.gl_swap_window();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60)); // TODO take exactly 1/60s every time by accounting for how long computation above takes
    }
}
