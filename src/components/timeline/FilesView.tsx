import React, { useEffect, useState } from "react";
import { api, TimelineEvent } from "../../utils/api";
import { FolderOpen, FileText, Trash2, Plus } from "lucide-react";
import { formatDistanceToNow } from "date-fns";

const TYPE_COLORS: Record<string, string> = {
  file_created: "var(--color-productivity)",
  file_modified: "var(--warn)",
  file_deleted: "var(--danger)",
  file_opened: "var(--accent2)",
};

const TYPE_ICONS: Record<string, React.ReactNode> = {
  file_created: <Plus size={12} />,
  file_modified: <FileText size={12} />,
  file_deleted: <Trash2 size={12} />,
  file_opened: <FolderOpen size={12} />,
};

export default function FilesView() {
  const [events, setEvents] = useState<TimelineEvent[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    api.getFileEvents(100).then(setEvents).catch(() => {}).finally(() => setLoading(false));
  }, []);

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 8, maxWidth: 800 }}>
      <div style={{ display: "flex", alignItems: "center", gap: 8, fontSize: 13, color: "var(--text-secondary)", paddingBottom: 8, borderBottom: "1px solid var(--border-dim)" }}>
        <FolderOpen size={16} />
        <span>{events.length} file events</span>
      </div>
      {loading && <p style={{ color: "var(--text-muted)", fontSize: 13 }}>Loading file activity...</p>}
      {events.map(e => (
        <div key={e.id} style={{ display: "flex", alignItems: "center", gap: 10, background: "var(--bg-raised)", border: "1px solid var(--border-dim)", borderRadius: "var(--radius-sm)", padding: "10px 12px" }}>
          <span style={{ color: TYPE_COLORS[e.event_type] || "var(--text-muted)" }}>
            {TYPE_ICONS[e.event_type] || <FileText size={12} />}
          </span>
          <div style={{ flex: 1, minWidth: 0 }}>
            <div style={{ fontSize: 13, color: "var(--text-primary)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{e.title}</div>
            {e.file_path && <div style={{ fontSize: 10, color: "var(--text-muted)", fontFamily: "var(--font-mono)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", marginTop: 2 }}>{e.file_path}</div>}
          </div>
          <span style={{ fontSize: 10, color: "var(--text-ghost)", fontFamily: "var(--font-mono)", whiteSpace: "nowrap" }}>
            {formatDistanceToNow(new Date(e.timestamp), { addSuffix: true })}
          </span>
        </div>
      ))}
      {!loading && events.length === 0 && (
        <div style={{ color: "var(--text-muted)", padding: 40, textAlign: "center" }}>
          File activity will appear here as you create, modify, and open files in your home directories.
        </div>
      )}
    </div>
  );
}
