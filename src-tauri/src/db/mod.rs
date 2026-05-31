use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimelineEvent {
    pub id: String,
    pub event_type: String,
    pub title: String,
    pub description: Option<String>,
    pub data: Option<String>,        // JSON blob
    pub app_name: Option<String>,
    pub url: Option<String>,
    pub file_path: Option<String>,
    pub screenshot_path: Option<String>,
    pub tags: Option<String>,        // comma-separated
    pub timestamp: DateTime<Utc>,
    pub duration_secs: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClipboardEntry {
    pub id: String,
    pub content: String,
    pub content_type: String,        // text, image, file
    pub source_app: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppUsageEntry {
    pub id: String,
    pub app_name: String,
    pub window_title: Option<String>,
    pub duration_secs: i64,
    pub date: String,                // YYYY-MM-DD
    pub category: Option<String>,    // productivity, entertainment, etc.
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemSnapshot {
    pub id: String,
    pub cpu_usage: f32,
    pub ram_used_mb: u64,
    pub ram_total_mb: u64,
    pub disk_used_gb: f32,
    pub disk_total_gb: f32,
    pub network_rx_kb: u64,
    pub network_tx_kb: u64,
    pub battery_percent: Option<f32>,
    pub battery_charging: Option<bool>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BrowserTab {
    pub id: String,
    pub title: String,
    pub url: String,
    pub browser: String,
    pub visit_count: i32,
    pub first_visited: DateTime<Utc>,
    pub last_visited: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub capture_screenshots: bool,
    pub screenshot_interval_secs: i64,
    pub capture_clipboard: bool,
    pub monitor_files: bool,
    pub monitor_browser: bool,
    pub ai_insights_enabled: bool,
    pub openrouter_api_key: Option<String>,
    pub retention_days: i64,
    pub excluded_apps: String,
    pub excluded_paths: String,
    pub blur_sensitive: bool,
    pub assistant_volume: i64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            capture_screenshots: true,
            screenshot_interval_secs: 30,
            capture_clipboard: true,
            monitor_files: true,
            monitor_browser: true,
            ai_insights_enabled: false,
            openrouter_api_key: None,
            retention_days: 30,
            excluded_apps: "1Password,Keychain,Banking".to_string(),
            excluded_paths: "/private,/.ssh".to_string(),
            blur_sensitive: true,
            assistant_volume: 80,
        }
    }
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &Path) -> anyhow::Result<Self> {
        let conn = Connection::open(path)?;
        
        // Enable WAL mode for better concurrent performance
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA cache_size=10000;")?;
        
        let db = Self { conn };
        db.create_schema()?;
        Ok(db)
    }

    fn create_schema(&self) -> anyhow::Result<()> {
        self.conn.execute_batch("
            CREATE TABLE IF NOT EXISTS timeline_events (
                id TEXT PRIMARY KEY,
                event_type TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                data TEXT,
                app_name TEXT,
                url TEXT,
                file_path TEXT,
                screenshot_path TEXT,
                tags TEXT,
                timestamp INTEGER NOT NULL,
                duration_secs INTEGER
            );

            CREATE INDEX IF NOT EXISTS idx_events_timestamp ON timeline_events(timestamp DESC);
            CREATE INDEX IF NOT EXISTS idx_events_type ON timeline_events(event_type);
            CREATE INDEX IF NOT EXISTS idx_events_app ON timeline_events(app_name);
            CREATE VIRTUAL TABLE IF NOT EXISTS events_fts USING fts5(
                id UNINDEXED,
                title,
                description,
                app_name,
                url,
                tags,
                content=timeline_events,
                content_rowid=rowid
            );

            CREATE TABLE IF NOT EXISTS clipboard_history (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                content_type TEXT NOT NULL DEFAULT 'text',
                source_app TEXT,
                timestamp INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_clipboard_timestamp ON clipboard_history(timestamp DESC);

            CREATE TABLE IF NOT EXISTS app_usage (
                id TEXT PRIMARY KEY,
                app_name TEXT NOT NULL,
                window_title TEXT,
                duration_secs INTEGER NOT NULL,
                date TEXT NOT NULL,
                category TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_app_usage_date ON app_usage(date DESC);
            CREATE INDEX IF NOT EXISTS idx_app_usage_app ON app_usage(app_name);

            CREATE TABLE IF NOT EXISTS system_snapshots (
                id TEXT PRIMARY KEY,
                cpu_usage REAL,
                ram_used_mb INTEGER,
                ram_total_mb INTEGER,
                disk_used_gb REAL,
                disk_total_gb REAL,
                network_rx_kb INTEGER,
                network_tx_kb INTEGER,
                battery_percent REAL,
                battery_charging INTEGER,
                timestamp INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_snapshots_timestamp ON system_snapshots(timestamp DESC);

            CREATE TABLE IF NOT EXISTS browser_tabs (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                url TEXT NOT NULL,
                browser TEXT NOT NULL,
                visit_count INTEGER DEFAULT 1,
                first_visited INTEGER NOT NULL,
                last_visited INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_browser_url ON browser_tabs(url);
            CREATE INDEX IF NOT EXISTS idx_browser_last ON browser_tabs(last_visited DESC);

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
        ")?;

        // Trigger to keep FTS in sync
        self.conn.execute_batch("
            CREATE TRIGGER IF NOT EXISTS events_fts_insert AFTER INSERT ON timeline_events BEGIN
                INSERT INTO events_fts(rowid, id, title, description, app_name, url, tags)
                VALUES (new.rowid, new.id, new.title, new.description, new.app_name, new.url, new.tags);
            END;

            CREATE TRIGGER IF NOT EXISTS events_fts_delete AFTER DELETE ON timeline_events BEGIN
                INSERT INTO events_fts(events_fts, rowid, id, title, description, app_name, url, tags)
                VALUES ('delete', old.rowid, old.id, old.title, old.description, old.app_name, old.url, old.tags);
            END;
        ")?;

        Ok(())
    }

    // ── Timeline Events ──────────────────────────────────────────────────────

    pub fn insert_event(&self, event: &TimelineEvent) -> anyhow::Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO timeline_events 
             (id, event_type, title, description, data, app_name, url, file_path, screenshot_path, tags, timestamp, duration_secs)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                event.id,
                event.event_type,
                event.title,
                event.description,
                event.data,
                event.app_name,
                event.url,
                event.file_path,
                event.screenshot_path,
                event.tags,
                event.timestamp.timestamp(),
                event.duration_secs,
            ],
        )?;
        Ok(())
    }

    pub fn get_timeline(&self, limit: i64, offset: i64, event_type: Option<&str>) -> anyhow::Result<Vec<TimelineEvent>> {
        let query = if let Some(et) = event_type {
            format!(
                "SELECT * FROM timeline_events WHERE event_type = '{}' ORDER BY timestamp DESC LIMIT {} OFFSET {}",
                et, limit, offset
            )
        } else {
            format!(
                "SELECT * FROM timeline_events ORDER BY timestamp DESC LIMIT {} OFFSET {}",
                limit, offset
            )
        };

        let mut stmt = self.conn.prepare(&query)?;
        let events = stmt.query_map([], |row| {
            let ts: i64 = row.get(10)?;
            Ok(TimelineEvent {
                id: row.get(0)?,
                event_type: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                data: row.get(4)?,
                app_name: row.get(5)?,
                url: row.get(6)?,
                file_path: row.get(7)?,
                screenshot_path: row.get(8)?,
                tags: row.get(9)?,
                timestamp: DateTime::from_timestamp(ts, 0).unwrap_or_default(),
                duration_secs: row.get(11)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
        Ok(events)
    }

    pub fn search_events(&self, query: &str, limit: i64) -> anyhow::Result<Vec<TimelineEvent>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.* FROM timeline_events t
             JOIN events_fts f ON t.rowid = f.rowid
             WHERE events_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2"
        )?;
        let events = stmt.query_map(params![query, limit], |row| {
            let ts: i64 = row.get(10)?;
            Ok(TimelineEvent {
                id: row.get(0)?,
                event_type: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                data: row.get(4)?,
                app_name: row.get(5)?,
                url: row.get(6)?,
                file_path: row.get(7)?,
                screenshot_path: row.get(8)?,
                tags: row.get(9)?,
                timestamp: DateTime::from_timestamp(ts, 0).unwrap_or_default(),
                duration_secs: row.get(11)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
        Ok(events)
    }

    pub fn delete_event(&self, id: &str) -> anyhow::Result<()> {
        self.conn.execute("DELETE FROM timeline_events WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ── Clipboard ────────────────────────────────────────────────────────────

    pub fn insert_clipboard(&self, entry: &ClipboardEntry) -> anyhow::Result<()> {
        // Dedup: don't insert if same content was inserted in the last 5 seconds
        let recent: Option<i64> = self.conn.query_row(
            "SELECT timestamp FROM clipboard_history WHERE content = ?1 ORDER BY timestamp DESC LIMIT 1",
            params![entry.content],
            |row| row.get(0),
        ).ok();
        
        if let Some(ts) = recent {
            if entry.timestamp.timestamp() - ts < 5 {
                return Ok(());
            }
        }

        self.conn.execute(
            "INSERT INTO clipboard_history (id, content, content_type, source_app, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![entry.id, entry.content, entry.content_type, entry.source_app, entry.timestamp.timestamp()],
        )?;
        Ok(())
    }

    pub fn get_clipboard_history(&self, limit: i64) -> anyhow::Result<Vec<ClipboardEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content, content_type, source_app, timestamp FROM clipboard_history ORDER BY timestamp DESC LIMIT ?1"
        )?;
        let entries = stmt.query_map(params![limit], |row| {
            let ts: i64 = row.get(4)?;
            Ok(ClipboardEntry {
                id: row.get(0)?,
                content: row.get(1)?,
                content_type: row.get(2)?,
                source_app: row.get(3)?,
                timestamp: DateTime::from_timestamp(ts, 0).unwrap_or_default(),
            })
        })?
        .collect::<Result<Vec<_>>>()?;
        Ok(entries)
    }

    // ── App Usage ────────────────────────────────────────────────────────────

    pub fn upsert_app_usage(&self, entry: &AppUsageEntry) -> anyhow::Result<()> {
        self.conn.execute(
            "INSERT INTO app_usage (id, app_name, window_title, duration_secs, date, category) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET duration_secs = duration_secs + excluded.duration_secs",
            params![entry.id, entry.app_name, entry.window_title, entry.duration_secs, entry.date, entry.category],
        )?;
        Ok(())
    }

    pub fn get_app_usage_by_date(&self, date: &str) -> anyhow::Result<Vec<AppUsageEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, app_name, window_title, SUM(duration_secs) as total, date, category
             FROM app_usage WHERE date = ?1
             GROUP BY app_name ORDER BY total DESC"
        )?;
        let entries = stmt.query_map(params![date], |row| {
            Ok(AppUsageEntry {
                id: row.get(0)?,
                app_name: row.get(1)?,
                window_title: row.get(2)?,
                duration_secs: row.get(3)?,
                date: row.get(4)?,
                category: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
        Ok(entries)
    }

    // ── System Snapshots ─────────────────────────────────────────────────────

    pub fn insert_snapshot(&self, snap: &SystemSnapshot) -> anyhow::Result<()> {
        self.conn.execute(
            "INSERT INTO system_snapshots 
             (id, cpu_usage, ram_used_mb, ram_total_mb, disk_used_gb, disk_total_gb, network_rx_kb, network_tx_kb, battery_percent, battery_charging, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                snap.id, snap.cpu_usage, snap.ram_used_mb, snap.ram_total_mb,
                snap.disk_used_gb, snap.disk_total_gb, snap.network_rx_kb, snap.network_tx_kb,
                snap.battery_percent, snap.battery_charging.map(|b| b as i64),
                snap.timestamp.timestamp()
            ],
        )?;
        Ok(())
    }

    pub fn get_recent_snapshots(&self, limit: i64) -> anyhow::Result<Vec<SystemSnapshot>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM system_snapshots ORDER BY timestamp DESC LIMIT ?1"
        )?;
        let snaps = stmt.query_map(params![limit], |row| {
            let ts: i64 = row.get(10)?;
            let charging: Option<i64> = row.get(9)?;
            Ok(SystemSnapshot {
                id: row.get(0)?,
                cpu_usage: row.get(1)?,
                ram_used_mb: row.get(2)?,
                ram_total_mb: row.get(3)?,
                disk_used_gb: row.get(4)?,
                disk_total_gb: row.get(5)?,
                network_rx_kb: row.get(6)?,
                network_tx_kb: row.get(7)?,
                battery_percent: row.get(8)?,
                battery_charging: charging.map(|c| c != 0),
                timestamp: DateTime::from_timestamp(ts, 0).unwrap_or_default(),
            })
        })?
        .collect::<Result<Vec<_>>>()?;
        Ok(snaps)
    }

    // ── Settings ─────────────────────────────────────────────────────────────

    pub fn get_settings(&self) -> anyhow::Result<Settings> {
        let mut settings = Settings::default();
        let mut stmt = self.conn.prepare("SELECT key, value FROM settings")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        for row in rows {
            let (key, value) = row?;
            match key.as_str() {
                "capture_screenshots" => settings.capture_screenshots = value == "true",
                "screenshot_interval_secs" => settings.screenshot_interval_secs = value.parse().unwrap_or(30),
                "capture_clipboard" => settings.capture_clipboard = value == "true",
                "monitor_files" => settings.monitor_files = value == "true",
                "monitor_browser" => settings.monitor_browser = value == "true",
                "ai_insights_enabled" => settings.ai_insights_enabled = value == "true",
                "openrouter_api_key" => settings.openrouter_api_key = if value.is_empty() { None } else { Some(value) },
                "retention_days" => settings.retention_days = value.parse().unwrap_or(30),
                "excluded_apps" => settings.excluded_apps = value,
                "excluded_paths" => settings.excluded_paths = value,
                "blur_sensitive" => settings.blur_sensitive = value == "true",
                _ => {}
            }
        }
        Ok(settings)
    }

    pub fn save_settings(&self, settings: &Settings) -> anyhow::Result<()> {
        let pairs = vec![
            ("capture_screenshots", settings.capture_screenshots.to_string()),
            ("screenshot_interval_secs", settings.screenshot_interval_secs.to_string()),
            ("capture_clipboard", settings.capture_clipboard.to_string()),
            ("monitor_files", settings.monitor_files.to_string()),
            ("monitor_browser", settings.monitor_browser.to_string()),
            ("ai_insights_enabled", settings.ai_insights_enabled.to_string()),
            ("openrouter_api_key", settings.openrouter_api_key.clone().unwrap_or_default()),
            ("retention_days", settings.retention_days.to_string()),
            ("excluded_apps", settings.excluded_apps.clone()),
            ("excluded_paths", settings.excluded_paths.clone()),
            ("blur_sensitive", settings.blur_sensitive.to_string()),
        ];

        for (key, value) in pairs {
            self.conn.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
                params![key, value],
            )?;
        }
        Ok(())
    }

    // ── Maintenance ──────────────────────────────────────────────────────────

    pub fn prune_old_data(&self, retention_days: i64) -> anyhow::Result<()> {
        let cutoff = Utc::now().timestamp() - (retention_days * 86400);
        self.conn.execute("DELETE FROM timeline_events WHERE timestamp < ?1", params![cutoff])?;
        self.conn.execute("DELETE FROM clipboard_history WHERE timestamp < ?1", params![cutoff])?;
        self.conn.execute("DELETE FROM system_snapshots WHERE timestamp < ?1", params![cutoff])?;
        self.conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        Ok(())
    }

    pub fn get_total_event_count(&self) -> anyhow::Result<i64> {
        Ok(self.conn.query_row("SELECT COUNT(*) FROM timeline_events", [], |r| r.get(0))?)
    }

    pub fn get_db_size_mb(&self) -> anyhow::Result<f64> {
        let page_count: i64 = self.conn.query_row("PRAGMA page_count", [], |r| r.get(0))?;
        let page_size: i64 = self.conn.query_row("PRAGMA page_size", [], |r| r.get(0))?;
        Ok((page_count * page_size) as f64 / 1_048_576.0)
    }
}
