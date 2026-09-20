use std::sync::Arc;

use tauri::State;

use crate::app_state::AppState;

#[tauri::command]
pub fn transport_stop(state: State<Arc<AppState>>) {
    state.transport.stop();
}

#[tauri::command]
pub fn transport_play(state: State<Arc<AppState>>) {
    // TODO eventually create a dedicated thread for transport
    let transport = Arc::clone(&state.transport);
    tauri::async_runtime::spawn_blocking(move || transport.play());
}
