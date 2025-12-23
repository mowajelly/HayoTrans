// 모듈 선언
pub mod types;
pub mod retriever;
pub mod parser;
pub mod commands;
pub mod storage;

use commands::AppState;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 로깅 초기화
    tracing_subscriber::fmt::init();

    // 앱 상태 초기화
    let app_state = AppState::new()
        .expect("Failed to initialize app state");

    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_persisted_scope::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            // Retriever commands
            commands::detect_game_engine,
            commands::is_game_supported,
            commands::create_rpg_maker_project_file,
            // Project commands
            commands::get_projects,
            commands::add_project,
            commands::delete_project,
            commands::open_project,
            commands::detect_engine,
            // Config commands
            commands::get_config,
            commands::set_language,
            commands::set_theme,
            commands::set_last_project,
            commands::set_window_size,
            commands::get_app_data_path,
            commands::open_app_data_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
