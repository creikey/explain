extern crate gl;
use crate::gl_vertices::*;
use crate::{ExplainObject, ZoomTransform, TypedExplainObject, Shaders};
use crate::util::*;
use sdl2::event::Event;
use serde_json::{from_str, Map, Value};

extern crate image;

use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct SavedText {
    text: String,
    transform: ZoomTransform,
    origin: P2,
}

impl SavedText {
    pub fn from_text(t: &Text) -> Self {
        Self {
            transform: t.zoom_transform.clone(),
            text: t.text.clone(),
            origin: t.origin.clone(),
        }
    }

    pub fn into_text(&self) -> Text {
        let mut to_return = Text::new(self.origin);
        to_return.zoom_transform = self.transform.clone();

        for c in self.text.chars() {
            to_return.add_character(&c.to_string());
        }

        to_return
    }
}

#[derive(Clone)]
pub struct Text {
    gl_vertices: VertexData<(P2, P2)>,
    texture: gl::types::GLuint,
    character_map: Map<String, Value>,
    zoom_transform: ZoomTransform,
    size: (u64, u64),
    origin: P2,
    width_offset: f32,
    text: String,
}

impl Text {
    pub fn new(origin: P2) -> Self {

        use vertex_attribs::*;
        let gl_vertices = VertexData::new(vec![POINT2_F32, POINT2_F32]);
        use image::DynamicImage;
        let img = image::load_from_memory(include_bytes!("arial-font.png")).unwrap();
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
            texture,
            gl_vertices,
            size,
            character_map,
            origin,
            zoom_transform: ZoomTransform::does_nothing(),
            width_offset: 0.0,
            text: String::from(""),
        }
    }
    fn add_character(&mut self, key_string: &String) {
        let character_to_rect = self
            .character_map
            .get("characters")
            .unwrap()
            .as_object()
            .unwrap();
        let rect = character_to_rect
            .get(key_string)
            .unwrap()
            .as_object()
            .unwrap();
        self.text.push_str(key_string);
        // TODO use strong typed version of this json stuff (should probably do this when font logic is abstracted to multiple fonts)
        let width_in_px = rect.get("width").unwrap().as_i64().unwrap();
        let height_in_px = rect.get("height").unwrap().as_i64().unwrap();
        let width = width_in_px as f32 / self.size.0 as f32;
        let height = height_in_px as f32 / self.size.1 as f32;
        let x = rect.get("x").unwrap().as_i64().unwrap() as f32 / self.size.0 as f32;
        let y = rect.get("y").unwrap().as_i64().unwrap() as f32 / self.size.1 as f32;
        let origin_y = rect.get("originY").unwrap().as_i64().unwrap();
        let origin_x = -rect.get("originX").unwrap().as_i64().unwrap() as f32;

        let vertical_offset = self.character_map.get("size").unwrap().as_i64().unwrap() - origin_y;
        let mut new_vertices = vec![
            (
                P2::new(self.width_offset + origin_x, vertical_offset as f32),
                P2::new(x, y),
            ), // upper left
            (
                P2::new(
                    self.width_offset + origin_x + width_in_px as f32,
                    vertical_offset as f32,
                ),
                P2::new(x + width, y),
            ), // upper right
            (
                P2::new(
                    self.width_offset + origin_x + width_in_px as f32,
                    (vertical_offset + height_in_px) as f32,
                ),
                P2::new(x + width, y + height),
            ), // lower right
            (
                P2::new(
                    self.width_offset + origin_x,
                    (vertical_offset + height_in_px) as f32,
                ),
                P2::new(x, y + height),
            ), // lower left
        ];
        for p in new_vertices.iter_mut() {
            p.0.x += self.origin.x;
            p.0.y += self.origin.y;
        }
        self.gl_vertices
            .append(&mut new_vertices, &mut vec![0, 1, 2, 0, 3, 2], false);
        self.width_offset += rect.get("advance").unwrap().as_i64().unwrap() as f32;
    }
}

impl ExplainObject for Text {
    // TODO figure out how to give this behavior to an object without
    // copy and pasting this method everywhere
    fn set_transform(&mut self, z: ZoomTransform) {
        self.zoom_transform = z;
    }
    fn draw(&self, shaders: &Shaders, projection: &na::Matrix4<f32>, camera: &ZoomTransform) {
        shaders.text.set_used();
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
        }

        let mut transform_to_use = ZoomTransform::does_nothing();
        self.zoom_transform.transform_other(&mut transform_to_use);
        camera.transform_other(&mut transform_to_use);

        shaders.text.write_mat4("projection", projection);
        transform_to_use.write_to_shader(&shaders.text);
        self.gl_vertices.draw();
    }

    fn process_event(&mut self, e: &Event) -> bool {
        if let Event::KeyDown { keycode, .. } = *e {
            return keycode.unwrap() != sdl2::keyboard::Keycode::Return;
        }
        if let Event::TextInput { text, .. } = &*e {
            let key_string = text;
            let character_to_rect = self
                .character_map
                .get("characters")
                .unwrap()
                .as_object()
                .unwrap();

            if !character_to_rect.contains_key(key_string) {
                return false;
            }

            self.add_character(key_string);

            return true;
        }
        false
    }
    fn get_as_type(&self) -> TypedExplainObject {
        TypedExplainObject::Text((*self).clone())
    }
}
