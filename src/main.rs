extern crate directories;
extern crate gl;
extern crate nalgebra as na;
extern crate sdl2;
extern crate serde;
#[macro_use]
pub mod gl_shaders;
pub mod gl_vertices;
mod line;
mod text;
mod util;

use util::*;
use line::Line;
use text::Text;

use directories::{BaseDirs, ProjectDirs, UserDirs};
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;
use sdl2::{event::Event, mouse};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub struct Movement {
    wrt_point: P2f64,
    zoom: f64,
    pan: V2f64,
}

impl Movement {
    fn new() -> Self {
        Self {
            wrt_point: P2f64::new(0.0, 0.0),
            zoom: 1.0,
            pan: V2f64::new(0.0, 0.0),
        }
    }
    fn apply_to_transform(&self, other: &mut ZoomTransform) {
        other.scale *= self.zoom;
        other.offset = self.zoom * other.offset + self.wrt_point.coords * (-self.zoom + 1.0);
        other.offset -= self.pan;
    }
}
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ZoomTransform {
    scale: f64,
    offset: V2f64,
}

impl ZoomTransform {
    fn new(scale: f64, offset: V2f64) -> Self {
        Self { scale, offset }
    }
    fn does_nothing() -> Self {
        Self {
            scale: 1.0,
            offset: V2f64::new(0.0, 0.0),
        }
    }
    fn transform_other(&self, other: &mut Self) {
        other.scale *= self.scale;
        other.offset *= self.scale;
        other.offset += self.offset;
    }
    fn transform_point(&self, other: P2f64) -> P2f64 {
        other * self.scale + self.offset
    }
    fn inverse_transform_point(&self, other: P2f64) -> P2f64 {
        (other - self.offset) / self.scale
        // other*(1.0/self.scale) + (-self.offset/self.scale)
    }
    fn become_inverse(&mut self) {
        // other*self.scale + self.offset

        // (other - self.offset)/self.scale
        // other*(1.0/self.scale) + (-self.offset/self.scale)
        self.offset = -self.offset / self.scale;
        self.scale = 1.0 / self.scale;
    }
    /// Writes to the `offset` and `scale` uniforms of the shader. Intended to be
    /// processed in the vertex shader like:
    /// `vec2 newPosition = scale*Position + offset;`
    fn write_to_shader(&self, program: &gl_shaders::ShaderProgram) {
        program.write_vec2("offset", &na::convert(self.offset));
        program.write_float("scale", self.scale as f32);
    }
}

pub trait Drawable {
    fn set_transform(&mut self, z: ZoomTransform);
    fn draw(&self, projection: &na::Matrix4<f32>, camera: &ZoomTransform);
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

fn fatal_msgbox(window: &sdl2::video::Window, msg: &str) {
    use sdl2::messagebox::{show_simple_message_box, MessageBoxFlag};
    show_simple_message_box(MessageBoxFlag::ERROR, "Fatal Error", msg, window).unwrap();
    panic!();
}

fn expect_msgbox<T: std::fmt::Debug, E: std::fmt::Debug>(
    window: &sdl2::video::Window,
    r: Result<T, E>,
    msg: &str,
) -> T {
    if r.is_ok() {
        r.unwrap()
    } else {
        fatal_msgbox(window, format!("{} - {:?}", msg, r.unwrap_err()).as_str());
        panic!();
    }
}

fn get_save_directory_path() -> PathBuf {
    // TODO msgbox the unwrap
    PathBuf::from(
        ProjectDirs::from("com", "creikey", "Explain")
            .unwrap()
            .data_dir(),
    )
}

fn get_save_file_path() -> PathBuf {
    get_save_directory_path().join("save.explain")
}

fn save(window: &sdl2::video::Window, world: &World) {
    let save_directory = get_save_directory_path();
    expect_msgbox(
        &window,
        std::fs::create_dir_all(&save_directory),
        format!(
            "failed to create save directory in {}",
            save_directory.to_str().unwrap()
        )
        .as_str(),
    );
    use std::fs::File;
    use std::io::prelude::*;
    let saved_world = SavedWorld::from_world(world);
    let encoded = bincode::serialize(&saved_world).unwrap();
    let save_file_path = get_save_file_path();
    println!(
        "{} | {}",
        save_directory.to_str().unwrap(),
        save_file_path.to_str().unwrap()
    );

    use std::fs::OpenOptions;
    let mut save_file = if save_file_path.exists() {
        OpenOptions::new().write(true).open(save_file_path).unwrap()
    } else {
        File::create(save_file_path).unwrap()
    };
    save_file.write_all(encoded.as_slice()).unwrap();
}

fn load_or_new_world() -> World {
    // TODO this stuff should definitely expect_msgbox to show that the save file is corrupt
    let save_path = get_save_file_path();
    let to_return: World;
    if save_path.exists() {
        let bytes = std::fs::read(save_path).unwrap();
        let saved_world: SavedWorld = bincode::deserialize(bytes.as_slice()).unwrap();
        to_return = saved_world.into_world();
    } else {
        to_return = World::new();
    }

    to_return
}

struct World {
    camera: ZoomTransform,
    lines: Vec<line::Line>,
    texts: Vec<text::Text>,
}

impl World {
    fn new() -> World {
        World {
            camera: ZoomTransform::does_nothing(),
            lines: vec![],
            texts: vec![],
        }
    }
}

#[derive(Serialize, Deserialize)]
struct SavedWorld {
    camera: ZoomTransform,
    lines: Vec<line::SavedLine>,
    texts: Vec<text::SavedText>,
}

impl SavedWorld {
    fn from_world(w: &World) -> Self {
        // TODO with_capacity
        let mut lines: Vec<line::SavedLine> = Vec::new();
        let mut texts: Vec<text::SavedText> = Vec::new();

        for l in w.lines.iter() {
            lines.push(line::SavedLine::from_line(l));
        }
        for t in w.texts.iter() {
            texts.push(text::SavedText::from_text(t));
        }

        Self {
            lines,
            texts,
            camera: w.camera.clone(),
        }
    }
    fn into_world(&self) -> World {
        let mut lines: Vec<line::Line> = Vec::new();
        let mut texts: Vec<text::Text> = Vec::new();

        for l in self.lines.iter() {
            lines.push(l.into_line());
        }
        for t in self.texts.iter() {
            texts.push(t.into_text());
        }

        World {
            lines,
            texts,
            camera: self.camera.clone(),
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
    let mut currently_creating_line: Option<Line> = None;
    let mut currently_creating_text: Option<Text> = None;

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

        let mut cur_movement = Movement::new();
        for event in event_pump.poll_iter() {
            use sdl2::mouse::MouseButton;
            let mut consumed_event = false;
            if let Some(line) = &mut currently_creating_line {
                let mut new_transform = world.camera.clone();
                new_transform.become_inverse();
                line.set_transform(new_transform);
                consumed_event = line.process_event(&event);
            }
            if let Some(text) = &mut currently_creating_text {
                let mut new_transform = world.camera.clone();
                new_transform.become_inverse();
                text.set_transform(new_transform);
                consumed_event = text.process_event(&event);
            }

            fn push_line_if_there(
                window: &sdl2::video::Window,
                world: &mut World,
                line: Option<Line>,
            ) -> Option<Line> {
                match line {
                    Some(o) => {
                        world.lines.push(o);
                        save(&window, &world);
                        None
                    }
                    None => line,
                }
            }
            fn push_text_if_there(
                window: &sdl2::video::Window,
                world: &mut World,
                text: Option<Text>,
            ) -> Option<Text> {
                match text {
                    Some(o) => {
                        world.texts.push(o);
                        save(&window, &world);
                        None
                    }
                    None => text,
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
                        currently_creating_line =
                            push_line_if_there(&window, &mut world, currently_creating_line);
                        currently_creating_line = Some(Line::new());
                    }
                    Event::MouseButtonUp {
                        mouse_btn: MouseButton::Left,
                        ..
                    } => {
                        currently_creating_line =
                            push_line_if_there(&window, &mut world, currently_creating_line);
                    }

                    // text
                    Event::KeyDown {
                        keycode: Some(Keycode::T),
                        ..
                    } => {
                        currently_creating_text =
                            push_text_if_there(&window, &mut world, currently_creating_text);
                        let global_pos = mouse_pos;
                        currently_creating_text =
                            Some(Text::new(P2::new(global_pos.x, global_pos.y)));
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Return),
                        ..
                    } => {
                        currently_creating_text =
                            push_text_if_there(&window, &mut world, currently_creating_text);
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
        if let Some(line) = &mut currently_creating_line {
            line.draw(mat, &world.camera);
        }
        if let Some(text) = &mut currently_creating_text {
            text.draw(mat, &world.camera);
        }
        for t in world.texts.iter_mut() {
            t.draw(mat, &world.camera);
        }
        for l in world.lines.iter_mut() {
            l.draw(mat, &world.camera);
        }

        window.gl_swap_window();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60)); // TODO take exactly 1/60s every time by accounting for how long computation above takes
    }
}
