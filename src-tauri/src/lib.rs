pub mod clips;
pub mod commands;
pub mod error;
pub mod state;
pub mod storage;

use commands::{clear_history, delete_clip, get_clip, get_settings, list_clips, update_settings};
use state::AppState;
use storage::Database;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let database = Database::open(&data_dir.join("clipclop.db"))?;
            app.manage(AppState::new(database));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_clips,
            get_clip,
            delete_clip,
            clear_history,
            get_settings,
            update_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
