use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use tauri::AppHandle;
use sysinfo::{System, Disks, Networks};
use uuid::Uuid;
use chrono::Utc;

use crate::db::{Database, SystemSnapshot, ClipboardEntry, TimelineEvent};

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub mod window_monitor;
pub mod file_monitor;
pub mod browser_monitor;

use file_monitor::start_file_watcher;

pub async fn start_all_monitors(db: Arc<Mutex<Database>>, _app: AppHandle) {
    log::info!("Starting all monitors...");

    let db1 = db.clone();
    let db2 = db.clone();
    let db3 = db.clone();
    let db4 = db.clone();
    let db5 = db.clone();

    // System stats monitor (every 5s)
    tokio::spawn(async move {
        system_monitor(db1).await;
    });

    // Clipboard monitor (every 1s)
    tokio::spawn(async move {
        clipboard_monitor(db2).await;
    });

    // Active window monitor (every 2s)
    tokio::spawn(async move {
        window_monitor::active_window_monitor(db3).await;
    });

    // File system watcher
    tokio::spawn(async move {
        if let Err(e) = start_file_watcher(db4).await {
            log::error!("File watcher error: {}", e);
        }
    });

    // Data retention cleanup (daily)
    tokio::spawn(async move {
        daily_maintenance(db5).await;
    });

    log::info!("All monitors started");
}

async fn system_monitor(db: Arc<Mutex<Database>>) {
    let mut sys = System::new_all();
    let mut disks = Disks::new_with_refreshed_list();
    let mut networks = Networks::new_with_refreshed_list();
    let mut tick = interval(Duration::from_secs(5)); 

    loop {
        tick.tick().await;

        sys.refresh_cpu();
        sys.refresh_memory();
        disks.refresh();
        networks.refresh();

        let cpu_usage = sys.global_cpu_info().frequency() as f32;
        let ram_used = sys.used_memory() / 1024 / 1024;
        let ram_total = sys.total_memory() / 1024 / 1024;

        let (disk_used, disk_total) = disks.iter().fold((0.0f32, 0.0f32), |acc, d| {
            let total = d.total_space() as f32 / 1_073_741_824.0;
            let used = (d.total_space() - d.available_space()) as f32 / 1_073_741_824.0;
            (acc.0 + used, acc.1 + total)
        });

        let (rx, tx) = networks.iter().fold((0u64, 0u64), |acc, (_, n)| {
            (acc.0 + n.received() / 1024, acc.1 + n.transmitted() / 1024)
        });

        let snapshot = SystemSnapshot {
            id: Uuid::new_v4().to_string(),
            cpu_usage,
            ram_used_mb: ram_used,
            ram_total_mb: ram_total,
            disk_used_gb: disk_used,
            disk_total_gb: disk_total,
            network_rx_kb: rx,
            network_tx_kb: tx,
            battery_percent: None,  // Platform-specific
            battery_charging: None,
            timestamp: Utc::now(),
        };

        if let Ok(db) = db.try_lock() {
            if let Err(e) = db.insert_snapshot(&snapshot) {
                log::warn!("Failed to insert system snapshot: {}", e);
            }
        }
    }
}

async fn clipboard_monitor(db: Arc<Mutex<Database>>) {
    let mut last_content = String::new();
    let mut tick = interval(Duration::from_secs(1));

    loop {
        tick.tick().await;

        // Read clipboard via arboard (cross-platform)
        if let Ok(content) = read_clipboard_text() {
            if !content.is_empty() && content != last_content {
                last_content = content.clone();

                let entry = ClipboardEntry {
                    id: Uuid::new_v4().to_string(),
                    content: content.clone(),
                    content_type: "text".to_string(),
                    source_app: get_active_app_name(),
                    timestamp: Utc::now(),
                };

                // Also record in timeline
                let event = TimelineEvent {
                    id: Uuid::new_v4().to_string(),
                    event_type: "clipboard".to_string(),
                    title: truncate(&content, 80),
                    description: Some(content.clone()),
                    data: None,
                    app_name: get_active_app_name(),
                    url: None,
                    file_path: None,
                    screenshot_path: None,
                    tags: Some("clipboard".to_string()),
                    timestamp: Utc::now(),
                    duration_secs: None,
                };

                if let Ok(db) = db.try_lock() {
                    let _ = db.insert_clipboard(&entry);
                    let _ = db.insert_event(&event);
                }
            }
        }
    }
}

async fn daily_maintenance(db: Arc<Mutex<Database>>) {
    let mut tick = interval(Duration::from_secs(3600)); // Every hour
    loop {
        tick.tick().await;
        if let Ok(db) = db.try_lock() {
            let retention = db.get_settings()
                .map(|s| s.retention_days)
                .unwrap_or(30);
            if let Err(e) = db.prune_old_data(retention) {
                log::warn!("Maintenance error: {}", e);
            }
        }
    }
}

// ── Platform-specific helpers ─────────────────────────────────────────────────

#[cfg(target_os = "windows")]
fn read_clipboard_text() -> anyhow::Result<String> {
    use std::process::Command;
    use std::os::windows::process::CommandExt;
    let mut cmd = Command::new("powershell");
    cmd.args(["-NoProfile", "-Command", "Get-Clipboard"])
        .creation_flags(CREATE_NO_WINDOW);
    let output = cmd.output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(target_os = "macos")]
fn read_clipboard_text() -> anyhow::Result<String> {
    use std::process::Command;
    let output = Command::new("pbpaste").output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(target_os = "linux")]
fn read_clipboard_text() -> anyhow::Result<String> {
    use std::process::Command;
    let output = Command::new("xclip")
        .args(["-selection", "clipboard", "-o"])
        .output()
        .or_else(|_| Command::new("xsel").args(["--clipboard", "--output"]).output())?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(target_os = "windows")]
pub fn get_active_app_name() -> Option<String> {
    use std::process::Command;
    use std::os::windows::process::CommandExt;
    let mut cmd = Command::new("powershell");
    cmd.args(["-NoProfile", "-Command",
        "(Get-Process | Where-Object {$_.MainWindowHandle -eq (Add-Type -PassThru -TypeDefinition 'using System;using System.Runtime.InteropServices;public class U{[DllImport(\"user32.dll\")]public static extern IntPtr GetForegroundWindow();}')::GetForegroundWindow()}).Name | Select-Object -First 1"
    ])
    .creation_flags(CREATE_NO_WINDOW);
    let output = cmd.output().ok()?;
    let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if name.is_empty() { None } else { Some(name) }
}

#[cfg(target_os = "macos")]
pub fn get_active_app_name() -> Option<String> {
    use std::process::Command;
    let output = Command::new("osascript")
        .args(["-e", "tell application \"System Events\" to get name of first process whose frontmost is true"])
        .output().ok()?;
    let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if name.is_empty() { None } else { Some(name) }
}

#[cfg(target_os = "linux")]
pub fn get_active_app_name() -> Option<String> {
    use std::process::Command;
    let output = Command::new("xdotool")
        .args(["getactivewindow", "getwindowname"])
        .output().ok()?;
    let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if name.is_empty() { None } else { Some(name) }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..s.char_indices().nth(max).map(|(i, _)| i).unwrap_or(s.len())])
    }
}
