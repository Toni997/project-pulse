use crate::audio::snapshot::render_graph::Render;

pub struct TrackNode {
    pub volume: f32,
    pub pan: f32,
    pub muted: bool,
}

impl Render for TrackNode {
    fn render(&self) -> Vec<f32> {
        todo!()
    }
}
