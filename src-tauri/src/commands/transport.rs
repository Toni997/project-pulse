use crate::audio::transport::TRANSPORT;

#[tauri::command]
pub fn transport_stop() {
    TRANSPORT.stop();
}

#[tauri::command]
pub fn transport_play() {
    // TODO eventually create a dedicated thread for transport
    tauri::async_runtime::spawn_blocking(|| TRANSPORT.play());
}
