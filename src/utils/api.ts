import { invoke } from "@tauri-apps/api/core";

export interface TimelineEvent {
  id: string;
  event_type: string;
  title: string;
  description?: string;
  data?: string;
  app_name?: string;
  url?: string;
  file_path?: string;
  screenshot_path?: string;
  tags?: string;
  timestamp: string;
  duration_secs?: number;
}

export interface ClipboardEntry {
  id: string;
  content: string;
  content_type: string;
  source_app?: string;
  timestamp: string;
}

export interface AppUsageEntry {
  id: string;
  app_name: string;
  window_title?: string;
  duration_secs: number;
  date: string;
  category?: string;
}

export interface SystemStats {
  cpu_usage: number;
  ram_used_mb: number;
  ram_total_mb: number;
  ram_percent: number;
  disk_used_gb: number;
  disk_total_gb: number;
  disk_percent: number;
  network_rx_kb: number;
  network_tx_kb: number;
  battery_percent?: number;
  battery_charging?: boolean;
  uptime_secs: number;
  process_count: number;
  top_processes: ProcessInfo[];
}

export interface ProcessInfo {
  name: string;
  cpu_percent: number;
  ram_mb: number;
}

export interface AIInsight {
  insight_type: string;
  title: string;
  body: string;
  score?: number;
  suggestions: string[];
}

export interface AppSettings {
  capture_screenshots: boolean;
  screenshot_interval_secs: number;
  capture_clipboard: boolean;
  monitor_files: boolean;
  monitor_browser: boolean;
  ai_insights_enabled: boolean;
  openrouter_api_key?: string;
  retention_days: number;
  excluded_apps: string;
  excluded_paths: string;
  blur_sensitive: boolean;
  assistant_volume: number;  // 0-100 volume percentage
}

export interface BrowserVisit {
  title: string;
  url: string;
  browser: string;
  visit_time: string;
  visit_count: number;
}

export interface ProductivitySummary {
  today: {
    total_active_minutes: number;
    top_apps: AppUsageEntry[];
    by_category: Record<string, number>;
  };
  memory: {
    total_events: number;
    db_size_mb: number;
  };
}

// ── API calls ─────────────────────────────────────────────────────────────────

export const api = {
  getTimeline: (limit = 50, offset = 0, event_type?: string) =>
    invoke<TimelineEvent[]>("get_timeline", { limit, offset, eventType: event_type }),

  searchEvents: (query: string, limit = 30) =>
    invoke<TimelineEvent[]>("search_events", { query, limit }),

  deleteEvent: (id: string) =>
    invoke<void>("delete_event", { id }),

  getSystemStats: () =>
    invoke<SystemStats>("get_system_stats"),

  getAppUsage: (date?: string) =>
    invoke<{ date: string; total_minutes: number; entries: AppUsageEntry[] }>("get_app_usage", { date }),

  getProductivitySummary: () =>
    invoke<ProductivitySummary>("get_productivity_summary"),

  getAiInsights: () =>
    invoke<AIInsight[]>("get_ai_insights"),

  getClipboardHistory: (limit = 100) =>
    invoke<ClipboardEntry[]>("get_clipboard_history", { limit }),

  getScreenshotHistory: (limit = 20) =>
    invoke<TimelineEvent[]>("get_screenshot_history", { limit }),

  getFileEvents: (limit = 50) =>
    invoke<TimelineEvent[]>("get_file_events", { limit }),

  getBrowserHistory: (limit = 100) =>
    invoke<BrowserVisit[]>("get_browser_history", { limit }),

  getSettings: () =>
    invoke<AppSettings>("get_settings"),

  updateSettings: (settings: AppSettings) =>
    invoke<void>("update_settings", { settings }),

  exportData: () =>
    invoke<string>("export_data"),

  clearHistory: (days?: number) =>
    invoke<void>("clear_history", { days }),

  getMemoryStats: () =>
    invoke<{ total_events: number; db_size_mb: number; oldest_event_days: number }>("get_memory_stats"),

  getActiveWindow: () =>
    invoke<{ app_name?: string; window_title?: string; timestamp: number }>("get_active_window_now"),
};
