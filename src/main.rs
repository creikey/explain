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

pub struct Camera {
    pub zoom: f32,
    pub offset: V2,
}

impl Camera {
    fn new() -> Self {
        Self {
            zoom: 0.0,
            offset: V2::new(0.0, 0.0),
        }
    }
    // WARNING: IF CHANGE FUNCTIONS, UPDATE IN THE SHADERS
    // as the vertices of shaders are stored in WORLD coordinates,
    // the shader uses the world_to_canvas function in glsl, so if it's changed here it MUST be updated in the shader as well.
    // TODO maybe figure out a way to put this logic into a matrix transform so code doesn't have to be updated across two mediums?
    fn canvas_to_world(&self, canvas_coordinate: P2) -> P2 {
        (canvas_coordinate + self.offset) * 2.0f32.powf(self.zoom)
    }
    fn world_to_canvas(&self, world_coordinate: P2) -> P2 {
        (world_coordinate) / 2.0f32.powf(self.zoom) - self.offset
    }
}

#[cfg(test)]
mod camera_tests {
    use super::*;
    #[test]
    fn transforms_are_inverse() {
        let mut c = Camera::new();
        // whole numbers so there aren't any tiny float imprecision problems with assert_eq
        // I would rather not use "about equal" tests as that may be indicative of a small problem with the inverse
        c.zoom = 2.0;
        c.offset = V2::new(300.0, -500.0);
        let input_point = P2::new(20.0, -25.0);
        println!("{}", c.world_to_canvas(input_point));
        assert_eq!(
            input_point,
            c.canvas_to_world(c.world_to_canvas(input_point))
        );
        assert_eq!(
            input_point,
            c.world_to_canvas(c.canvas_to_world(input_point))
        );
    }
}

pub trait Drawable {
    fn draw(&self, projection: &na::Matrix4<f32>, camera: &Camera);
    fn process_event(&mut self, e: &Event, camera: &Camera) -> bool;
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
    let mut camera = Camera::new();
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
        // println!("{}", camera.offset + (mouse_pos.coords * 2.0f32.powf(camera.zoom)));
        for event in event_pump.poll_iter() {
            use sdl2::mouse::MouseButton;
            // pass event on through trait
            let mut consumed_event = false;
            if let Some(item) = &mut item_currently_creating {
                consumed_event = item.process_event(&event, &camera);
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
                        // let before_zoom_world_position = camera.offset + (mouse_pos.coords * 2.0f32.powf(camera.zoom));
                        // let before_zoom_world_position = camera.offset + mouse_pos.coords;
                        let before_zoom = camera.canvas_to_world(mouse_pos);
                        camera.zoom -= (y as f32) * 0.04;
                        let after_zoom = camera.canvas_to_world(mouse_pos);
                        camera.offset += camera.world_to_canvas(before_zoom) - camera.world_to_canvas(after_zoom);
                        // println!("{}", before_zoom - after_zoom);
                        // let after_zoom_world_position =
                        // camera.offset + (mouse_pos.coords * 2.0f32.powf(camera.zoom));
                        // camera.offset += before_zoom_world_position - after_zoom_world_position;
                        // println!("{} , {}", before_zoom_world_position, after_zoom_world_position);
                        let scale_mat = na::Matrix3::new_nonuniform_scaling_wrt_point(
                            &na::Vector2::new(scale_delta, scale_delta),
                            &mouse_pos,
                        );
                    }
                    Event::KeyDown {
                        // TODO remove this it's a hack for testing zoom stuff
                        keycode: Some(Keycode::U),
                        ..
                    } => {
                        camera.offset = V2::new(0.0, 0.0);
                    }

                    // panning
                    Event::MouseMotion { xrel, yrel, .. } => {
                        if middle_down {
                            camera.offset -= V2::new(xrel as f32, yrel as f32);
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
        for i in items.iter() {
            i.draw(mat, &camera);
        }
        if let Some(item) = &item_currently_creating {
            item.draw(mat, &camera);
        }

        window.gl_swap_window();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
