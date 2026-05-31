use tauri::Manager;
use std::sync::Arc;
use tokio::sync::Mutex;

mod commands;
mod db;
mod monitors;
mod ai;
mod utils;

pub use db::Database;
pub use monitors::*;
 
use ai::assistant_commands::AssistantAppState;

pub struct AppState {
    pub db: Arc<Mutex<Database>>,
    pub monitor_handles: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let app_handle = app.handle().clone();

            let data_dir = app.path().app_data_dir()
                .expect("Failed to get app data dir");
            std::fs::create_dir_all(&data_dir).ok();

            let db = Database::new(&data_dir.join("second_brain.db"))
                .expect("Failed to initialize database");
            let db = Arc::new(Mutex::new(db));

            app.manage(AppState {
                db: db.clone(),
                monitor_handles: Arc::new(Mutex::new(Vec::new())),
            });

            app.manage(AssistantAppState::new());

            // Background monitors
            let db_clone = db.clone();
            let handle_clone = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                monitors::start_all_monitors(db_clone, handle_clone).await;
            });

            // Voice assistant loop
            let handle_clone2 = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                let assistant_state = handle_clone2.state::<AssistantAppState>();
                let config = assistant_state.config.lock().await.clone();
                let db_for_assistant = handle_clone2.state::<AppState>().db.clone();
                ai::voice_assistant::run_assistant(db_for_assistant, handle_clone2, config).await;
            });

            log::info!("Second Brain initialized");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_timeline,
            commands::search_events,
            commands::delete_event,
            commands::get_system_stats,
            commands::get_active_window_now,
            commands::get_memory_stats,
            commands::get_app_usage,
            commands::get_productivity_summary,
            commands::get_clipboard_history,
            commands::get_screenshot_history,
            commands::get_file_events,
            commands::get_browser_history,
            commands::get_ai_insights,
            commands::export_data,
            commands::clear_history,
            commands::get_settings,
            commands::update_settings,
            ai::assistant_commands::get_assistant_config,
            ai::assistant_commands::update_assistant_config,
            ai::assistant_commands::check_llm_provider,
            ai::assistant_commands::chat_with_assistant,
            ai::assistant_commands::speak_text,
            ai::assistant_commands::clear_conversation,
            ai::assistant_commands::get_assistant_state,
            ai::assistant_commands::get_recent_alerts,
            ai::assistant_commands::acknowledge_alert,
            ai::assistant_commands::list_ollama_models,
            ai::assistant_commands::get_system_alert_check,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running Second Brain");
}
