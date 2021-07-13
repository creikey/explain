extern crate directories;
extern crate gl;
extern crate nalgebra as na;
extern crate sdl2;
extern crate serde;
#[macro_use]
mod gl_shaders;
mod gl_vertices;
mod line;
mod saving;
mod text;
mod util;
mod world;
mod zooming;

use line::Line;
use saving::*;
use text::Text;
use util::*;
use world::*;
use zooming::*;

use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;
use sdl2::{event::Event, mouse};
use std::time::Duration;

/// Stuff that is on the whiteboard, panned/zoomed around
pub trait ExplainObject {
    fn set_transform(&mut self, z: ZoomTransform);
    fn draw(&self, shaders: &Shaders, projection: &na::Matrix4<f32>, camera: &ZoomTransform);
    fn process_event(&mut self, e: &Event) -> bool;
    fn get_as_type(&self) -> TypedExplainObject; // this will copy, don't use it all the time
}

pub enum TypedExplainObject {
    line(Line),
    text(Text),
}

// Should there be a better scheme for how shaders are stored/managed or is this good enough?
pub struct Shaders {
    line: gl_shaders::ShaderProgram,
    text: gl_shaders::ShaderProgram,
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
    let mut world = load_or_new_world();
    let mut currently_creating: Option<Box<dyn ExplainObject>> = None;

    let mut event_pump = sdl_context.event_pump().unwrap();

    // gl stuff
    let shaders = Shaders {
        line: shader!("line.vert", "line.frag"),
        text: shader!("text.vert", "text.frag"),
    };
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

        let mut cur_movement = Movement::new();
        for event in event_pump.poll_iter() {
            use sdl2::mouse::MouseButton;
            let mut consumed_event = false;
            if let Some(object) = &mut currently_creating {
                let mut new_transform = world.camera.clone();
                new_transform.become_inverse();
                object.set_transform(new_transform);
                consumed_event = object.process_event(&event);
            }

            // TODO move to world
            fn push_object_if_there(
                window: &sdl2::video::Window,
                world: &mut World,
                object: Option<Box<dyn ExplainObject>>,
            ) {
                if let Some(object) = object {
                    let as_type = object.get_as_type();
                    match as_type {
                        TypedExplainObject::line(l) => {
                            world.lines.push(l);
                        }
                        TypedExplainObject::text(t) => {
                            world.texts.push(t);
                        }
                    }
                    save(window, world);
                }
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
                        push_object_if_there(&window, &mut world, currently_creating);
                        currently_creating = Some(Box::new(Line::new()));
                    }
                    Event::MouseButtonUp {
                        mouse_btn: MouseButton::Left,
                        ..
                    } => {
                        push_object_if_there(&window, &mut world, currently_creating);
                        currently_creating = None
                    }

                    // text
                    Event::KeyDown {
                        keycode: Some(Keycode::T),
                        ..
                    } => {
                        push_object_if_there(&window, &mut world, currently_creating);
                        let global_pos = mouse_pos;
                        currently_creating =
                            Some(Box::new(Text::new(P2::new(global_pos.x, global_pos.y))));
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Return),
                        ..
                    } => {
                        push_object_if_there(&window, &mut world, currently_creating);
                        currently_creating = None;
                    }

                    // zooming
                    Event::MouseWheel { y, .. } => {
                        let scale_delta = 1.0 + (y as f64) * 0.05;
                        cur_movement.zoom = scale_delta;
                        cur_movement.wrt_point = na::convert(mouse_pos);
                    }

                    Event::KeyDown {
                        keycode: Some(Keycode::E),
                        ..
                    } => {
                        // debug e key to zoom out really far
                        let scale_delta = 0.1;
                        cur_movement.zoom = scale_delta;
                        cur_movement.wrt_point = P2f64::new(105.0, 73.0);
                        // cur_movement.wrt_point = mouse_pos;
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Q),
                        ..
                    } => {
                        // debug q key to zoom in really far to first object
                        let scale_delta = 10.0;
                        cur_movement.zoom = scale_delta;
                        cur_movement.wrt_point = P2f64::new(105.0, 73.0);
                        // P2::from(items[0].get_moved_around().get_drawing_transform().offset);
                    }

                    // panning
                    Event::MouseMotion { xrel, yrel, .. } => {
                        if middle_down {
                            cur_movement.pan -= V2f64::new(xrel as f64, yrel as f64);
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
        cur_movement.apply_to_transform(&mut world.camera);
        if let Some(o) = &mut currently_creating {
            o.draw(&shaders, mat, &world.camera);
        }
        for t in world.texts.iter_mut() {
            t.draw(&shaders, mat, &world.camera);
        }
        for l in world.lines.iter_mut() {
            l.draw(&shaders, mat, &world.camera);
        }

        window.gl_swap_window();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60)); // TODO take exactly 1/60s every time by accounting for how long computation above takes
    }
}
