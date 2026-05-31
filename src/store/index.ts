import { create } from "zustand";
import { TimelineEvent, SystemStats, AppUsageEntry, ClipboardEntry, AppSettings, ProductivitySummary } from "../utils/api";

interface AppStore {
  // Active view
  activeView: "timeline" | "search" | "dashboard" | "clipboard" | "files" | "browser" | "insights" | "settings";
  setActiveView: (view: AppStore["activeView"]) => void;

  // Search
  searchQuery: string;
  setSearchQuery: (q: string) => void;
  searchResults: TimelineEvent[];
  setSearchResults: (r: TimelineEvent[]) => void;
  isSearching: boolean;
  setIsSearching: (v: boolean) => void;

  // System stats (live)
  systemStats: SystemStats | null;
  setSystemStats: (s: SystemStats) => void;

  // Timeline
  timelineEvents: TimelineEvent[];
  setTimelineEvents: (e: TimelineEvent[]) => void;
  timelineLoading: boolean;
  setTimelineLoading: (v: boolean) => void;

  // Clipboard
  clipboardHistory: ClipboardEntry[];
  setClipboardHistory: (c: ClipboardEntry[]) => void;

  // App usage
  appUsage: { date: string; total_minutes: number; entries: AppUsageEntry[] } | null;
  setAppUsage: (a: AppStore["appUsage"]) => void;

  // Productivity
  productivitySummary: ProductivitySummary | null;
  setProductivitySummary: (p: ProductivitySummary) => void;

  // Settings
  settings: AppSettings | null;
  setSettings: (s: AppSettings) => void;

  // UI state
  sidebarCollapsed: boolean;
  toggleSidebar: () => void;
  
  activeWindow: { app_name?: string; window_title?: string } | null;
  setActiveWindow: (w: AppStore["activeWindow"]) => void;
}

export const useStore = create<AppStore>((set) => ({
  activeView: "dashboard",
  setActiveView: (activeView) => set({ activeView }),

  searchQuery: "",
  setSearchQuery: (searchQuery) => set({ searchQuery }),
  searchResults: [],
  setSearchResults: (searchResults) => set({ searchResults }),
  isSearching: false,
  setIsSearching: (isSearching) => set({ isSearching }),

  systemStats: null,
  setSystemStats: (systemStats) => set({ systemStats }),

  timelineEvents: [],
  setTimelineEvents: (timelineEvents) => set({ timelineEvents }),
  timelineLoading: false,
  setTimelineLoading: (timelineLoading) => set({ timelineLoading }),

  clipboardHistory: [],
  setClipboardHistory: (clipboardHistory) => set({ clipboardHistory }),

  appUsage: null,
  setAppUsage: (appUsage) => set({ appUsage }),

  productivitySummary: null,
  setProductivitySummary: (productivitySummary) => set({ productivitySummary }),

  settings: null,
  setSettings: (settings) => set({ settings }),

  sidebarCollapsed: false,
  toggleSidebar: () => set((s) => ({ sidebarCollapsed: !s.sidebarCollapsed })),
  
  activeWindow: null,
  setActiveWindow: (activeWindow) => set({ activeWindow }),
}));
