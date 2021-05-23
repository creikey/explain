extern crate gl;
use crate::gl_shaders::*;
use crate::gl_vertices::*;
use crate::Drawable;
use na::Point3;
use sdl2::event::Event;
use serde_json::{from_str, Map, Value};

extern crate image;

type P = na::Point3<f32>;
type P2 = na::Point2<f32>;

pub struct Text {
    shader_program: ShaderProgram,
    gl_vertices: VertexData<(P, P2)>,
    texture: gl::types::GLuint,
    character_map: Map<String, Value>,
    size: (u64, u64),
    width_offset: f32,
    text: String,
}

impl Text {
    pub fn new() -> Self {
        let shader_program = shader!("text.vert", "text.frag");

        use vertex_attribs::*;
        let mut gl_vertices = VertexData::new(vec![POINT3_F32, POINT2_F32]);
        use image::DynamicImage;
        let img = image::open("src/arial-font.png").unwrap();
        let mut texture = 0;
        let mut size = (0, 0);
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            match img {
                DynamicImage::ImageRgba8(buf) => {
                    size.0 = buf.width() as u64;
                    size.1 = buf.height() as u64;
                    gl::TexImage2D(
                        gl::TEXTURE_2D,
                        0,
                        gl::RGBA as i32,
                        buf.width() as i32,
                        buf.height() as i32,
                        0,
                        gl::RGBA as u32,
                        gl::UNSIGNED_BYTE,
                        buf.as_ptr() as *const gl::types::GLvoid,
                    );
                    gl::GenerateMipmap(gl::TEXTURE_2D);
                    println!("Created texture: {}", texture);
                }
                _ => {
                    panic!("Unexpected image type: {}", &format!("{:?}", img)[0..10]);
                }
            }

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        let maybe_character_map: Value = from_str(include_str!("arial-font.json")).unwrap();
        let character_map;
        match maybe_character_map {
            Value::Object(m) => {
                character_map = m;
            }
            _ => {
                panic!("Unexpected json type from sdf character location map");
            }
        }

        Text {
            shader_program,
            texture,
            gl_vertices,
            size,
            character_map,
            width_offset: 0.0,
            text: String::from("A"),
        }
    }
}

impl Drawable for Text {
    fn draw(&self, projection: &na::Matrix4<f32>, camera: &na::Matrix4<f32>) {
        self.shader_program.set_used();
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
        }
        self.shader_program.write_mat4("projection", projection);
        self.shader_program.write_mat4("camera", camera);
        self.gl_vertices.draw();
    }
    fn process_event(&mut self, e: &Event, camera_inv: &na::Matrix4<f32>) {
        if let Event::TextInput { text, .. } = &*e {
            let key_string = text;
            println!("{:?}", key_string);
            let character_to_rect = self
                .character_map
                .get("characters")
                .unwrap()
                .as_object()
                .unwrap(); // TODO move this to constructor
            if character_to_rect.contains_key(key_string) {
                let rect = character_to_rect
                    .get(key_string)
                    .unwrap()
                    .as_object()
                    .unwrap();
                // TODO use strong typed version of this json stuff (should probably do this when font logic is abstracted to multiple fonts)
                let width_in_px = rect.get("width").unwrap().as_i64().unwrap();
                let height_in_px = rect.get("height").unwrap().as_i64().unwrap();
                let width = width_in_px as f32 / self.size.0 as f32;
                let height = height_in_px as f32 / self.size.1 as f32;
                let x = rect.get("x").unwrap().as_i64().unwrap() as f32 / self.size.0 as f32;
                let y = rect.get("y").unwrap().as_i64().unwrap() as f32 / self.size.1 as f32;
                let origin_y = rect.get("originY").unwrap().as_i64().unwrap();
                let origin_x = -rect.get("originX").unwrap().as_i64().unwrap() as f32;

                let vertical_offset =
                    self.character_map.get("size").unwrap().as_i64().unwrap() - origin_y;
                println!("{}", vertical_offset);
                self.gl_vertices.append(
                    &mut vec![
                        (
                            P::new(self.width_offset + origin_x, vertical_offset as f32, 0.0),
                            P2::new(x, y),
                        ), // upper left
                        (
                            P::new(
                                self.width_offset + origin_x + width_in_px as f32,
                                vertical_offset as f32,
                                0.0,
                            ),
                            P2::new(x + width, y),
                        ), // upper right
                        (
                            P::new(
                                self.width_offset + origin_x + width_in_px as f32,
                                (vertical_offset + height_in_px) as f32,
                                0.0,
                            ),
                            P2::new(x + width, y + height),
                        ), // lower right
                        (
                            P::new(
                                self.width_offset + origin_x,
                                (vertical_offset + height_in_px) as f32,
                                0.0,
                            ),
                            P2::new(x, y + height),
                        ), // lower left
                    ],
                    &mut vec![0, 1, 2, 0, 3, 2],
                    false,
                );
                self.width_offset += rect.get("advance").unwrap().as_i64().unwrap() as f32;
            }
        }
    }
}