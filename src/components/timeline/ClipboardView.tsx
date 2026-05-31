import React, { useEffect, useState } from "react";
import { api, ClipboardEntry } from "../../utils/api";
import { Clipboard, Copy, Clock } from "lucide-react";
import { formatDistanceToNow } from "date-fns";
import styles from "./ClipboardView.module.css";

export default function ClipboardView() {
  const [entries, setEntries] = useState<ClipboardEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [copied, setCopied] = useState<string | null>(null);

  useEffect(() => {
    api.getClipboardHistory(200).then(setEntries).catch(() => {}).finally(() => setLoading(false));
  }, []);

  const copyToClipboard = async (text: string, id: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(id);
      setTimeout(() => setCopied(null), 1500);
    } catch {}
  };

  return (
    <div className={styles.view}>
      <div className={styles.header}>
        <Clipboard size={16} />
        <span>{entries.length} clipboard entries</span>
      </div>
      {loading && <div className={styles.loading}>Loading clipboard history...</div>}
      <div className={styles.list}>
        {entries.map(entry => (
          <div key={entry.id} className={styles.entry}>
            <div className={styles.entryContent}>{entry.content}</div>
            <div className={styles.entryMeta}>
              {entry.source_app && <span className={styles.app}>{entry.source_app}</span>}
              <span className={styles.time}>
                <Clock size={10} /> {formatDistanceToNow(new Date(entry.timestamp), { addSuffix: true })}
              </span>
              <button className={styles.copyBtn} onClick={() => copyToClipboard(entry.content, entry.id)}>
                {copied === entry.id ? "✓ Copied" : <><Copy size={11} /> Copy</>}
              </button>
            </div>
          </div>
        ))}
        {!loading && entries.length === 0 && (
          <div className={styles.empty}>No clipboard history yet. Copy some text to get started.</div>
        )}
      </div>
    </div>
  );
}
