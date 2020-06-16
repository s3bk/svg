use pathfinder_geometry::{
    vector::Vector2F,
    transform2d::Transform2F,
};
use pathfinder_renderer::scene::Scene;

use crate::{DrawOptions};

#[derive(Debug)]
pub struct DebugInfo {
    labels: Vec<(DebugEntry, String)>
}
impl DebugInfo {
    pub fn new() -> DebugInfo {
        DebugInfo {
            labels: Vec::new()
        }
    }
    pub fn add_point(&mut self, position: Vector2F, label: impl Into<String>) {
        self.labels.push((DebugEntry::Point { position }, label.into()));
    }
    pub fn add_vector(&mut self, origin: Vector2F, direction: Vector2F, label: impl Into<String>) {
        self.labels.push((DebugEntry::Vector { origin, direction }, label.into()));
    }
    pub (crate) fn draw(&self, scene: &mut Scene, options: &DrawOptions) {
        if self.labels.is_empty() {
            return;
        }
        for label in &self.labels {
            let pos = match label.0 {
                DebugEntry::Point { position } => position,
                DebugEntry::Vector { origin, direction } => origin + direction
            };
            let outline = options.debug_font.text(&label.1, Transform2F::from_translation(pos) * Transform2F::from_scale(20.0));
            options.draw(scene, &outline);
        }
    }
}

#[derive(Debug)]
enum DebugEntry {
    Point { position: Vector2F },
    Vector { origin: Vector2F, direction: Vector2F },
}