import React from "react";
import { useStore } from "../store";
import styles from "./Sidebar.module.css";
import {
  Brain, LayoutDashboard, Clock, Search, Clipboard,
  Globe, FolderOpen, Sparkles, Settings, ChevronLeft,
  ChevronRight, Activity
} from "lucide-react";

const NAV_ITEMS = [
  { id: "dashboard", label: "Dashboard", icon: LayoutDashboard },
  { id: "timeline", label: "Timeline", icon: Clock },
  { id: "search", label: "Search", icon: Search },
  { id: "clipboard", label: "Clipboard", icon: Clipboard },
  { id: "browser", label: "Browser", icon: Globe },
  { id: "files", label: "Files", icon: FolderOpen },
  { id: "insights", label: "AI Insights", icon: Sparkles, accent: true },
] as const;

export default function Sidebar() {
  const { activeView, setActiveView, sidebarCollapsed, toggleSidebar, systemStats, activeWindow } = useStore();

  return (
    <aside className={`${styles.sidebar} ${sidebarCollapsed ? styles.collapsed : ""}`}>
      {/* Logo */}
      <div className={styles.logo}>
        <div className={styles.logoIcon}>
          <Brain size={20} />
        </div>
        {!sidebarCollapsed && (
          <div className={styles.logoText}>
            <span className={styles.logoName}>Second Brain</span>
            <span className={styles.logoSub}>Intelligence OS</span>
          </div>
        )}
      </div>

      {/* Live indicator */}
      {!sidebarCollapsed && (
        <div className={styles.liveIndicator}>
          <div className="pulse-dot" />
          <span>Recording activity</span>
        </div>
      )}

      {/* Active window */}
      {!sidebarCollapsed && activeWindow?.app_name && (
        <div className={styles.activeWindowPill}>
          <Activity size={10} />
          <span className={styles.activeWindowName}>{activeWindow.app_name}</span>
        </div>
      )}

      {/* Nav */}
      <nav className={styles.nav}>
        {NAV_ITEMS.map(({ id, label, icon: Icon, accent }) => (
          <button
            key={id}
            className={`${styles.navItem} ${activeView === id ? styles.active : ""} ${accent ? styles.accentItem : ""}`}
            onClick={() => setActiveView(id as any)}
            title={sidebarCollapsed ? label : undefined}
          >
            <Icon size={16} />
            {!sidebarCollapsed && <span>{label}</span>}
            {accent && !sidebarCollapsed && <span className={styles.aiTag}>AI</span>}
          </button>
        ))}
      </nav>

      {/* System vitals mini */}
      {!sidebarCollapsed && systemStats && (
        <div className={styles.vitals}>
          <div className={styles.vitalRow}>
            <span className={styles.vitalLabel}>CPU</span>
            <div className={styles.vitalBar}>
              <div
                className={styles.vitalFill}
                style={{
                  width: `${systemStats.cpu_usage}%`,
                  background: systemStats.cpu_usage > 80 ? "var(--danger)" : "var(--accent)"
                }}
              />
            </div>
            <span className={styles.vitalValue}>{systemStats.cpu_usage.toFixed(0)}%</span>
          </div>
          <div className={styles.vitalRow}>
            <span className={styles.vitalLabel}>RAM</span>
            <div className={styles.vitalBar}>
              <div
                className={styles.vitalFill}
                style={{
                  width: `${systemStats.ram_percent}%`,
                  background: systemStats.ram_percent > 85 ? "var(--warn)" : "var(--accent2)"
                }}
              />
            </div>
            <span className={styles.vitalValue}>{systemStats.ram_percent.toFixed(0)}%</span>
          </div>
          <div className={styles.vitalRow}>
            <span className={styles.vitalLabel}>Disk</span>
            <div className={styles.vitalBar}>
              <div
                className={styles.vitalFill}
                style={{
                  width: `${systemStats.disk_percent}%`,
                  background: "var(--color-browser)"
                }}
              />
            </div>
            <span className={styles.vitalValue}>{systemStats.disk_percent.toFixed(0)}%</span>
          </div>
        </div>
      )}

      {/* Settings + collapse */}
      <div className={styles.bottom}>
        <button
          className={`${styles.navItem} ${activeView === "settings" ? styles.active : ""}`}
          onClick={() => setActiveView("settings")}
          title={sidebarCollapsed ? "Settings" : undefined}
        >
          <Settings size={16} />
          {!sidebarCollapsed && <span>Settings</span>}
        </button>
        <button className={styles.collapseBtn} onClick={toggleSidebar} title="Toggle sidebar">
          {sidebarCollapsed ? <ChevronRight size={14} /> : <ChevronLeft size={14} />}
        </button>
      </div>
    </aside>
  );
}
