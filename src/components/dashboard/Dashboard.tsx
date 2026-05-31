import React, { useEffect, useState } from "react";
import { useStore } from "../../store";
import { api, AppUsageEntry } from "../../utils/api";
import { PieChart, Pie, Cell, Tooltip, AreaChart, Area, XAxis, YAxis, ResponsiveContainer } from "recharts";
import { Activity, Clock, Cpu, Database, Globe, Clipboard, Zap, TrendingUp, Monitor } from "lucide-react";
import styles from "./Dashboard.module.css";

const CATEGORY_COLORS: Record<string, string> = {
  development: "#4da6ff",
  browser: "#a78bfa",
  communication: "#fb923c",
  productivity: "#3dffa0",
  entertainment: "#f472b6",
  terminal: "#22d3ee",
  design: "#fbbf24",
  other: "#6b7280",
};

function formatDuration(secs: number) {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m`;
}

function formatBytes(mb: number) {
  if (mb > 1024) return `${(mb / 1024).toFixed(1)} GB`;
  return `${mb.toFixed(0)} MB`;
}

export default function Dashboard() {
  const { systemStats, productivitySummary } = useStore();
  const [appUsage, setAppUsage] = useState<{ date: string; total_minutes: number; entries: AppUsageEntry[] } | null>(null);
  const [memStats, setMemStats] = useState<{ total_events: number; db_size_mb: number } | null>(null);
  const [cpuHistory, setCpuHistory] = useState<{ t: string; v: number }[]>([]);

  useEffect(() => {
    api.getAppUsage().then(setAppUsage).catch(() => {});
    api.getMemoryStats().then(setMemStats).catch(() => {});
  }, []);

  // Build CPU history from live stats
  useEffect(() => {
    if (systemStats) {
      setCpuHistory(prev => {
        const now = new Date().toLocaleTimeString("en", { hour: "2-digit", minute: "2-digit", second: "2-digit" });
        const next = [...prev, { t: now, v: systemStats.cpu_usage }];
        return next.slice(-20);
      });
    }
  }, [systemStats]);

  // Category pie data
  const categoryData = productivitySummary?.today.by_category
    ? Object.entries(productivitySummary.today.by_category)
        .map(([name, secs]) => ({ name, value: Math.round(secs / 60) }))
        .filter(d => d.value > 0)
        .sort((a, b) => b.value - a.value)
    : [];

  const topApps = appUsage?.entries.slice(0, 7) || [];

  return (
    <div className={styles.dashboard}>
      {/* KPI Row */}
      <div className={styles.kpiRow}>
        <KPICard
          icon={<Clock size={16} />}
          label="Active Today"
          value={appUsage ? formatDuration(appUsage.total_minutes * 60) : "—"}
          accent="accent"
        />
        <KPICard
          icon={<Database size={16} />}
          label="Memories Stored"
          value={memStats ? memStats.total_events.toLocaleString() : "—"}
          accent="blue"
        />
        <KPICard
          icon={<Cpu size={16} />}
          label="CPU Usage"
          value={systemStats ? `${systemStats.cpu_usage.toFixed(1)}%` : "—"}
          accent={systemStats && systemStats.cpu_usage > 80 ? "danger" : "accent"}
        />
        <KPICard
          icon={<Activity size={16} />}
          label="RAM Used"
          value={systemStats ? formatBytes(systemStats.ram_used_mb) : "—"}
          accent="blue"
        />
        <KPICard
          icon={<Monitor size={16} />}
          label="Processes"
          value={systemStats ? systemStats.process_count.toString() : "—"}
          accent="neutral"
        />
        <KPICard
          icon={<Database size={16} />}
          label="DB Size"
          value={memStats ? `${memStats.db_size_mb.toFixed(1)} MB` : "—"}
          accent="neutral"
        />
      </div>

      {/* Main grid */}
      <div className={styles.mainGrid}>
        {/* CPU chart */}
        <div className={styles.card}>
          <div className={styles.cardHeader}>
            <Cpu size={14} className={styles.cardIcon} />
            <span className={styles.cardTitle}>CPU — Live</span>
            <span className={styles.cardBadge}>{systemStats?.cpu_usage.toFixed(1)}%</span>
          </div>
          <div className={styles.chartWrap}>
            <ResponsiveContainer width="100%" height={120}>
              <AreaChart data={cpuHistory} margin={{ top: 5, right: 5, bottom: 5, left: -30 }}>
                <defs>
                  <linearGradient id="cpuGrad" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#3dffa0" stopOpacity={0.3} />
                    <stop offset="95%" stopColor="#3dffa0" stopOpacity={0} />
                  </linearGradient>
                </defs>
                <XAxis dataKey="t" hide />
                <YAxis domain={[0, 100]} tick={{ fontSize: 9, fill: "#4a5568" }} />
                <Tooltip
                  contentStyle={{ background: "var(--bg-elevated)", border: "1px solid var(--border-subtle)", borderRadius: 6, fontSize: 11 }}
                  formatter={(v: number) => [`${v.toFixed(1)}%`, "CPU"]}
                  labelStyle={{ display: "none" }}
                />
                <Area type="monotone" dataKey="v" stroke="#3dffa0" strokeWidth={1.5} fill="url(#cpuGrad)" />
              </AreaChart>
            </ResponsiveContainer>
          </div>

          {/* Top processes */}
          <div className={styles.processTable}>
            {systemStats?.top_processes.slice(0, 4).map((proc, i) => (
              <div key={i} className={styles.processRow}>
                <span className={styles.processName}>{proc.name}</span>
                <div className={styles.processBars}>
                  <div className={styles.procBarTrack}>
                    <div className={styles.procBarFill} style={{ width: `${Math.min(proc.cpu_percent, 100)}%`, background: "#3dffa0" }} />
                  </div>
                  <span className={styles.procVal}>{proc.cpu_percent.toFixed(1)}%</span>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* RAM + Disk */}
        <div className={styles.card}>
          <div className={styles.cardHeader}>
            <Activity size={14} className={styles.cardIcon} />
            <span className={styles.cardTitle}>System Resources</span>
          </div>
          {systemStats && (
            <div className={styles.resourceMetrics}>
              <ResourceMeter
                label="RAM"
                used={systemStats.ram_used_mb}
                total={systemStats.ram_total_mb}
                unit="MB"
                color="#4da6ff"
                percent={systemStats.ram_percent}
              />
              <ResourceMeter
                label="Disk"
                used={systemStats.disk_used_gb * 1024}
                total={systemStats.disk_total_gb * 1024}
                unit="GB"
                color="#a78bfa"
                percent={systemStats.disk_percent}
                divisor={1024}
              />
              <div className={styles.netStats}>
                <div className={styles.netStat}>
                  <span className={styles.netLabel}>↓ Download</span>
                  <span className={styles.netVal} style={{ color: "#3dffa0" }}>{formatBytes(systemStats.network_rx_kb / 1024)} / poll</span>
                </div>
                <div className={styles.netStat}>
                  <span className={styles.netLabel}>↑ Upload</span>
                  <span className={styles.netVal} style={{ color: "#4da6ff" }}>{formatBytes(systemStats.network_tx_kb / 1024)} / poll</span>
                </div>
                <div className={styles.netStat}>
                  <span className={styles.netLabel}>Uptime</span>
                  <span className={styles.netVal}>{formatDuration(systemStats.uptime_secs)}</span>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* App usage + pie */}
        <div className={`${styles.card} ${styles.cardWide}`}>
          <div className={styles.cardHeader}>
            <TrendingUp size={14} className={styles.cardIcon} />
            <span className={styles.cardTitle}>Time by Category</span>
            <span className={styles.cardSub}>Today</span>
          </div>
          <div className={styles.usageLayout}>
            {categoryData.length > 0 ? (
              <>
                <div className={styles.pieWrap}>
                  <ResponsiveContainer width={160} height={160}>
                    <PieChart>
                      <Pie data={categoryData} cx="50%" cy="50%" innerRadius={45} outerRadius={75} paddingAngle={2} dataKey="value">
                        {categoryData.map((entry) => (
                          <Cell key={entry.name} fill={CATEGORY_COLORS[entry.name] || CATEGORY_COLORS.other} />
                        ))}
                      </Pie>
                      <Tooltip
                        contentStyle={{ background: "var(--bg-elevated)", border: "1px solid var(--border-subtle)", borderRadius: 6, fontSize: 11 }}
                        formatter={(v: number) => [`${v}m`, ""]}
                      />
                    </PieChart>
                  </ResponsiveContainer>
                  <div className={styles.pieLegend}>
                    {categoryData.slice(0, 5).map((d) => (
                      <div key={d.name} className={styles.legendItem}>
                        <span className={styles.legendDot} style={{ background: CATEGORY_COLORS[d.name] || "#6b7280" }} />
                        <span className={styles.legendName}>{d.name}</span>
                        <span className={styles.legendVal}>{d.value}m</span>
                      </div>
                    ))}
                  </div>
                </div>

                {/* Top apps list */}
                <div className={styles.appList}>
                  {topApps.map((app, i) => (
                    <div key={app.id} className={styles.appRow}>
                      <div className={styles.appRank}>{i + 1}</div>
                      <div className={styles.appInfo}>
                        <span className={styles.appName}>{app.app_name}</span>
                        <div className={styles.appBarTrack}>
                          <div
                            className={styles.appBarFill}
                            style={{
                              width: `${(app.duration_secs / (topApps[0]?.duration_secs || 1)) * 100}%`,
                              background: CATEGORY_COLORS[app.category || "other"] || "#6b7280"
                            }}
                          />
                        </div>
                      </div>
                      <span className={styles.appDuration}>{formatDuration(app.duration_secs)}</span>
                    </div>
                  ))}
                  {topApps.length === 0 && (
                    <div className={styles.empty}>No app usage data yet. Start using apps!</div>
                  )}
                </div>
              </>
            ) : (
              <div className={styles.empty}>Activity data will appear as you work...</div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

function KPICard({ icon, label, value, accent }: { icon: React.ReactNode; label: string; value: string; accent: string }) {
  const colors: Record<string, string> = {
    accent: "var(--accent)",
    blue: "var(--accent2)",
    danger: "var(--danger)",
    warn: "var(--warn)",
    neutral: "var(--text-secondary)",
  };
  return (
    <div className={styles.kpiCard}>
      <div className={styles.kpiIcon} style={{ color: colors[accent] }}>{icon}</div>
      <div className={styles.kpiBody}>
        <div className={styles.kpiValue} style={{ color: colors[accent] }}>{value}</div>
        <div className={styles.kpiLabel}>{label}</div>
      </div>
    </div>
  );
}

function ResourceMeter({ label, used, total, unit, color, percent, divisor = 1 }: any) {
  return (
    <div className={styles.resourceMeter}>
      <div className={styles.resourceLabel}>
        <span>{label}</span>
        <span style={{ color }}>
          {divisor > 1 ? `${(used / divisor).toFixed(1)} / ${(total / divisor).toFixed(1)} ${unit}` : `${used.toFixed(0)} / ${total.toFixed(0)} ${unit}`}
        </span>
      </div>
      <div className={styles.resourceTrack}>
        <div className={styles.resourceFill} style={{ width: `${Math.min(percent, 100)}%`, background: color }} />
      </div>
      <span className={styles.resourcePct}>{percent.toFixed(1)}%</span>
    </div>
  );
}
