import React from "react";
import { useStore } from "../../store";
import { TimelineEvent } from "../../utils/api";
import { Search, Clock, FolderOpen, Globe, Clipboard, Monitor, Zap } from "lucide-react";
import { formatDistanceToNow } from "date-fns";
import styles from "./SearchView.module.css";

const TYPE_ICONS: Record<string, React.ReactNode> = {
  clipboard: <Clipboard size={12} />,
  app_switch: <Monitor size={12} />,
  file_created: <FolderOpen size={12} />,
  file_modified: <FolderOpen size={12} />,
  browser: <Globe size={12} />,
};

const TYPE_COLORS: Record<string, string> = {
  clipboard: "var(--accent)",
  app_switch: "var(--accent2)",
  file_created: "var(--color-productivity)",
  browser: "var(--color-browser)",
};

export default function SearchView() {
  const { searchQuery, searchResults, isSearching } = useStore();

  if (!searchQuery) {
    return (
      <div className={styles.emptySearch}>
        <div className={styles.emptyIcon}><Search size={40} /></div>
        <div className={styles.emptyTitle}>Search your entire digital memory</div>
        <div className={styles.emptyHints}>
          <div className={styles.hint}><Clipboard size={13} /><span>Clipboard content you've copied</span></div>
          <div className={styles.hint}><FolderOpen size={13} /><span>Files you've opened or created</span></div>
          <div className={styles.hint}><Globe size={13} /><span>Websites and tabs you visited</span></div>
          <div className={styles.hint}><Monitor size={13} /><span>Apps you used and for how long</span></div>
        </div>
        <div className={styles.shortcut}>Press <kbd>⌘K</kbd> to focus search anytime</div>
      </div>
    );
  }

  return (
    <div className={styles.searchView}>
      <div className={styles.resultsHeader}>
        {isSearching ? (
          <span className={styles.searching}><Zap size={13} /> Searching memory...</span>
        ) : (
          <span className={styles.resultCount}>
            {searchResults.length === 0 ? "No results" : `${searchResults.length} result${searchResults.length !== 1 ? "s" : ""}`} for <strong>"{searchQuery}"</strong>
          </span>
        )}
      </div>

      {!isSearching && searchResults.length === 0 && searchQuery && (
        <div className={styles.noResults}>
          <Search size={28} />
          <div>Nothing found for "{searchQuery}"</div>
          <div className={styles.noResultsSub}>Try a different keyword or browse the timeline</div>
        </div>
      )}

      <div className={styles.resultsList}>
        {searchResults.map(event => (
          <SearchResult key={event.id} event={event} query={searchQuery} />
        ))}
      </div>
    </div>
  );
}

function SearchResult({ event, query }: { event: TimelineEvent; query: string }) {
  const color = TYPE_COLORS[event.event_type] || "var(--text-muted)";
  const icon = TYPE_ICONS[event.event_type] || <Clock size={12} />;

  return (
    <div className={styles.result}>
      <div className={styles.resultIcon} style={{ color }}>{icon}</div>
      <div className={styles.resultBody}>
        <div className={styles.resultTitle} dangerouslySetInnerHTML={{ __html: highlight(event.title, query) }} />
        {event.description && (
          <div className={styles.resultDesc} dangerouslySetInnerHTML={{ __html: highlight(truncate(event.description, 200), query) }} />
        )}
        <div className={styles.resultMeta}>
          <span className={styles.resultType}>{event.event_type.replace("_", " ")}</span>
          {event.app_name && <span className={styles.resultApp}>{event.app_name}</span>}
          <span className={styles.resultTime}>{formatDistanceToNow(new Date(event.timestamp), { addSuffix: true })}</span>
        </div>
      </div>
    </div>
  );
}

function highlight(text: string, query: string): string {
  if (!query.trim()) return text;
  const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  return text.replace(new RegExp(`(${escaped})`, "gi"), '<mark style="background:var(--accent-glow);color:var(--accent);border-radius:2px;padding:0 2px;">$1</mark>');
}

function truncate(s: string, max: number) {
  return s.length <= max ? s : s.slice(0, max) + "…";
}
