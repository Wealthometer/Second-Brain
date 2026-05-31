import React, { useEffect, useState } from "react";
import { api, BrowserVisit } from "../../utils/api";
import { Globe, ExternalLink, Clock } from "lucide-react";
import { formatDistanceToNow } from "date-fns";

export default function BrowserView() {
  const [visits, setVisits] = useState<BrowserVisit[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    api.getBrowserHistory(200).then(setVisits).catch(() => {}).finally(() => setLoading(false));
  }, []);

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 12, maxWidth: 800 }}>
      <div style={{ display: "flex", alignItems: "center", gap: 8, fontSize: 13, color: "var(--text-secondary)", paddingBottom: 8, borderBottom: "1px solid var(--border-dim)" }}>
        <Globe size={16} />
        <span>{visits.length} browser visits (last 24h)</span>
      </div>
      {loading && <p style={{ color: "var(--text-muted)", fontSize: 13 }}>Reading browser history...</p>}
      {visits.map((v, i) => (
        <div key={i} style={{ background: "var(--bg-raised)", border: "1px solid var(--border-dim)", borderRadius: "var(--radius-sm)", padding: "10px 14px" }}>
          <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
            <Globe size={13} style={{ color: "var(--color-browser)", flexShrink: 0 }} />
            <span style={{ fontSize: 13, color: "var(--text-primary)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{v.title || v.url}</span>
            <span style={{ marginLeft: "auto", fontSize: 10, color: "var(--text-ghost)", fontFamily: "var(--font-mono)", whiteSpace: "nowrap" }}>
              {formatDistanceToNow(new Date(v.visit_time), { addSuffix: true })}
            </span>
          </div>
          <div style={{ fontSize: 11, color: "var(--text-muted)", fontFamily: "var(--font-mono)", marginTop: 4, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{v.url}</div>
          <div style={{ display: "flex", gap: 8, marginTop: 6 }}>
            <span style={{ fontSize: 10, color: "var(--color-browser)", background: "var(--bg-overlay)", padding: "1px 6px", borderRadius: 3, border: "1px solid var(--border-dim)" }}>{v.browser}</span>
            {v.visit_count > 1 && <span style={{ fontSize: 10, color: "var(--text-muted)" }}>{v.visit_count}× visited</span>}
          </div>
        </div>
      ))}
      {!loading && visits.length === 0 && (
        <div style={{ color: "var(--text-muted)", padding: 40, textAlign: "center" }}>
          No browser history found. Make sure Chrome, Firefox, or Brave is installed.
        </div>
      )}
    </div>
  );
}
