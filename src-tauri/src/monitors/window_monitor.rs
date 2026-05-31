use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use uuid::Uuid;
use chrono::Utc;

use crate::db::{Database, AppUsageEntry, TimelineEvent};
use super::get_active_app_name;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub struct WindowMonitor {
    pub current_app: Option<String>,
    pub current_title: Option<String>,
    pub session_start: chrono::DateTime<Utc>,
}

impl WindowMonitor {
    pub fn new() -> Self {
        Self {
            current_app: None,
            current_title: None,
            session_start: Utc::now(),
        }
    }
}

pub async fn active_window_monitor(db: Arc<Mutex<Database>>) {
    let mut monitor = WindowMonitor::new();
    let mut tick = interval(Duration::from_secs(2));

    loop {
        tick.tick().await;

        let active_app = get_active_app_name();
        let active_title = get_active_window_title();

        // Detect app switch
        if active_app != monitor.current_app {
            // Save previous session
            if let Some(prev_app) = &monitor.current_app {
                let duration = (Utc::now() - monitor.session_start).num_seconds();
                if duration > 2 {
                    let entry = AppUsageEntry {
                        id: format!("{}-{}", prev_app.replace(' ', "_"), Utc::now().date_naive()),
                        app_name: prev_app.clone(),
                        window_title: monitor.current_title.clone(),
                        duration_secs: duration,
                        date: Utc::now().date_naive().to_string(),
                        category: categorize_app(prev_app),
                    };

                    let event = TimelineEvent {
                        id: Uuid::new_v4().to_string(),
                        event_type: "app_switch".to_string(),
                        title: format!("Used {}", prev_app),
                        description: monitor.current_title.clone(),
                        data: None,
                        app_name: Some(prev_app.clone()),
                        url: None,
                        file_path: None,
                        screenshot_path: None,
                        tags: Some(format!("app,{}", categorize_app(prev_app).unwrap_or_default())),
                        timestamp: monitor.session_start,
                        duration_secs: Some(duration),
                    };

                    if let Ok(db) = db.try_lock() {
                        let _ = db.upsert_app_usage(&entry);
                        // Only log if session was > 10s to reduce noise
                        if duration > 10 {
                            let _ = db.insert_event(&event);
                        }
                    }
                }
            }

            monitor.current_app = active_app;
            monitor.current_title = active_title;
            monitor.session_start = Utc::now();
        }
    }
}

#[cfg(target_os = "windows")]
pub fn get_active_window_title() -> Option<String> {
    use std::process::Command;
    use std::os::windows::process::CommandExt;
    let mut cmd = Command::new("powershell");
    cmd.args(["-NoProfile", "-Command",
        "(Add-Type -PassThru -TypeDefinition 'using System;using System.Runtime.InteropServices;using System.Text;public class W{[DllImport(\"user32.dll\")]public static extern IntPtr GetForegroundWindow();[DllImport(\"user32.dll\")]public static extern int GetWindowText(IntPtr h,StringBuilder s,int m);}')::GetWindowText((Add-Type -PassThru -TypeDefinition 'using System;using System.Runtime.InteropServices;public class U{[DllImport(\"user32.dll\")]public static extern IntPtr GetForegroundWindow();}')::GetForegroundWindow(), ($b = New-Object System.Text.StringBuilder 256), 256) | Out-Null; $b.ToString()"
    ])
    .creation_flags(CREATE_NO_WINDOW);
    let output = cmd.output().ok()?;
    let title = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if title.is_empty() { None } else { Some(title) }
}

#[cfg(target_os = "macos")]
pub fn get_active_window_title() -> Option<String> {
    use std::process::Command;
    let output = Command::new("osascript")
        .args(["-e", "tell application \"System Events\" to get title of first window of (first process whose frontmost is true)"])
        .output().ok()?;
    let title = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if title.is_empty() || title.starts_with("execution error") { None } else { Some(title) }
}

#[cfg(target_os = "linux")]
pub fn get_active_window_title() -> Option<String> {
    use std::process::Command;
    let output = Command::new("xdotool")
        .args(["getactivewindow", "getwindowname"])
        .output().ok()?;
    let title = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if title.is_empty() { None } else { Some(title) }
}

pub fn categorize_app(app: &str) -> Option<String> {
    let app_lower = app.to_lowercase();
    let category = if ["code", "cursor", "vim", "nvim", "sublime", "atom", "webstorm", "intellij", "rider", "clion", "xcode"]
        .iter().any(|&a| app_lower.contains(a)) {
        "development"
    } else if ["chrome", "firefox", "safari", "edge", "brave", "opera", "vivaldi"]
        .iter().any(|&a| app_lower.contains(a)) {
        "browser"
    } else if ["slack", "discord", "teams", "zoom", "telegram", "whatsapp", "signal"]
        .iter().any(|&a| app_lower.contains(a)) {
        "communication"
    } else if ["word", "excel", "powerpoint", "notion", "obsidian", "evernote", "onenote", "docs"]
        .iter().any(|&a| app_lower.contains(a)) {
        "productivity"
    } else if ["spotify", "vlc", "youtube", "netflix", "steam", "games"]
        .iter().any(|&a| app_lower.contains(a)) {
        "entertainment"
    } else if ["terminal", "iterm", "wezterm", "alacritty", "cmd", "powershell", "bash"]
        .iter().any(|&a| app_lower.contains(a)) {
        "terminal"
    } else if ["figma", "photoshop", "sketch", "illustrator", "canva", "affinity"]
        .iter().any(|&a| app_lower.contains(a)) {
        "design"
    } else {
        "other"
    };
    Some(category.to_string())
}
