pub struct VertexAttrib {
    // TODO figure out a way to automatically calculate the gl_type and size of 3 components based
    // on a type parameter
    gl_type: gl::types::GLenum,
    size: gl::types::GLint,
    components: gl::types::GLint, // must be between 1 and 4
}

pub mod vertex_attribs {
    use super::*;
    pub const POINT3_F32: VertexAttrib = VertexAttrib {
        gl_type: gl::FLOAT,
        size: 3 * std::mem::size_of::<f32>() as i32,
        components: 3,
    };
    pub const POINT2_F32: VertexAttrib = VertexAttrib {
        gl_type: gl::FLOAT,
        size: 2 * std::mem::size_of::<f32>() as i32,
        components: 2,
    };
    pub const VECTOR2_F32: VertexAttrib = VertexAttrib {
        gl_type: gl::FLOAT,
        size: 2 * std::mem::size_of::<f32>() as i32,
        components: 2,
    };
}

pub struct VertexData<T> {
    vao: gl::types::GLuint,
    vbo: gl::types::GLuint,
    ebo: gl::types::GLuint,
    data: Vec<T>,
    indices: Vec<u32>, // NOTE this u32 must be the same size as GL_UNSIGNED_INT
    attributes: Vec<VertexAttrib>,
    stride: gl::types::GLsizei,
}

impl<T> VertexData<T> {
    /// The type T should be a single struct or a tuple of types that each vertex should have
    /// attached to it.
    ///
    /// # Arguments
    ///
    /// * `attributes` - A Vec that holds metadata about the type T used, that is later
    /// procedurally passed to gl. Preferably use the `vertex_attribs::*` constants.
    pub fn new(attributes: Vec<VertexAttrib>) -> Self {
        let mut vao: gl::types::GLuint = 0;
        let mut vbo: gl::types::GLuint = 0;
        let mut ebo: gl::types::GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);
        }
        // Automatically calculate stride based on the total size of all the vertex attributes. Not
        // sure if this is correct or not
        let mut stride: gl::types::GLint = 0;
        for a in attributes.iter() {
            stride += a.size;
            if !(a.components >= 1 && a.components <= 4) {
                panic!(
                    "Components of vector attribute must be between 1 and 4! You gave me: {}",
                    a.components
                );
            }
        }
        VertexData {
            vao,
            vbo,
            ebo,
            data: Vec::new(),
            indices: Vec::new(),
            attributes,
            stride,
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
    pub fn draw(&self) {
        self.activate();
        unsafe {
            gl::DrawElements(
                gl::TRIANGLES, // mode
                self.indices.len() as i32,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        }
        self.deactivate();
    }
    pub fn data(&mut self) -> &mut Vec<T> {
        &mut self.data
    }
    /// Length of vertices and attributes
    pub fn data_len(&self) -> usize {
        self.data.len()
    }
    pub fn get_vertex(&self, index: usize) -> &T {
        &self.data[index]
    }

    pub fn set_vertex_data(&mut self, index: usize, data: T, last_update: bool) {
        self.data[index] = data;
        self.update_on_gpu(last_update);
    }

    pub fn update_on_gpu(&mut self, last_update: bool) {
        let storage_type = if last_update {
            gl::STATIC_DRAW
        } else {
            gl::DYNAMIC_DRAW
        };
        fn vec_size<T>(v: &Vec<T>) -> gl::types::GLsizeiptr {
            (v.len() * std::mem::size_of::<T>()) as gl::types::GLsizeiptr
        }
        unsafe {
            gl::BindVertexArray(self.vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo); // bind the vbo buffer to the array_buffer slot
            gl::BufferData(
                gl::ARRAY_BUFFER, // target
                vec_size(&self.data),
                self.data.as_ptr() as *const gl::types::GLvoid, // pointer to data
                storage_type,
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                vec_size(&self.indices),
                self.indices.as_ptr() as *const gl::types::GLvoid,
                storage_type,
            );

            let mut attrib_array_i = 0;
            let mut cur_pointer_offset = 0;
            for a in self.attributes.iter() {
                gl::VertexAttribPointer(
                    attrib_array_i,
                    a.components,
                    a.gl_type,
                    gl::FALSE,
                    self.stride,
                    cur_pointer_offset as *const std::ffi::c_void,
                );
                gl::EnableVertexAttribArray(attrib_array_i);
                attrib_array_i += 1;
                cur_pointer_offset += a.size;
            }

            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
    }

    /// Automatically offsets the indices to the current length of the vertex array (so you can
    /// specify each index relative such that it starts from 0). Will also update the data onto GPU
    /// memory.
    /// # Arguments
    ///
    /// * `new_data` - new vertex data, directly copied into opengl memory so better be contiguous!
    /// * `new_indices` - new index order with which to use the vertices, used to avoid repetition
    /// * `last_update` - If this is the last vertex update, will be stored in static instead of
    /// dynamic memory for greater efficiency
    pub fn append(&mut self, new_data: &mut Vec<T>, new_indices: &mut Vec<u32>, last_update: bool) {
        for elem in new_indices.iter_mut() {
            *elem += self.data.len() as u32;
        }
        self.data.append(new_data);
        self.indices.append(new_indices);

        self.update_on_gpu(last_update);
    }
}
