extern crate gl;
use crate::gl_shaders::*;
use crate::Drawable;
use na::{Vector2, Vector3};
use sdl2::event::Event;
use sdl2::rect::Point;
use std::ffi::CString;
use std::time::{Duration, Instant};

type V = Vector3<f32>;
type P = na::Point3<f32>;
type V2 = Vector2<f32>;

fn vec_size<T>(v: &Vec<T>) -> gl::types::GLsizeiptr {
    (v.len() * std::mem::size_of::<T>()) as gl::types::GLsizeiptr
}

struct VertexData {
    vao: gl::types::GLuint,
    vbo: gl::types::GLuint,
    ebo: gl::types::GLuint,
    data: Vec<(P, V2)>,
    indices: Vec<u32>, // NOTE this u32 must be the same size as GL_UNSIGNED_INT
}

impl VertexData {
    fn new() -> Self {
        let mut vao: gl::types::GLuint = 0;
        let mut vbo: gl::types::GLuint = 0;
        let mut ebo: gl::types::GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);
        }
        VertexData {
            vao,
            vbo,
            ebo,
            data: Vec::new(),
            indices: Vec::new(),
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
    last_point: Option<P>,
}

impl Line {
    pub fn new() -> Line {
        let shader_program = ShaderProgram::from_shaders(&[
            Shader::from_source(include_str!("line.vert"), ShaderType::Vertex).unwrap(),
            Shader::from_source(include_str!("line.frag"), ShaderType::Fragment).unwrap(),
        ])
        .unwrap();

        Line {
            last_point: None,
            shader_program,
            gl_vertices: VertexData::new(),
        }
    }
}

impl Drawable for Line {
    fn draw(&self, projection: &na::Matrix4<f32>, camera: &na::Matrix4<f32>) {
        if self.gl_vertices.data.len() == 0 {
            return; // nothing in the vertices array, nothing to draw
        }
        // draw triangle
        self.shader_program.set_used();
        self.shader_program.write_mat4("projection", projection);
        self.shader_program.write_mat4("camera", camera);
        self.shader_program.write_float("width", 2.0);
        unsafe {
            self.gl_vertices.activate();
            gl::DrawElements(
                gl::TRIANGLES, // mode
                self.gl_vertices.indices.len() as i32,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
            self.gl_vertices.deactivate();
        }
    }
    fn process_event(&mut self, e: &Event, camera_inv: &na::Matrix4<f32>) {
        // TODO when line is committed move out of DYNAMIC_DRAW memory
        if let Event::MouseMotion { x, y, .. } = *e {
            let new_point = (camera_inv).transform_point(&na::Point3::new(x as f32, y as f32, 0.0));

            if self.last_point.is_none() {
                self.last_point = Some(new_point);
                return;
            }
            let last_point = self.last_point.unwrap();

            use std::f32::consts::PI;
            fn rotate(v: V2, theta: f32) -> V2 {
                let rot = na::Matrix2::new(theta.cos(), -theta.sin(), theta.sin(), theta.cos());
                rot * v
            }
            let towards_new = (new_point - last_point).normalize().xy();
            let up = rotate(towards_new, -PI / 2.0);
            let down = rotate(towards_new, PI / 2.0);

            // I duplicate the last plane's up/down normals so that the line appears contiguous,
            // TODO change to use bisector and some sort of bevel thing
            let mut last_up = up;
            let mut last_down = down;
            if self.gl_vertices.data.len() > 0 {
                last_up = self.gl_vertices.data[self.gl_vertices.data.len() - 2].1;
                last_down = self.gl_vertices.data[self.gl_vertices.data.len() - 1].1;
            }

            self.gl_vertices.data.append(&mut vec![
                (last_point, last_up),
                (last_point, last_down),
                (new_point, up),
                (new_point, down),
            ]);

            let mut new_indices: Vec<u32> = vec![0, 1, 3, 0, 2, 3];
            for elem in new_indices.iter_mut() {
                *elem += self.gl_vertices.data.len() as u32;
            }
            self.gl_vertices.indices.append(&mut new_indices);

            unsafe {
                gl::BindVertexArray(self.gl_vertices.vao);

                gl::BindBuffer(gl::ARRAY_BUFFER, self.gl_vertices.vbo); // bind the vbo buffer to the array_buffer slot
                gl::BufferData(
                    gl::ARRAY_BUFFER, // target
                    vec_size(&self.gl_vertices.data),
                    self.gl_vertices.data.as_ptr() as *const gl::types::GLvoid, // pointer to data
                    gl::DYNAMIC_DRAW,                                           // usage
                );

                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.gl_vertices.ebo);
                gl::BufferData(
                    gl::ELEMENT_ARRAY_BUFFER,
                    vec_size(&self.gl_vertices.indices),
                    self.gl_vertices.indices.as_ptr() as *const gl::types::GLvoid,
                    gl::DYNAMIC_DRAW,
                );

                gl::VertexAttribPointer(
                    0,
                    3,
                    gl::FLOAT,
                    gl::FALSE,
                    (5 * std::mem::size_of::<f32>()) as gl::types::GLint,
                    std::ptr::null(),
                );
                gl::EnableVertexAttribArray(0);

                gl::VertexAttribPointer(
                    1,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    (5 * std::mem::size_of::<f32>()) as gl::types::GLint,
                    (3 * std::mem::size_of::<f32>()) as *const std::ffi::c_void,
                );
                gl::EnableVertexAttribArray(1);

                gl::BindVertexArray(0);
                gl::BindBuffer(gl::ARRAY_BUFFER, 0);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            }

            self.last_point = Some(new_point);
        }
    }
}
