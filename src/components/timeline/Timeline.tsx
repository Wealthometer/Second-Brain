import React, { useEffect, useState, useCallback } from "react";
import { useStore } from "../../store";
import { api, TimelineEvent } from "../../utils/api";
import { formatDistanceToNow, format } from "date-fns";
import {
  Clipboard, Globe, FolderOpen, Monitor, Camera, Code,
  RefreshCw, Filter, Trash2, ChevronDown
} from "lucide-react";
import styles from "./Timeline.module.css";

const EVENT_ICONS: Record<string, React.ReactNode> = {
  clipboard: <Clipboard size={13} />,
  app_switch: <Monitor size={13} />,
  file_created: <FolderOpen size={13} />,
  file_modified: <FolderOpen size={13} />,
  file_deleted: <FolderOpen size={13} />,
  file_opened: <FolderOpen size={13} />,
  screenshot: <Camera size={13} />,
  browser: <Globe size={13} />,
};

const EVENT_COLORS: Record<string, string> = {
  clipboard: "var(--accent)",
  app_switch: "var(--accent2)",
  file_created: "var(--color-productivity)",
  file_modified: "var(--warn)",
  file_deleted: "var(--danger)",
  screenshot: "var(--color-design)",
  browser: "var(--color-browser)",
};

const EVENT_FILTERS = [
  { label: "All", value: undefined },
  { label: "Clipboard", value: "clipboard" },
  { label: "Apps", value: "app_switch" },
  { label: "Files", value: "file_created" },
  { label: "Screenshots", value: "screenshot" },
];

export default function Timeline() {
  const { timelineEvents, setTimelineEvents, timelineLoading, setTimelineLoading } = useStore();
  const [filter, setFilter] = useState<string | undefined>(undefined);
  const [offset, setOffset] = useState(0);
  const [hasMore, setHasMore] = useState(true);
  const LIMIT = 40;

  const load = useCallback(async (reset = false) => {
    setTimelineLoading(true);
    try {
      const off = reset ? 0 : offset;
      const events = await api.getTimeline(LIMIT, off, filter);
      if (reset) {
        setTimelineEvents(events);
        setOffset(LIMIT);
      } else {
        setTimelineEvents([...timelineEvents, ...events]);
        setOffset(off + LIMIT);
      }
      setHasMore(events.length === LIMIT);
    } catch {}
    setTimelineLoading(false);
  }, [filter, offset, timelineEvents]);

  useEffect(() => {
    load(true);
    const interval = setInterval(() => load(true), 10000);
    return () => clearInterval(interval);
  }, [filter]);

  const handleDelete = async (id: string) => {
    await api.deleteEvent(id).catch(() => {});
    setTimelineEvents(timelineEvents.filter(e => e.id !== id));
  };

  // Group events by day
  const grouped = groupByDay(timelineEvents);

  return (
    <div className={styles.timeline}>
      {/* Filter bar */}
      <div className={styles.filterBar}>
        {EVENT_FILTERS.map(f => (
          <button
            key={f.label}
            className={`${styles.filterBtn} ${filter === f.value ? styles.active : ""}`}
            onClick={() => { setFilter(f.value); setOffset(0); }}
          >
            {f.label}
          </button>
        ))}
        <button className={styles.refreshBtn} onClick={() => load(true)}>
          <RefreshCw size={13} />
        </button>
      </div>

      {/* Events feed */}
      {Object.entries(grouped).map(([day, events]) => (
        <div key={day} className={styles.dayGroup}>
          <div className={styles.dayHeader}>
            <span className={styles.dayLabel}>{day}</span>
            <div className={styles.dayLine} />
            <span className={styles.dayCount}>{events.length} events</span>
          </div>

          <div className={styles.eventList}>
            {events.map((event, i) => (
              <EventCard key={event.id} event={event} onDelete={handleDelete} />
            ))}
          </div>
        </div>
      ))}

      {timelineEvents.length === 0 && !timelineLoading && (
        <div className={styles.emptyState}>
          <div className={styles.emptyIcon}><Monitor size={32} /></div>
          <div className={styles.emptyTitle}>Your memory is empty</div>
          <div className={styles.emptyBody}>Second Brain is running. Activity will appear here automatically as you work.</div>
        </div>
      )}

      {hasMore && (
        <button className={styles.loadMore} onClick={() => load(false)} disabled={timelineLoading}>
          <ChevronDown size={14} />
          {timelineLoading ? "Loading..." : "Load more"}
        </button>
      )}
    </div>
  );
}

function EventCard({ event, onDelete }: { event: TimelineEvent; onDelete: (id: string) => void }) {
  const color = EVENT_COLORS[event.event_type] || "var(--text-muted)";
  const icon = EVENT_ICONS[event.event_type] || <Code size={13} />;

  return (
    <div className={styles.eventCard}>
      <div className={styles.eventDot} style={{ background: color }} />
      <div className={styles.eventIconWrap} style={{ color }}>
        {icon}
      </div>
      <div className={styles.eventBody}>
        <div className={styles.eventTitle}>{event.title}</div>
        {event.description && event.description !== event.title && (
          <div className={styles.eventDesc}>{truncate(event.description, 120)}</div>
        )}
        <div className={styles.eventMeta}>
          {event.app_name && <span className={styles.metaTag}>{event.app_name}</span>}
          {event.url && <span className={styles.metaTag} title={event.url}>{getDomain(event.url)}</span>}
          {event.duration_secs && event.duration_secs > 0 && (
            <span className={styles.metaDuration}>{formatDur(event.duration_secs)}</span>
          )}
          <span className={styles.metaTime}>
            {formatDistanceToNow(new Date(event.timestamp), { addSuffix: true })}
          </span>
        </div>
      </div>
      <button className={styles.deleteBtn} onClick={() => onDelete(event.id)} title="Remove from memory">
        <Trash2 size={11} />
      </button>
    </div>
  );
}

function groupByDay(events: TimelineEvent[]): Record<string, TimelineEvent[]> {
  const groups: Record<string, TimelineEvent[]> = {};
  for (const event of events) {
    const d = new Date(event.timestamp);
    const dayLabel = isToday(d) ? "Today" : isYesterday(d) ? "Yesterday" : format(d, "EEEE, MMMM d");
    if (!groups[dayLabel]) groups[dayLabel] = [];
    groups[dayLabel].push(event);
  }
  return groups;
}

function isToday(d: Date) {
  const now = new Date();
  return d.getDate() === now.getDate() && d.getMonth() === now.getMonth() && d.getFullYear() === now.getFullYear();
}

function isYesterday(d: Date) {
  const yest = new Date();
  yest.setDate(yest.getDate() - 1);
  return d.getDate() === yest.getDate() && d.getMonth() === yest.getMonth() && d.getFullYear() === yest.getFullYear();
}

function getDomain(url: string) {
  try { return new URL(url).hostname.replace("www.", ""); } catch { return url; }
}

function truncate(s: string, max: number) {
  return s.length <= max ? s : s.slice(0, max) + "…";
}

function formatDur(secs: number) {
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  if (m > 0) return `${m}m ${s}s`;
  return `${s}s`;
}
