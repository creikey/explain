use crate::util::*;
use crate::gl_shaders::ShaderProgram;
use serde::{Deserialize, Serialize};

pub struct Movement {
    pub wrt_point: P2f64,
    pub zoom: f64,
    pub pan: V2f64,
}

impl Movement {
    pub fn new() -> Self {
        Self {
            wrt_point: P2f64::new(0.0, 0.0),
            zoom: 1.0,
            pan: V2f64::new(0.0, 0.0),
        }
    }
    pub fn apply_to_transform(&self, other: &mut ZoomTransform) {
        other.scale *= self.zoom;
        other.offset = self.zoom * other.offset + self.wrt_point.coords * (-self.zoom + 1.0);
        other.offset -= self.pan;
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ZoomTransform {
    scale: f64,
    offset: V2f64,
}

impl ZoomTransform {
    pub fn new(scale: f64, offset: V2f64) -> Self {
        Self { scale, offset }
    }
    pub fn does_nothing() -> Self {
        Self {
            scale: 1.0,
            offset: V2f64::new(0.0, 0.0),
        }
    }
    pub fn transform_other(&self, other: &mut Self) {
        other.scale *= self.scale;
        other.offset *= self.scale;
        other.offset += self.offset;
    }
    pub fn transform_point(&self, other: P2f64) -> P2f64 {
        other * self.scale + self.offset
    }
    pub fn inverse_transform_point(&self, other: P2f64) -> P2f64 {
        (other - self.offset) / self.scale
        // other*(1.0/self.scale) + (-self.offset/self.scale)
    }
    pub fn become_inverse(&mut self) {
        // other*self.scale + self.offset

        // (other - self.offset)/self.scale
        // other*(1.0/self.scale) + (-self.offset/self.scale)
        self.offset = -self.offset / self.scale;
        self.scale = 1.0 / self.scale;
    }
    /// Writes to the `offset` and `scale` uniforms of the shader. Intended to be
    /// processed in the vertex shader like:
    /// `vec2 newPosition = scale*Position + offset;`
    pub fn write_to_shader(&self, program: &ShaderProgram) {
        program.write_vec2("offset", &na::convert(self.offset));
        program.write_float("scale", self.scale as f32);
    }
}