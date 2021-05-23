use gl;
use nalgebra as na;
use std;
use std::ffi::CString;

// based off of https://github.com/Nercury/rust-and-opengl-lessons/blob/master/lesson-03/src/render_gl.rs

macro_rules! shader {
    ($vert_program:literal , $frag_program:literal) => {
        ShaderProgram::from_shaders(&[
            Shader::from_source(include_str!($vert_program), ShaderType::Vertex).unwrap(),
            Shader::from_source(include_str!($frag_program), ShaderType::Fragment).unwrap(),
        ])
        .unwrap()
    };
}
pub struct ShaderProgram {
    id: gl::types::GLuint,
}

impl ShaderProgram {
    fn get_location(&self, name: &str) -> i32 {
        unsafe {
            gl::GetUniformLocation(
                self.id(),
                CString::new(name).unwrap().as_bytes_with_nul().as_ptr() as *const i8,
            )
        }
    }
    // read only reference might be the wrong thing in this case, as it is modifying GPU data that
    // the shader "owns"
    pub fn write_mat4(&self, name: &str, mat: &na::Matrix4<f32>) {
        unsafe {
            gl::UniformMatrix4fv(
                self.get_location(name),
                1,
                gl::FALSE,
                mat.as_slice().as_ptr(),
            );
        }
    }
    pub fn write_mat3(&self, name: &str, mat: &na::Matrix3<f32>) {
        unsafe {
            gl::UniformMatrix3fv(
                self.get_location(name),
                1,
                gl::FALSE,
                mat.as_slice().as_ptr(),
            );
        }
    }
    pub fn write_float(&self, name: &str, f: f32) {
        unsafe {
            gl::Uniform1f(self.get_location(name), f);
        }
    }
    pub fn from_shaders(shaders: &[Shader]) -> Result<ShaderProgram, String> {
        let program_id = unsafe { gl::CreateProgram() };

        for shader in shaders {
            unsafe {
                gl::AttachShader(program_id, shader.id());
            }
        }

        unsafe {
            gl::LinkProgram(program_id);
        }

        let mut success: gl::types::GLint = 1;
        unsafe {
            gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let error = create_whitespace_cstring_with_len(len as usize);

            unsafe {
                gl::GetProgramInfoLog(
                    program_id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar,
                );
            }

            return Err(error.to_string_lossy().into_owned());
        }

        for shader in shaders {
            unsafe {
                gl::DetachShader(program_id, shader.id());
            }
        }

        Ok(ShaderProgram { id: program_id })
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }

    pub fn set_used(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

pub enum ShaderType {
    Vertex,
    Fragment,
}

impl ShaderType {
    fn to_gl_type(self) -> gl::types::GLuint {
        match self {
            ShaderType::Vertex => gl::VERTEX_SHADER,
            ShaderType::Fragment => gl::FRAGMENT_SHADER,
        }
    }
}

pub struct Shader {
    id: gl::types::GLuint,
}

impl Shader {
    pub fn from_source(source_str: &str, shader_kind: ShaderType) -> Result<Shader, String> {
        let kind = shader_kind.to_gl_type();
        let id = unsafe { gl::CreateShader(kind) };
        let source = &CString::new(source_str).unwrap();
        unsafe {
            gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
            gl::CompileShader(id);
        }

        let mut success: gl::types::GLint = 1;
        unsafe {
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let error = create_whitespace_cstring_with_len(len as usize);

            unsafe {
                gl::GetShaderInfoLog(
                    id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar,
                );
            }

            return Err(error.to_string_lossy().into_owned());
        }

        Ok(Shader { id })
    }

    fn id(&self) -> gl::types::GLuint {
        self.id
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

fn create_whitespace_cstring_with_len(len: usize) -> CString {
    // allocate buffer of correct size
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    // fill it with len spaces
    buffer.extend([b' '].iter().cycle().take(len));
    // convert buffer to CString
    unsafe { CString::from_vec_unchecked(buffer) }
}
