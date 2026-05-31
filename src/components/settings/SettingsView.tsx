import React, { useEffect, useState } from "react";
import { api, AppSettings } from "../../utils/api";
import { Settings, Shield, Clock, Brain, HardDrive, Save, RefreshCw, AlertCircle, CheckCircle2 } from "lucide-react";
import styles from "./SettingsView.module.css";

export default function SettingsView() {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<"capture" | "ai" | "privacy" | "storage">("capture");

  useEffect(() => {
    api.getSettings()
      .then(setSettings)
      .catch(() => setError("Failed to load settings"))
      .finally(() => setLoading(false));
  }, []);

  const save = async () => {
    if (!settings) return;
    setSaving(true);
    setError(null);
    try {
      await api.updateSettings(settings);
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (e: any) {
      setError(e.toString());
    }
    setSaving(false);
  };

  const set = <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => {
    if (!settings) return;
    setSettings({ ...settings, [key]: value });
  };

  if (loading) return <div className={styles.loading}>Loading settings...</div>;
  if (!settings) return <div className={styles.error}>Failed to load settings</div>;

  return (
    <div className={styles.settingsPage}>
      <div className={styles.pageHeader}>
        <Settings size={20} style={{ color: "var(--accent)" }} />
        <div>
          <div className={styles.pageTitle}>Settings</div>
          <div className={styles.pageSub}>Configure what Second Brain monitors and remembers</div>
        </div>
        <button className={`${styles.saveBtn} ${saved ? styles.saveBtnSuccess : ""}`} onClick={save} disabled={saving}>
          {saved ? <><CheckCircle2 size={14} /> Saved!</> : saving ? <><RefreshCw size={14} /> Saving...</> : <><Save size={14} /> Save Changes</>}
        </button>
      </div>

      {error && (
        <div className={styles.errorBanner}>
          <AlertCircle size={14} />
          {error}
        </div>
      )}

      {/* Tab nav */}
      <div className={styles.tabs}>
        {[
          { id: "capture", label: "Capture", icon: <Brain size={13} /> },
          { id: "ai", label: "AI & Assistant", icon: <Brain size={13} /> },
          { id: "privacy", label: "Privacy", icon: <Shield size={13} /> },
          { id: "storage", label: "Storage", icon: <HardDrive size={13} /> },
        ].map(tab => (
          <button
            key={tab.id}
            className={`${styles.tab} ${activeTab === tab.id ? styles.tabActive : ""}`}
            onClick={() => setActiveTab(tab.id as any)}
          >
            {tab.icon}
            {tab.label}
          </button>
        ))}
      </div>

      <div className={styles.tabContent}>

        {/* ── Capture tab ─────────────────────────────────────────────────── */}
        {activeTab === "capture" && (
          <div className={styles.section}>
            <SectionTitle icon={<Brain size={15} />} title="Activity Capture" sub="Choose what Second Brain records" />

            <ToggleRow
              label="Clipboard History"
              sub="Record everything you copy"
              checked={settings.capture_clipboard}
              onChange={v => set("capture_clipboard", v)}
            />
            <ToggleRow
              label="File Activity"
              sub="Track files created, modified, and opened in your home directories"
              checked={settings.monitor_files}
              onChange={v => set("monitor_files", v)}
            />
            <ToggleRow
              label="Browser History"
              sub="Read browser history from Chrome, Firefox, Edge, Brave, Arc"
              checked={settings.monitor_browser}
              onChange={v => set("monitor_browser", v)}
            />
            <ToggleRow
              label="Screenshots"
              sub="Periodic screen captures for visual memory"
              checked={settings.capture_screenshots}
              onChange={v => set("capture_screenshots", v)}
            />
            {settings.capture_screenshots && (
              <SliderRow
                label="Screenshot interval"
                value={settings.screenshot_interval_secs}
                min={10}
                max={300}
                step={10}
                format={v => `${v}s`}
                onChange={v => set("screenshot_interval_secs", v)}
              />
            )}

            <div className={styles.divider} />
            <SectionTitle icon={<Clock size={15} />} title="App Usage Tracking" sub="Monitor which apps you use and for how long" />
            <div className={styles.infoBox}>
              <div className={styles.infoBoxText}>
                Active window tracking runs automatically. Apps are categorized into: development, browser, communication, productivity, entertainment, terminal, design.
              </div>
            </div>
          </div>
        )}

        {/* ── AI tab ──────────────────────────────────────────────────────── */}
        {activeTab === "ai" && (
          <div className={styles.section}>
            <SectionTitle icon={<Brain size={15} />} title="AI Insights" sub="Cloud-based activity analysis (OpenRouter)" />

            <ToggleRow
              label="Enable AI Insights"
              sub="Analyze your productivity and generate smart summaries"
              checked={settings.ai_insights_enabled}
              onChange={v => set("ai_insights_enabled", v)}
            />

            {settings.ai_insights_enabled && (
              <div className={styles.field}>
                <label className={styles.fieldLabel}>OpenRouter API Key</label>
                <input
                  className={styles.fieldInput}
                  type="password"
                  placeholder="sk-or-..."
                  value={settings.openrouter_api_key || ""}
                  onChange={e => set("openrouter_api_key", e.target.value)}
                />
                <div className={styles.fieldNote}>
                  Get a free key at <a href="https://openrouter.ai" target="_blank" rel="noopener" className={styles.link}>openrouter.ai</a>. Many models have a free tier.
                </div>
              </div>
            )}

            <div className={styles.divider} />
            <SectionTitle icon={<Brain size={15} />} title="Voice Audio" sub="Adjust text-to-speech volume" />
            <SliderRow
              label="Voice Volume"
              value={settings.assistant_volume}
              min={0}
              max={100}
              step={5}
              format={v => `${v}%`}
              onChange={v => set("assistant_volume", v)}
            />

            <div className={styles.divider} />
            <SectionTitle icon={<Brain size={15} />} title="Voice Assistant (Aria)" sub="Configure via the floating Aria button (bottom-right)" />
            <div className={styles.infoBox}>
              <div className={styles.infoBoxText}>
                Aria supports two AI modes:
                <br /><strong style={{ color: "var(--accent)" }}>🏠 Ollama (Local)</strong> — Runs entirely on your machine. Install Ollama, pull a model like <code>llama3</code> or <code>mistral</code>, and Aria works offline with zero data leaving your PC.
                <br /><strong style={{ color: "var(--accent2)" }}>☁️ OpenRouter (Cloud)</strong> — Use GPT-4o-mini, Claude Haiku, or any model via API. Faster but requires internet.
                <br /><br />TTS (text-to-speech) uses the OS built-in engine by default — no install needed.
                <br /><br />Volume control works best on Windows. On macOS and Linux, volume may be controlled by the system volume or have limited effectiveness depending on the TTS engine.
              </div>
            </div>
          </div>
        )}

        {/* ── Privacy tab ─────────────────────────────────────────────────── */}
        {activeTab === "privacy" && (
          <div className={styles.section}>
            <SectionTitle icon={<Shield size={15} />} title="Privacy Controls" sub="All data is stored locally. Nothing is uploaded." />

            <ToggleRow
              label="Blur sensitive screenshots"
              sub="Detect and blur password fields and private content in captures"
              checked={settings.blur_sensitive}
              onChange={v => set("blur_sensitive", v)}
            />

            <div className={styles.field}>
              <label className={styles.fieldLabel}>Excluded Apps (never recorded)</label>
              <textarea
                className={styles.fieldTextarea}
                value={settings.excluded_apps}
                onChange={e => set("excluded_apps", e.target.value)}
                rows={3}
                placeholder="1Password, Keychain, Banking"
              />
              <div className={styles.fieldNote}>Comma-separated app names. These apps will be excluded from all monitoring.</div>
            </div>

            <div className={styles.field}>
              <label className={styles.fieldLabel}>Excluded File Paths (never recorded)</label>
              <textarea
                className={styles.fieldTextarea}
                value={settings.excluded_paths}
                onChange={e => set("excluded_paths", e.target.value)}
                rows={3}
                placeholder="/private, /.ssh, /Documents/Private"
              />
              <div className={styles.fieldNote}>Comma-separated path prefixes. File events under these paths are ignored.</div>
            </div>

            <div className={styles.privacyCard}>
              <Shield size={18} style={{ color: "var(--accent)" }} />
              <div>
                <div className={styles.privacyTitle}>100% Local & Private</div>
                <div className={styles.privacySub}>Your activity data never leaves your device. The database is stored in your local app data directory. AI features only send what you explicitly ask about.</div>
              </div>
            </div>
          </div>
        )}

        {/* ── Storage tab ─────────────────────────────────────────────────── */}
        {activeTab === "storage" && (
          <div className={styles.section}>
            <SectionTitle icon={<HardDrive size={15} />} title="Data Retention" sub="Manage how long Second Brain keeps your history" />

            <SliderRow
              label="Keep history for"
              value={settings.retention_days}
              min={1}
              max={365}
              step={1}
              format={v => `${v} days`}
              onChange={v => set("retention_days", v)}
            />

            <div className={styles.dangerZone}>
              <div className={styles.dangerTitle}>Danger Zone</div>
              <div className={styles.dangerRow}>
                <div>
                  <div className={styles.dangerLabel}>Clear all history</div>
                  <div className={styles.dangerSub}>Permanently delete all recorded activity</div>
                </div>
                <button className={styles.dangerBtn} onClick={async () => {
                  if (confirm("Delete ALL history? This cannot be undone.")) {
                    await api.clearHistory(0);
                  }
                }}>
                  Delete All
                </button>
              </div>
              <div className={styles.dangerRow}>
                <div>
                  <div className={styles.dangerLabel}>Export data</div>
                  <div className={styles.dangerSub}>Download your full history as JSON</div>
                </div>
                <button className={styles.exportBtn} onClick={async () => {
                  const path = await api.exportData();
                  alert(`Exported to: ${path}`);
                }}>
                  Export JSON
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

// ── Sub-components ────────────────────────────────────────────────────────────

function SectionTitle({ icon, title, sub }: { icon: React.ReactNode; title: string; sub: string }) {
  return (
    <div style={{ marginBottom: 16 }}>
      <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 4 }}>
        <span style={{ color: "var(--accent)" }}>{icon}</span>
        <span style={{ fontFamily: "var(--font-display)", fontWeight: 600, fontSize: 15, color: "var(--text-primary)" }}>{title}</span>
      </div>
      <div style={{ fontSize: 12, color: "var(--text-muted)", marginLeft: 23 }}>{sub}</div>
    </div>
  );
}

function ToggleRow({ label, sub, checked, onChange }: {
  label: string; sub: string; checked: boolean; onChange: (v: boolean) => void;
}) {
  return (
    <div className={styles.toggleRow}>
      <div className={styles.toggleInfo}>
        <div className={styles.toggleLabel}>{label}</div>
        <div className={styles.toggleSub}>{sub}</div>
      </div>
      <label className={styles.toggle}>
        <input type="checkbox" checked={checked} onChange={e => onChange(e.target.checked)} />
        <span className={styles.toggleSlider} />
      </label>
    </div>
  );
}

function SliderRow({ label, value, min, max, step, format, onChange }: {
  label: string; value: number; min: number; max: number; step: number;
  format: (v: number) => string; onChange: (v: number) => void;
}) {
  return (
    <div className={styles.sliderRow}>
      <div className={styles.sliderHeader}>
        <span className={styles.sliderLabel}>{label}</span>
        <span className={styles.sliderValue}>{format(value)}</span>
      </div>
      <input
        type="range" min={min} max={max} step={step} value={value}
        onChange={e => onChange(Number(e.target.value))}
        className={styles.slider}
      />
    </div>
  );
}
