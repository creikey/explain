use crate::zooming::*;
use crate::line::*;
use crate::text::*;
use serde::{Deserialize, Serialize};

pub struct World {
    pub camera: ZoomTransform,
    pub lines: Vec<Line>,
    pub texts: Vec<Text>,
}

impl World {
    pub fn new() -> World {
        World {
            camera: ZoomTransform::does_nothing(),
            lines: vec![],
            texts: vec![],
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SavedWorld {
    camera: ZoomTransform,
    lines: Vec<SavedLine>,
    texts: Vec<SavedText>,
}

impl SavedWorld {
    pub fn from_world(w: &World) -> Self {
        // TODO with_capacity
        let mut lines: Vec<SavedLine> = Vec::new();
        let mut texts: Vec<SavedText> = Vec::new();

        for l in w.lines.iter() {
            lines.push(SavedLine::from_line(l));
        }
        for t in w.texts.iter() {
            texts.push(SavedText::from_text(t));
        }

        Self {
            lines,
            texts,
            camera: w.camera.clone(),
        }
    }
    pub fn into_world(&self) -> World {
        let mut lines: Vec<Line> = Vec::new();
        let mut texts: Vec<Text> = Vec::new();

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
