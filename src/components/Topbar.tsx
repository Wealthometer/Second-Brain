import React, { useState, useRef, useEffect } from "react";
import { Search, Command, Bell, Zap } from "lucide-react";
import { useStore } from "../store";
import { api } from "../utils/api";
import styles from "./Topbar.module.css";
import { format } from "date-fns";

export default function Topbar() {
  const {
    activeView, setActiveView,
    searchQuery, setSearchQuery,
    setSearchResults, setIsSearching
  } = useStore();

  const [time, setTime] = useState(new Date());
  const searchRef = useRef<HTMLInputElement>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout>>();

  useEffect(() => {
    const t = setInterval(() => setTime(new Date()), 1000);
    return () => clearInterval(t);
  }, []);

  // Keyboard shortcut: Cmd/Ctrl+K
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        searchRef.current?.focus();
        setActiveView("search");
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  const handleSearch = (value: string) => {
    setSearchQuery(value);
    if (value.trim()) {
      setActiveView("search");
      clearTimeout(debounceRef.current);
      setIsSearching(true);
      debounceRef.current = setTimeout(async () => {
        try {
          const results = await api.searchEvents(value);
          setSearchResults(results);
        } catch {}
        setIsSearching(false);
      }, 300);
    }
  };

  const VIEW_TITLES: Record<string, string> = {
    dashboard: "Dashboard",
    timeline: "Activity Timeline",
    search: "Search Memory",
    clipboard: "Clipboard History",
    browser: "Browser History",
    files: "File Activity",
    insights: "AI Insights",
    settings: "Settings",
  };

  return (
    <header className={styles.topbar}>
      <div className={styles.left}>
        <h1 className={styles.viewTitle}>{VIEW_TITLES[activeView] || activeView}</h1>
      </div>

      {/* Search */}
      <div className={styles.searchWrap}>
        <Search size={14} className={styles.searchIcon} />
        <input
          ref={searchRef}
          className={styles.searchInput}
          placeholder="Search your entire digital memory..."
          value={searchQuery}
          onChange={(e) => handleSearch(e.target.value)}
          onFocus={() => setActiveView("search")}
        />
        <div className={styles.searchShortcut}>
          <Command size={11} />
          <span>K</span>
        </div>
      </div>

      <div className={styles.right}>
        {/* Live memory indicator */}
        <div className={styles.memoryPill}>
          <Zap size={11} />
          <span>Memory Active</span>
        </div>

        {/* Clock */}
        <div className={styles.clock}>
          <span className={styles.clockTime}>{format(time, "HH:mm:ss")}</span>
          <span className={styles.clockDate}>{format(time, "MMM d")}</span>
        </div>
      </div>
    </header>
  );
}
