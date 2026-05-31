// Browser history is read directly from browser SQLite databases
// This is safe, local-only, and doesn't require any extensions

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BrowserVisit {
    pub title: String,
    pub url: String,
    pub browser: String,
    pub visit_time: DateTime<Utc>,
    pub visit_count: i64,
}

/// Get browser history database paths per OS and browser
pub fn get_browser_history_paths() -> Vec<(String, PathBuf)> {
    let mut paths = Vec::new();
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return paths,
    };

    #[cfg(target_os = "windows")]
    {
        let appdata = home.join("AppData").join("Local");
        paths.push(("Chrome".into(), appdata.join("Google/Chrome/User Data/Default/History")));
        paths.push(("Edge".into(), appdata.join("Microsoft/Edge/User Data/Default/History")));
        paths.push(("Brave".into(), appdata.join("BraveSoftware/Brave-Browser/User Data/Default/History")));
        paths.push(("Firefox".into(), find_firefox_profile(&home.join("AppData/Roaming/Mozilla/Firefox/Profiles"))));
    }

    #[cfg(target_os = "macos")]
    {
        let lib = home.join("Library");
        paths.push(("Chrome".into(), lib.join("Application Support/Google/Chrome/Default/History")));
        paths.push(("Safari".into(), lib.join("Safari/History.db")));
        paths.push(("Firefox".into(), find_firefox_profile(&lib.join("Application Support/Firefox/Profiles"))));
        paths.push(("Brave".into(), lib.join("Application Support/BraveSoftware/Brave-Browser/Default/History")));
        paths.push(("Arc".into(), lib.join("Application Support/Arc/User Data/Default/History")));
    }

    #[cfg(target_os = "linux")]
    {
        let config = home.join(".config");
        paths.push(("Chrome".into(), config.join("google-chrome/Default/History")));
        paths.push(("Chromium".into(), config.join("chromium/Default/History")));
        paths.push(("Brave".into(), config.join("BraveSoftware/Brave-Browser/Default/History")));
        paths.push(("Firefox".into(), find_firefox_profile(&home.join(".mozilla/firefox"))));
    }

    paths.into_iter().filter(|(_, p)| p.exists()).collect()
}

fn find_firefox_profile(profiles_dir: &PathBuf) -> PathBuf {
    if let Ok(entries) = std::fs::read_dir(profiles_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let history = path.join("places.sqlite");
                if history.exists() {
                    return history;
                }
            }
        }
    }
    profiles_dir.join("nonexistent")
}

/// Read Chrome/Edge/Brave history (Chromium-based)
/// Note: browser must be closed or we use a temp copy
pub fn read_chromium_history(db_path: &PathBuf, browser: &str, since_timestamp: i64) -> Vec<BrowserVisit> {
    // Copy to temp to avoid lock conflicts
    let tmp = std::env::temp_dir().join(format!("sb_history_{}.db", browser));
    if std::fs::copy(db_path, &tmp).is_err() {
        return Vec::new();
    }

    let conn = match rusqlite::Connection::open(&tmp) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    // Chrome timestamps are microseconds since 1601-01-01
    // Convert our Unix timestamp to Chrome timestamp
    let chrome_epoch_offset: i64 = 11_644_473_600_000_000; // microseconds
    let chrome_since = since_timestamp * 1_000_000 + chrome_epoch_offset;

    let query = "
        SELECT urls.title, urls.url, urls.visit_count, visits.visit_time
        FROM visits
        JOIN urls ON visits.url = urls.id
        WHERE visits.visit_time > ?1
        ORDER BY visits.visit_time DESC
        LIMIT 500
    ";

    let mut stmt = match conn.prepare(query) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let visits: Vec<BrowserVisit> = stmt.query_map(rusqlite::params![chrome_since], |row| {
        let chrome_ts: i64 = row.get(3)?;
        let unix_ts = (chrome_ts - chrome_epoch_offset) / 1_000_000;
        Ok(BrowserVisit {
            title: row.get(0)?,
            url: row.get(1)?,
            visit_count: row.get(2)?,
            browser: browser.to_string(),
            visit_time: chrono::DateTime::from_timestamp(unix_ts, 0).unwrap_or_default(),
        })
    })
    .ok()
    .map(|rows| rows.flatten().collect())
    .unwrap_or_default();

    let _ = std::fs::remove_file(tmp);
    visits
}

/// Read Firefox history
pub fn read_firefox_history(db_path: &PathBuf, since_timestamp: i64) -> Vec<BrowserVisit> {
    let tmp = std::env::temp_dir().join("sb_firefox_history.db");
    if std::fs::copy(db_path, &tmp).is_err() {
        return Vec::new();
    }

    let conn = match rusqlite::Connection::open(&tmp) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    // Firefox timestamps are microseconds since Unix epoch
    let ff_since = since_timestamp * 1_000_000;

    let query = "
        SELECT p.title, p.url, p.visit_count, h.visit_date
        FROM moz_historyvisits h
        JOIN moz_places p ON h.place_id = p.id
        WHERE h.visit_date > ?1
        ORDER BY h.visit_date DESC
        LIMIT 500
    ";

    let mut stmt = match conn.prepare(query) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let visits: Vec<BrowserVisit> = stmt.query_map(rusqlite::params![ff_since], |row| {
        let ff_ts: i64 = row.get(3)?;
        let unix_ts = ff_ts / 1_000_000;
        Ok(BrowserVisit {
            title: row.get::<_, Option<String>>(0)?.unwrap_or_else(|| row.get(1).unwrap_or_default()),
            url: row.get(1)?,
            visit_count: row.get(2)?,
            browser: "Firefox".to_string(),
            visit_time: chrono::DateTime::from_timestamp(unix_ts, 0).unwrap_or_default(),
        })
    })
    .ok()
    .map(|rows| rows.flatten().collect())
    .unwrap_or_default();

    let _ = std::fs::remove_file(tmp);
    visits
}
