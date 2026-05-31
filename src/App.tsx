import React, { useEffect } from "react";
import Sidebar from "./components/Sidebar";
import Topbar from "./components/Topbar";
import Dashboard from "./components/dashboard/Dashboard";
import Timeline from "./components/timeline/Timeline";
import SearchView from "./components/search/SearchView";
import ClipboardView from "./components/timeline/ClipboardView";
import BrowserView from "./components/timeline/BrowserView";
import FilesView from "./components/timeline/FilesView";
import InsightsView from "./components/insights/InsightsView";
import SettingsView from "./components/settings/SettingsView";
import { useStore } from "./store";
import { api } from "./utils/api";
import VoiceAssistant from "./components/assistant/VoiceAssistant";
import styles from "./App.module.css";

export default function App() {
  const { activeView, setSystemStats, setActiveWindow, setProductivitySummary } = useStore();

  // Live system stats polling
  useEffect(() => {
    const pollStats = async () => {
      try {
        const stats = await api.getSystemStats();
        setSystemStats(stats);
      } catch (e) {
        // Silent fail during dev without Tauri
      }
    };

    pollStats();
    const interval = setInterval(pollStats, 3000);
    return () => clearInterval(interval);
  }, []);

  // Active window polling
  useEffect(() => {
    const pollWindow = async () => {
      try {
        const win = await api.getActiveWindow();
        setActiveWindow(win);
      } catch {}
    };
    const interval = setInterval(pollWindow, 2000);
    return () => clearInterval(interval);
  }, []);

  // Productivity summary
  useEffect(() => {
    const loadSummary = async () => {
      try {
        const summary = await api.getProductivitySummary();
        setProductivitySummary(summary);
      } catch {}
    };
    loadSummary();
    const interval = setInterval(loadSummary, 60000);
    return () => clearInterval(interval);
  }, []);

  const renderView = () => {
    switch (activeView) {
      case "dashboard": return <Dashboard />;
      case "timeline": return <Timeline />;
      case "search": return <SearchView />;
      case "clipboard": return <ClipboardView />;
      case "browser": return <BrowserView />;
      case "files": return <FilesView />;
      case "insights": return <InsightsView />;
      case "settings": return <SettingsView />;
      default: return <Dashboard />;
    }
  };

  return (
    <div className={styles.appShell}>
      <Sidebar />
      <div className={styles.mainArea}>
        <Topbar />
        <main className={styles.content}>
          {renderView()}
        </main>
      </div>
      {/* Global floating voice assistant — always present */}
      <VoiceAssistant />
    </div>
  );
}
