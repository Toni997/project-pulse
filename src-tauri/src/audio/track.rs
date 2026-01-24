use serde::Deserialize;

#[derive(Deserialize)]
pub struct Track {
    name: String,
    volume: f32,
    pan: f32,
    muted: bool,
    audio_file: String,
}
