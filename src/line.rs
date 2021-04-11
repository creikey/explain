extern crate gl;
use crate::gl_shaders::*;
use crate::Drawable;
use na::Vector3;
use sdl2::event::Event;
use sdl2::rect::Point;
use std::ffi::CString;
use std::time::{Duration, Instant};

// NOTE the float datatype of this vector _must_ match the float datatype used to hand the vector
// data to shaders
type V = Vector3<f32>;

struct VertexData {
    vao: gl::types::GLuint,
    vbo: gl::types::GLuint,
    vertices: Vec<V>,
}

impl VertexData {
    fn new() -> Self {
        let mut vao: gl::types::GLuint = 0;
        let mut vbo: gl::types::GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
        }
        VertexData {
            vao,
            vbo,
            vertices: Vec::new(),
        }
    }
    fn activate(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
        }
    }
    fn deactivate(&self) {
        unsafe {
            gl::BindVertexArray(0);
        }
    }
}

pub struct Line {
    shader_program: ShaderProgram,
    gl_vertices: VertexData,
    points: Vec<Point>,
    counter: f32,
}

impl Line {
    pub fn new() -> Line {
        let shader_program = ShaderProgram::from_shaders(&[
            Shader::from_source(include_str!("line.vert"), ShaderType::Vertex).unwrap(),
            Shader::from_source(include_str!("line.frag"), ShaderType::Fragment).unwrap(),
        ])
        .unwrap();

        Line {
            points: Vec::new(),
            shader_program,
            gl_vertices: VertexData::new(),
            counter: 0.0,
        }
    }
}

impl Drawable for Line {
    fn draw(&self, projection: &na::Matrix4<f32>) {
        // draw triangle
        self.shader_program.set_used();
        unsafe {
            let projection_location = gl::GetUniformLocation(
                self.shader_program.id(),
                CString::new("projection")
                    .unwrap()
                    .as_bytes_with_nul()
                    .as_ptr() as *const i8,
            );
            gl::UniformMatrix4fv(
                projection_location,
                1,
                gl::FALSE,
                projection.as_slice().as_ptr(),
            );
            self.gl_vertices.activate();
            gl::DrawArrays(
                gl::TRIANGLES,                          // mode
                0,                                      // starting index in the enabled arrays
                self.gl_vertices.vertices.len() as i32, // number of indices to be rendered
            );
        }
    }
    fn process_event(&mut self, e: &Event) {
        self.counter += 0.16;
        if let Event::MouseMotion { x, y, .. } = *e {
            self.gl_vertices
                .vertices
                .push(Vector3::new(x as f32, y as f32, 0.0));
            unsafe {
                gl::BindVertexArray(self.gl_vertices.vao);
                gl::BindBuffer(gl::ARRAY_BUFFER, self.gl_vertices.vbo); // bind the vbo buffer to the array_buffer slot
                gl::BufferData(
                    gl::ARRAY_BUFFER, // target
                    (self.gl_vertices.vertices.len() * std::mem::size_of::<V>())
                        as gl::types::GLsizeiptr, // size of data in bytes
                    self.gl_vertices.vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
                    gl::DYNAMIC_DRAW,                                               // usage
                );
                gl::VertexAttribPointer(
                    0,
                    3,
                    gl::FLOAT,
                    gl::FALSE,
                    (3 * std::mem::size_of::<f32>()) as gl::types::GLint,
                    std::ptr::null(),
                );
                gl::EnableVertexAttribArray(0);
                gl::BindBuffer(gl::ARRAY_BUFFER, 0); // clear the array_buffer slot
            }
        }
    }
}
