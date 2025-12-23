// 모듈 선언
pub mod types;
pub mod retriever;
pub mod parser;
pub mod commands;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 로깅 초기화
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_persisted_scope::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::detect_game_engine,
            commands::is_game_supported,
            commands::create_rpg_maker_project_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
