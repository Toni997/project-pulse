use crate::audio::engine::AUDIO_ENGINE;

pub fn initialize_project(loaded_file_path: Option<&str>) {
    // load settings
    // load project
    unsafe {
        AUDIO_ENGINE.start();
    }
}
