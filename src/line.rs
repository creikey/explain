extern crate gl;
use crate::gl_shaders::*;
use crate::gl_vertices::*;
use crate::Drawable;
use na::Vector2;
use sdl2::event::Event;

type P = na::Point3<f32>;
type V2 = Vector2<f32>;

pub struct Line {
    shader_program: ShaderProgram,
    gl_vertices: VertexData<(P, V2)>,
    last_point: Option<P>,
}

impl Line {
    pub fn new() -> Line {
        let shader_program = shader!("line.vert", "line.frag");

        use vertex_attribs::*;
        Line {
            last_point: None,
            shader_program,
            gl_vertices: VertexData::new(vec![POINT3_F32, VECTOR2_F32]),
        }
    }
}

impl Drawable for Line {
    fn draw(&self, projection: &na::Matrix4<f32>, camera: &na::Matrix4<f32>) {
        if self.gl_vertices.data_len() == 0 {
            return; // nothing in the vertices array, nothing to draw
        }
        self.shader_program.set_used();
        self.shader_program.write_mat4("projection", projection);
        self.shader_program.write_mat4("camera", camera);
        self.shader_program.write_float("width", 2.0);
        self.gl_vertices.draw();
    }
    fn process_event(&mut self, e: &Event, camera_inv: &na::Matrix4<f32>) -> bool {
        // TODO when line is committed move out of DYNAMIC_DRAW memory
        if let Event::MouseMotion { x, y, .. } = *e {
            let new_point = (camera_inv).transform_point(&na::Point3::new(x as f32, y as f32, 0.0));

            if self.last_point.is_none() {
                self.last_point = Some(new_point);
                return true;
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
            if self.gl_vertices.data_len() > 0 {
                last_up = self
                    .gl_vertices
                    .get_vertex(self.gl_vertices.data_len() - 2)
                    .1;
                last_down = self
                    .gl_vertices
                    .get_vertex(self.gl_vertices.data_len() - 1)
                    .1;
            }

            self.gl_vertices.append(
                &mut vec![
                    (last_point, last_up),
                    (last_point, last_down),
                    (new_point, up),
                    (new_point, down),
                ],
                &mut vec![0, 1, 3, 0, 2, 3],
                false,
            );

            self.last_point = Some(new_point);
            return true;
        }
        false
    }
}
