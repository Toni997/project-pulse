use crate::audio::snapshot::render_graph::Render;

pub struct ClipNode {
    source_offset_samples: usize,
    source_path: String,
}

impl Render for ClipNode {
    fn render(&self) -> Vec<f32> {
        todo!()
    }
}
