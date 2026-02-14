use crate::audio::snapshot::render_graph::Render;

pub struct MasterNode {
    pub volume: f32,
    pub pan: f32,
    pub muted: bool,
}

impl Render for MasterNode {
    fn render(&self) -> Vec<f32> {
        todo!()
    }
}
