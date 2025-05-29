mod database_handler;
use tauri::Manager;

type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            database_handler::init_database(app.app_handle().clone())?;
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            database_handler::save_note,
            database_handler::load_note,
            database_handler::list_notes,
            database_handler::load_notes_for_days,
            database_handler::delete_note
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
