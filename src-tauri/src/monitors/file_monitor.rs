use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::Mutex;
use notify::{Watcher, RecursiveMode, Event, EventKind};
use uuid::Uuid;
use chrono::Utc;

use crate::db::{Database, TimelineEvent};

pub async fn start_file_watcher(db: Arc<Mutex<Database>>) -> anyhow::Result<()> {
    let (tx, mut rx) = tokio::sync::mpsc::channel(200);

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        if let Ok(event) = res {
            let _ = tx.blocking_send(event);
        }
    })?;

    // Watch common user directories
    let watch_paths = get_watch_paths();
    for path in &watch_paths {
        if path.exists() {
            if let Err(e) = watcher.watch(path, RecursiveMode::NonRecursive) {
                log::warn!("Cannot watch {:?}: {}", path, e);
            }
        }
    }

    log::info!("File watcher active on {} paths", watch_paths.len());

    while let Some(event) = rx.recv().await {
        if let Some(timeline_event) = event_to_timeline(&event) {
            if let Ok(db) = db.try_lock() {
                let _ = db.insert_event(&timeline_event);
            }
        }
    }

    // Keep watcher alive
    drop(watcher);
    Ok(())
}

fn get_watch_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(home) = dirs::home_dir() {
        for dir in &["Downloads", "Documents", "Desktop", "Pictures", "Videos", "Music"] {
            paths.push(home.join(dir));
        }
    }

    paths
}

fn event_to_timeline(event: &Event) -> Option<TimelineEvent> {
    let path = event.paths.first()?;
    let path_str = path.to_string_lossy().to_string();

    // Skip hidden files, temp files, system files
    let file_name = path.file_name()?.to_string_lossy();
    if file_name.starts_with('.') || file_name.ends_with('~') || file_name.contains(".tmp") {
        return None;
    }

    // Skip directories for modify events (too noisy)
    if path.is_dir() {
        return None;
    }

    let (event_type, title) = match &event.kind {
        EventKind::Create(_) => ("file_created", format!("Created: {}", file_name)),
        EventKind::Modify(_) => ("file_modified", format!("Modified: {}", file_name)),
        EventKind::Remove(_) => ("file_deleted", format!("Deleted: {}", file_name)),
        EventKind::Access(_) => ("file_opened", format!("Opened: {}", file_name)),
        _ => return None,
    };

    let ext = path.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    Some(TimelineEvent {
        id: Uuid::new_v4().to_string(),
        event_type: event_type.to_string(),
        title,
        description: Some(path_str.clone()),
        data: None,
        app_name: None,
        url: None,
        file_path: Some(path_str),
        screenshot_path: None,
        tags: Some(format!("file,{}", ext)),
        timestamp: Utc::now(),
        duration_secs: None,
    })
}
