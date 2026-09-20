pub struct TrackNode {
    volume: f32,
    pan: f32,
    muted: bool,
}

impl TrackNode {
    pub fn new(volume: f32, pan: f32, muted: bool) -> Self {
        Self { volume, pan, muted }
    }
}
