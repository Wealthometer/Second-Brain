import React, { useEffect, useState } from "react";
import { api, AIInsight } from "../../utils/api";
import { Sparkles, TrendingUp, AlertCircle, RefreshCw, Zap } from "lucide-react";

export default function InsightsView() {
  const [insights, setInsights] = useState<AIInsight[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const load = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await api.getAiInsights();
      setInsights(data);
      if (data.length === 0) setError("AI insights require an API key. Configure it in Settings → Assistant.");
    } catch (e: any) {
      setError(e.toString());
    }
    setLoading(false);
  };

  useEffect(() => { load(); }, []);

  return (
    <div style={{ maxWidth: 700, display: "flex", flexDirection: "column", gap: 20 }}>
      <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
        <Sparkles size={20} style={{ color: "var(--accent)" }} />
        <div>
          <div style={{ fontFamily: "var(--font-display)", fontSize: 18, fontWeight: 700, color: "var(--text-primary)" }}>AI Insights</div>
          <div style={{ fontSize: 12, color: "var(--text-muted)" }}>Powered by your configured LLM provider</div>
        </div>
        <button onClick={load} disabled={loading} style={{ marginLeft: "auto", display: "flex", alignItems: "center", gap: 5, padding: "6px 12px", borderRadius: "var(--radius-sm)", border: "1px solid var(--border-subtle)", background: "transparent", color: "var(--text-secondary)", fontSize: 12, cursor: "pointer" }}>
          <RefreshCw size={12} /> Refresh
        </button>
      </div>

      {loading && (
        <div style={{ display: "flex", alignItems: "center", gap: 10, color: "var(--accent)", fontFamily: "var(--font-mono)", fontSize: 13, padding: 20 }}>
          <Zap size={14} /> Generating insights...
        </div>
      )}

      {error && (
        <div style={{ display: "flex", gap: 10, padding: "14px 16px", background: "var(--warn-dim)", border: "1px solid rgba(255,179,71,0.25)", borderRadius: "var(--radius-md)", color: "var(--warn)" }}>
          <AlertCircle size={16} style={{ flexShrink: 0 }} />
          <div>
            <div style={{ fontWeight: 600, fontSize: 13, marginBottom: 4 }}>Insights unavailable</div>
            <div style={{ fontSize: 12 }}>{error}</div>
          </div>
        </div>
      )}

      {insights.map((insight, i) => (
        <InsightCard key={i} insight={insight} />
      ))}
    </div>
  );
}

function InsightCard({ insight }: { insight: AIInsight }) {
  return (
    <div style={{ background: "var(--bg-raised)", border: "1px solid var(--border-subtle)", borderRadius: "var(--radius-md)", padding: 20 }}>
      <div style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: 12 }}>
        <TrendingUp size={16} style={{ color: "var(--accent)" }} />
        <span style={{ fontFamily: "var(--font-display)", fontWeight: 600, fontSize: 15, color: "var(--text-primary)" }}>{insight.title}</span>
        {insight.score !== undefined && (
          <div style={{ marginLeft: "auto", fontFamily: "var(--font-mono)", fontSize: 20, fontWeight: 700, color: insight.score > 70 ? "var(--accent)" : insight.score > 40 ? "var(--warn)" : "var(--danger)" }}>
            {Math.round(insight.score)}
            <span style={{ fontSize: 11, color: "var(--text-muted)" }}>/100</span>
          </div>
        )}
      </div>
      <p style={{ fontSize: 13, color: "var(--text-secondary)", lineHeight: 1.7, marginBottom: 16 }}>{insight.body}</p>
      {insight.suggestions.length > 0 && (
        <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
          {insight.suggestions.map((s, i) => (
            <div key={i} style={{ display: "flex", gap: 8, alignItems: "flex-start", padding: "6px 10px", background: "var(--bg-elevated)", borderRadius: "var(--radius-sm)", fontSize: 12, color: "var(--text-secondary)" }}>
              <span style={{ color: "var(--accent)", fontWeight: 700, flexShrink: 0 }}>{i + 1}.</span>
              {s}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
