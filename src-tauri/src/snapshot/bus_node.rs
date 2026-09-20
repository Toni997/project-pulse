pub struct BusNode {
    volume: f32,
    pan: f32,
    muted: bool,
}

impl BusNode {
    pub fn new(volume: f32, pan: f32, muted: bool) -> Self {
        Self { volume, pan, muted }
    }
}
