use tauri::State;
use serde::Serialize;
use chrono::Utc;
use sysinfo::System;

use crate::AppState;
use crate::db::{Settings, TimelineEvent};
use crate::monitors::window_monitor::get_active_window_title;
use crate::monitors::get_active_app_name;
use crate::ai::{AIEngine, ProductivityContext};

// ── Timeline ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_timeline(
    state: State<'_, AppState>,
    limit: Option<i64>,
    offset: Option<i64>,
    event_type: Option<String>,
) -> Result<Vec<TimelineEvent>, String> {
    let db = state.db.lock().await;
    db.get_timeline(limit.unwrap_or(50), offset.unwrap_or(0), event_type.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_events(
    state: State<'_, AppState>,
    query: String,
    limit: Option<i64>,
) -> Result<Vec<TimelineEvent>, String> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }
    let db = state.db.lock().await;
    db.search_events(&query, limit.unwrap_or(30))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_event(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let db = state.db.lock().await;
    db.delete_event(&id).map_err(|e| e.to_string())
}

// ── System Stats ─────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct SystemStats {
    cpu_usage: f32,
    ram_used_mb: u64,
    ram_total_mb: u64,
    ram_percent: f32,
    disk_used_gb: f32,
    disk_total_gb: f32,
    disk_percent: f32,
    network_rx_kb: u64,
    network_tx_kb: u64,
    battery_percent: Option<f32>,
    battery_charging: Option<bool>,
    uptime_secs: u64,
    process_count: usize,
    top_processes: Vec<ProcessInfo>,
}

#[derive(Serialize)]
pub struct ProcessInfo {
    name: String,
    cpu_percent: f32,
    ram_mb: u64,
}

#[tauri::command]
pub async fn get_system_stats() -> Result<SystemStats, String> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let disks = sysinfo::Disks::new_with_refreshed_list();
    let networks = sysinfo::Networks::new_with_refreshed_list();

    let ram_used = sys.used_memory() / 1024 / 1024;
    let ram_total = sys.total_memory() / 1024 / 1024;
    let ram_percent = if ram_total > 0 { ram_used as f32 / ram_total as f32 * 100.0 } else { 0.0 };

    let (disk_used, disk_total) = disks.iter().fold((0.0f32, 0.0f32), |acc, d| {
        let total = d.total_space() as f32 / 1_073_741_824.0;
        let used = (d.total_space() - d.available_space()) as f32 / 1_073_741_824.0;
        (acc.0 + used, acc.1 + total)
    });
    let disk_percent = if disk_total > 0.0 { disk_used / disk_total * 100.0 } else { 0.0 };

    let (rx, tx) = networks.iter().fold((0u64, 0u64), |acc, (_, n)| {
        (acc.0 + n.received() / 1024, acc.1 + n.transmitted() / 1024)
    });

    // Top 5 processes by CPU
    let mut procs: Vec<ProcessInfo> = sys.processes().values()
        .map(|p| ProcessInfo {
            name: p.name().to_string(),
            cpu_percent: p.cpu_usage(),
            ram_mb: p.memory() / 1024 / 1024,
        })
        .collect();
    procs.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap_or(std::cmp::Ordering::Equal));
    procs.truncate(5); 

    Ok(SystemStats {
        cpu_usage: sys.global_cpu_info().frequency() as f32,
        ram_used_mb: ram_used,
        ram_total_mb: ram_total,
        ram_percent,
        disk_used_gb: disk_used,
        disk_total_gb: disk_total,
        disk_percent,
        network_rx_kb: rx,
        network_tx_kb: tx,
        battery_percent: None,
        battery_charging: None,
        uptime_secs: System::uptime(),
        process_count: sys.processes().len(),
        top_processes: procs,
    })
}

// ── App Usage ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_app_usage(
    state: State<'_, AppState>,
    date: Option<String>,
) -> Result<serde_json::Value, String> {
    let db = state.db.lock().await;
    let date_str = date.unwrap_or_else(|| Utc::now().date_naive().to_string());
    let entries = db.get_app_usage_by_date(&date_str).map_err(|e| e.to_string())?;
    
    let total_secs: i64 = entries.iter().map(|e| e.duration_secs).sum();
    
    Ok(serde_json::json!({
        "date": date_str,
        "total_minutes": total_secs / 60,
        "entries": entries,
    }))
}

// ── AI Insights ───────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_ai_insights(
    state: State<'_, AppState>,
) -> Result<Vec<crate::ai::AIInsight>, String> {
    let db = state.db.lock().await;
    let settings = db.get_settings().map_err(|e| e.to_string())?;

    if !settings.ai_insights_enabled {
        return Ok(vec![]);
    }

    let api_key = settings.openrouter_api_key
        .ok_or("No API key configured")?;

    let today = Utc::now().date_naive().to_string();
    let usage = db.get_app_usage_by_date(&today).map_err(|e| e.to_string())?;

    drop(db); // Release lock before async calls

    let ai = AIEngine::new(api_key);

    let context = ProductivityContext {
        top_apps: usage.iter().take(5).map(|u| u.app_name.clone()).collect(),
        active_minutes: usage.iter().map(|u| u.duration_secs).sum::<i64>() / 60,
        focus_sessions: 0, // TODO: calculate from timeline
        context_switches_per_hour: usage.len() as f32,
        peak_hour: "Unknown".to_string(),
        browser_tabs: 0,
    };

    let summary = ai.generate_productivity_summary(&context).await
        .map_err(|e| e.to_string())?;

    Ok(vec![summary])
}

#[tauri::command]
pub async fn get_productivity_summary(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let db = state.db.lock().await;
    let today = Utc::now().date_naive().to_string();
    let usage = db.get_app_usage_by_date(&today).map_err(|e| e.to_string())?;
    let total_events = db.get_total_event_count().map_err(|e| e.to_string())?;
    let db_size = db.get_db_size_mb().map_err(|e| e.to_string())?;

    let total_secs: i64 = usage.iter().map(|e| e.duration_secs).sum();
    
    // Group by category
    let mut by_category: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for entry in &usage {
        let cat = entry.category.clone().unwrap_or_else(|| "other".to_string());
        *by_category.entry(cat).or_default() += entry.duration_secs;
    }

    Ok(serde_json::json!({
        "today": {
            "total_active_minutes": total_secs / 60,
            "top_apps": usage.iter().take(10).collect::<Vec<_>>(),
            "by_category": by_category,
        },
        "memory": {
            "total_events": total_events,
            "db_size_mb": db_size,
        }
    }))
}

// ── Clipboard ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_clipboard_history(
    state: State<'_, AppState>,
    limit: Option<i64>,
) -> Result<Vec<crate::db::ClipboardEntry>, String> {
    let db = state.db.lock().await;
    db.get_clipboard_history(limit.unwrap_or(100))
        .map_err(|e| e.to_string())
}

// ── File Events ───────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_file_events(
    state: State<'_, AppState>,
    limit: Option<i64>,
) -> Result<Vec<TimelineEvent>, String> {
    let db = state.db.lock().await;
    let all = db.get_timeline(limit.unwrap_or(50), 0, Some("file_created"))
        .map_err(|e| e.to_string())?;
    Ok(all)
}

// ── Screenshots ───────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_screenshot_history(
    state: State<'_, AppState>,
    limit: Option<i64>,
) -> Result<Vec<TimelineEvent>, String> {
    let db = state.db.lock().await;
    db.get_timeline(limit.unwrap_or(20), 0, Some("screenshot"))
        .map_err(|e| e.to_string())
}

// ── Browser ───────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_browser_history(
    limit: Option<i64>,
) -> Result<Vec<crate::monitors::browser_monitor::BrowserVisit>, String> {
    use crate::monitors::browser_monitor::*;

    let since = Utc::now().timestamp() - 86400; // Last 24h
    let paths = get_browser_history_paths();
    let mut all_visits = Vec::new();

    for (browser, path) in &paths {
        let visits = if browser == "Firefox" {
            read_firefox_history(path, since)
        } else {
            read_chromium_history(path, browser, since)
        };
        all_visits.extend(visits);
    }

    // Sort by visit time descending
    all_visits.sort_by(|a, b| b.visit_time.cmp(&a.visit_time));
    all_visits.truncate(limit.unwrap_or(100) as usize);

    Ok(all_visits)
}

// ── Active Window ─────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ActiveWindowInfo {
    app_name: Option<String>,
    window_title: Option<String>,
    timestamp: i64,
}

#[tauri::command]
pub async fn get_active_window_now() -> Result<ActiveWindowInfo, String> {
    Ok(ActiveWindowInfo {
        app_name: get_active_app_name(),
        window_title: get_active_window_title(),
        timestamp: Utc::now().timestamp(),
    })
}

// ── Settings ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_settings(
    state: State<'_, AppState>,
) -> Result<Settings, String> {
    let db = state.db.lock().await;
    db.get_settings().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_settings(
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<(), String> {
    let db = state.db.lock().await;
    db.save_settings(&settings).map_err(|e| e.to_string())
}

// ── Memory ────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct MemoryStats {
    total_events: i64,
    db_size_mb: f64,
    oldest_event_days: i64,
}

#[tauri::command]
pub async fn get_memory_stats(
    state: State<'_, AppState>,
) -> Result<MemoryStats, String> {
    let db = state.db.lock().await;
    Ok(MemoryStats {
        total_events: db.get_total_event_count().unwrap_or(0),
        db_size_mb: db.get_db_size_mb().unwrap_or(0.0),
        oldest_event_days: 0, // TODO
    })
}

// ── Data Management ───────────────────────────────────────────────────────────

#[tauri::command]
pub async fn export_data(
    state: State<'_, AppState>,
    _format: Option<String>,
) -> Result<String, String> {
    let db = state.db.lock().await;
    let events = db.get_timeline(10000, 0, None).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(&events).map_err(|e| e.to_string())?;

    let export_path = dirs::download_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(format!("second_brain_export_{}.json", chrono::Utc::now().format("%Y%m%d_%H%M%S")));

    std::fs::write(&export_path, json).map_err(|e| e.to_string())?;
    Ok(export_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn clear_history(
    state: State<'_, AppState>,
    days: Option<i64>,
) -> Result<(), String> {
    let db = state.db.lock().await;
    db.prune_old_data(days.unwrap_or(0)).map_err(|e| e.to_string())
}
