pub struct MasterNode {
    volume: f32,
    pan: f32,
    muted: bool,
}

impl MasterNode {
    pub fn new(volume: f32, pan: f32, muted: bool) -> Self {
        Self { volume, pan, muted }
    }
}
