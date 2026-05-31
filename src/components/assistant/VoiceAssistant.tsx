import React, { useEffect, useRef, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import styles from "./VoiceAssistant.module.css";
import {
  Mic, MicOff, Volume2, VolumeX, Send, Bot, AlertTriangle,
  AlertCircle, Info, Cpu, MemoryStick, HardDrive, Thermometer,
  Battery, BatteryLow, X, ChevronDown, ChevronUp, Zap,
  Settings2, RefreshCw, CheckCircle2, Wifi, WifiOff
} from "lucide-react";

// ── Types ─────────────────────────────────────────────────────────────────────

interface Alert {
  id: string;
  severity: "info" | "warning" | "critical";
  category: string;
  title: string;
  message: string;
  spoken_text: string;
  suggestion?: string;
  timestamp: string;
  acknowledged: boolean;
}

interface ChatMessage {
  role: "user" | "assistant" | "system";
  content: string;
}

interface ProviderStatus {
  name: string;
  connected: boolean;
  model: string;
  error?: string;
}

interface AssistantState {
  is_speaking: boolean;
  is_listening: boolean;
  recent_alerts: Alert[];
  conversation: ChatMessage[];
  provider_status: ProviderStatus;
}

interface AssistantConfig {
  provider: string;
  ollama_model: string;
  ollama_url: string;
  openrouter_api_key?: string;
  openrouter_model: string;
  tts_engine: string;
  tts_voice: string;
  tts_rate: number;
  tts_enabled: boolean;
  stt_enabled: boolean;
  cpu_critical_pct: number;
  cpu_warn_pct: number;
  ram_critical_pct: number;
  ram_warn_pct: number;
  disk_critical_pct: number;
  temp_critical_c: number;
  summary_interval_mins: number;
  alert_cooldown_secs: number;
}

// ── Alert banner ──────────────────────────────────────────────────────────────

function AlertBanner({ alert, onDismiss }: { alert: Alert; onDismiss: () => void }) {
  const isCrit = alert.severity === "critical";
  const isWarn = alert.severity === "warning";

  const icon = alert.category.includes("cpu") ? <Cpu size={15} />
    : alert.category.includes("ram") ? <MemoryStick size={15} />
    : alert.category.includes("disk") ? <HardDrive size={15} />
    : alert.category.includes("temp") ? <Thermometer size={15} />
    : alert.category.includes("battery") ? <BatteryLow size={15} />
    : isCrit ? <AlertCircle size={15} /> : <AlertTriangle size={15} />;

  return (
    <div className={`${styles.alertBanner} ${styles[`alert_${alert.severity}`]}`}>
      <div className={styles.alertIcon}>{icon}</div>
      <div className={styles.alertBody}>
        <div className={styles.alertTitle}>{alert.title}</div>
        <div className={styles.alertMsg}>{alert.message}</div>
        {alert.suggestion && <div className={styles.alertSuggestion}>→ {alert.suggestion}</div>}
      </div>
      <button className={styles.alertDismiss} onClick={onDismiss} title="Dismiss">
        <X size={12} />
      </button>
    </div>
  );
}

// ── Provider status badge ─────────────────────────────────────────────────────

function ProviderBadge({ status }: { status: ProviderStatus | null }) {
  if (!status) return null;
  return (
    <div className={`${styles.providerBadge} ${status.connected ? styles.providerOk : styles.providerError}`}>
      {status.connected ? <Wifi size={11} /> : <WifiOff size={11} />}
      <span>{status.name}</span>
      {status.connected && <span className={styles.modelName}>{status.model}</span>}
      {!status.connected && status.error && (
        <span className={styles.providerErr}>{status.error}</span>
      )}
    </div>
  );
}

// ── Main component ────────────────────────────────────────────────────────────

export default function VoiceAssistant() {
  const [isOpen, setIsOpen] = useState(false);
  const [isMuted, setIsMuted] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [isSpeaking, setIsSpeaking] = useState(false);
  const [input, setInput] = useState("");
  const [messages, setMessages] = useState<ChatMessage[]>([
    {
      role: "assistant",
      content: "Hi! I'm Aria, your Second Brain assistant. I'm monitoring your system and will alert you to anything important. Ask me anything about your activity, productivity, or system health.",
    },
  ]);
  const [alerts, setAlerts] = useState<Alert[]>([]);
  const [providerStatus, setProviderStatus] = useState<ProviderStatus | null>(null);
  const [ollamaModels, setOllamaModels] = useState<string[]>([]);
  const [config, setConfig] = useState<AssistantConfig | null>(null);
  const [showConfig, setShowConfig] = useState(false);
  const [speakReplies, setSpeakReplies] = useState(true);

  const chatEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  // Auto-scroll chat
  useEffect(() => {
    chatEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  // Load initial state
  useEffect(() => {
    loadState();
  }, []);

  // Listen for backend alert events
  useEffect(() => {
    const unlisten1 = listen<Alert>("assistant-alert", ({ payload }) => {
      setAlerts(prev => [payload, ...prev].slice(0, 10));
      if (!isMuted) {
        // Flash the assistant open if closed and critical
        if (payload.severity === "critical") {
          setIsOpen(true);
        }
      }
    });

    const unlisten2 = listen<string>("assistant-message", ({ payload }) => {
      setMessages(prev => [...prev, { role: "assistant", content: payload }]);
    });

    return () => {
      unlisten1.then(f => f());
      unlisten2.then(f => f());
    };
  }, [isMuted]);

  const loadState = async () => {
    try {
      const [state, cfg] = await Promise.all([
        invoke<AssistantState>("get_assistant_state"),
        invoke<AssistantConfig>("get_assistant_config"),
      ]);
      setProviderStatus(state.provider_status);
      setAlerts(state.recent_alerts || []);
      setConfig(cfg);
      if (state.conversation.length > 0) {
        setMessages(prev => {
          const existing = new Set(prev.map(m => m.content));
          const newMsgs = state.conversation.filter(m => !existing.has(m.content));
          return [...prev, ...newMsgs];
        });
      }
    } catch (e) {
      // Running in browser dev mode without Tauri
      console.warn("Tauri not available:", e);
    }
  };

  const checkProvider = async () => {
    try {
      const status = await invoke<ProviderStatus>("check_llm_provider");
      setProviderStatus(status);
    } catch {}
  };

  const fetchOllamaModels = async () => {
    try {
      const models = await invoke<string[]>("list_ollama_models");
      setOllamaModels(models);
    } catch {}
  };

  const sendMessage = useCallback(async () => {
    if (!input.trim() || isLoading) return;
    const userMsg = input.trim();
    setInput("");
    setMessages(prev => [...prev, { role: "user", content: userMsg }]);
    setIsLoading(true);

    try {
      const response = await invoke<string>("chat_with_assistant", {
        message: userMsg,
        speakResponse: speakReplies && !isMuted,
      });
      setMessages(prev => [...prev, { role: "assistant", content: response }]);
    } catch (e: any) {
      setMessages(prev => [...prev, {
        role: "assistant",
        content: "I couldn't process that right now. Make sure an AI provider is configured in settings.",
      }]);
    }
    setIsLoading(false);
  }, [input, isLoading, speakReplies, isMuted]);

  const handleKey = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  };

  const dismissAlert = async (id: string) => {
    try {
      await invoke("acknowledge_alert", { alertId: id });
    } catch {}
    setAlerts(prev => prev.filter(a => a.id !== id));
  };

  const clearChat = async () => {
    try {
      await invoke("clear_conversation");
    } catch {}
    setMessages([{
      role: "assistant",
      content: "Conversation cleared. How can I help you?",
    }]);
  };

  const speakText = async (text: string) => {
    if (isMuted) return;
    try {
      await invoke("speak_text", { text });
    } catch {}
  };

  const saveConfig = async () => {
    if (!config) return;
    try {
      await invoke("update_assistant_config", { config });
      await checkProvider();
      setShowConfig(false);
    } catch (e) {
      console.error(e);
    }
  };

  const unacknowledgedAlerts = alerts.filter(a => !a.acknowledged);
  const criticalCount = unacknowledgedAlerts.filter(a => a.severity === "critical").length;

  return (
    <>
      {/* Floating trigger button */}
      <div className={styles.triggerWrapper}>
        {/* Alert count badge */}
        {criticalCount > 0 && !isOpen && (
          <div className={styles.alertBadge}>{criticalCount}</div>
        )}
        <button
          className={`${styles.triggerBtn} ${isOpen ? styles.triggerOpen : ""} ${criticalCount > 0 ? styles.triggerAlert : ""}`}
          onClick={() => setIsOpen(v => !v)}
          title="Open Aria assistant"
        >
          <Bot size={20} />
          {isSpeaking && <div className={styles.speakRing} />}
        </button>
      </div>

      {/* Assistant panel */}
      {isOpen && (
        <div className={styles.panel}>
          {/* Header */}
          <div className={styles.panelHeader}>
            <div className={styles.headerLeft}>
              <div className={styles.ariaAvatar}>
                <Bot size={16} />
                {!isMuted && <span className={styles.avatarPulse} />}
              </div>
              <div>
                <div className={styles.ariaName}>Aria</div>
                <div className={styles.ariaStatus}>
                  {isLoading ? "Thinking..." : isSpeaking ? "Speaking..." : "Active"}
                </div>
              </div>
            </div>
            <div className={styles.headerActions}>
              <button
                className={`${styles.iconBtn} ${isMuted ? styles.iconBtnActive : ""}`}
                onClick={() => setIsMuted(v => !v)}
                title={isMuted ? "Unmute voice" : "Mute voice"}
              >
                {isMuted ? <VolumeX size={14} /> : <Volume2 size={14} />}
              </button>
              <button className={styles.iconBtn} onClick={() => setShowConfig(v => !v)} title="Settings">
                <Settings2 size={14} />
              </button>
              <button className={styles.iconBtn} onClick={clearChat} title="Clear chat">
                <RefreshCw size={14} />
              </button>
              <button className={styles.iconBtn} onClick={() => setIsOpen(false)}>
                <X size={14} />
              </button>
            </div>
          </div>

          {/* Provider status */}
          <ProviderBadge status={providerStatus} />

          {/* Config panel */}
          {showConfig && config && (
            <AssistantConfigPanel
              config={config}
              onChange={setConfig}
              onSave={saveConfig}
              onCancel={() => setShowConfig(false)}
              ollamaModels={ollamaModels}
              onFetchModels={fetchOllamaModels}
              onCheckProvider={checkProvider}
            />
          )}

          {/* Active alerts */}
          {!showConfig && unacknowledgedAlerts.length > 0 && (
            <div className={styles.alertsSection}>
              <div className={styles.alertsSectionTitle}>
                <AlertCircle size={12} />
                <span>Active Alerts</span>
                <span className={styles.alertsCount}>{unacknowledgedAlerts.length}</span>
              </div>
              {unacknowledgedAlerts.slice(0, 3).map(alert => (
                <AlertBanner key={alert.id} alert={alert} onDismiss={() => dismissAlert(alert.id)} />
              ))}
            </div>
          )}

          {/* Chat messages */}
          {!showConfig && (
            <div className={styles.chatArea}>
              {messages.map((msg, i) => (
                <ChatBubble
                  key={i}
                  message={msg}
                  onSpeak={msg.role === "assistant" ? () => speakText(msg.content) : undefined}
                  isMuted={isMuted}
                />
              ))}
              {isLoading && (
                <div className={`${styles.bubble} ${styles.bubbleAssistant}`}>
                  <div className={styles.thinkingDots}>
                    <span /><span /><span />
                  </div>
                </div>
              )}
              <div ref={chatEndRef} />
            </div>
          )}

          {/* Quick suggestions */}
          {!showConfig && messages.length <= 1 && (
            <div className={styles.suggestions}>
              {QUICK_PROMPTS.map(prompt => (
                <button
                  key={prompt}
                  className={styles.suggestionBtn}
                  onClick={() => { setInput(prompt); inputRef.current?.focus(); }}
                >
                  {prompt}
                </button>
              ))}
            </div>
          )}

          {/* Input */}
          {!showConfig && (
            <div className={styles.inputRow}>
              <div className={styles.inputWrap}>
                <input
                  ref={inputRef}
                  className={styles.chatInput}
                  placeholder="Ask Aria anything..."
                  value={input}
                  onChange={e => setInput(e.target.value)}
                  onKeyDown={handleKey}
                  disabled={isLoading}
                />
              </div>
              <div className={styles.inputActions}>
                <label className={styles.speakToggle} title="Speak replies">
                  <input
                    type="checkbox"
                    checked={speakReplies && !isMuted}
                    onChange={e => setSpeakReplies(e.target.checked)}
                    disabled={isMuted}
                  />
                  <Volume2 size={13} />
                </label>
                <button
                  className={styles.sendBtn}
                  onClick={sendMessage}
                  disabled={!input.trim() || isLoading}
                >
                  <Send size={14} />
                </button>
              </div>
            </div>
          )}
        </div>
      )}
    </>
  );
}

// ── Chat bubble ───────────────────────────────────────────────────────────────

function ChatBubble({ message, onSpeak, isMuted }: {
  message: ChatMessage;
  onSpeak?: () => void;
  isMuted: boolean;
}) {
  const isUser = message.role === "user";
  return (
    <div className={`${styles.bubble} ${isUser ? styles.bubbleUser : styles.bubbleAssistant}`}>
      {!isUser && (
        <div className={styles.bubbleAvatar}>
          <Bot size={11} />
        </div>
      )}
      <div className={styles.bubbleContent}>
        <div className={styles.bubbleText}>{message.content}</div>
        {onSpeak && !isMuted && (
          <button className={styles.bubbleSpeakBtn} onClick={onSpeak} title="Read aloud">
            <Volume2 size={10} />
          </button>
        )}
      </div>
    </div>
  );
}

// ── Config panel ──────────────────────────────────────────────────────────────

function AssistantConfigPanel({
  config, onChange, onSave, onCancel, ollamaModels, onFetchModels, onCheckProvider
}: {
  config: AssistantConfig;
  onChange: (c: AssistantConfig) => void;
  onSave: () => void;
  onCancel: () => void;
  ollamaModels: string[];
  onFetchModels: () => void;
  onCheckProvider: () => void;
}) {
  const set = (key: keyof AssistantConfig, value: any) =>
    onChange({ ...config, [key]: value });

  return (
    <div className={styles.configPanel}>
      <div className={styles.configTitle}>Assistant Settings</div>

      {/* Provider */}
      <div className={styles.configSection}>
        <div className={styles.configLabel}>AI Provider</div>
        <div className={styles.providerTabs}>
          {["ollama", "openrouter", "none"].map(p => (
            <button
              key={p}
              className={`${styles.providerTab} ${config.provider === p ? styles.providerTabActive : ""}`}
              onClick={() => set("provider", p)}
            >
              {p === "ollama" && "🏠 Ollama (Local)"}
              {p === "openrouter" && "☁️ OpenRouter"}
              {p === "none" && "⊘ Disabled"}
            </button>
          ))}
        </div>
      </div>

      {/* Ollama config */}
      {config.provider === "ollama" && (
        <div className={styles.configSection}>
          <div className={styles.configLabel}>Ollama URL</div>
          <input
            className={styles.configInput}
            value={config.ollama_url}
            onChange={e => set("ollama_url", e.target.value)}
            placeholder="http://localhost:11434"
          />
          <div className={styles.configLabel}>Model</div>
          <div className={styles.modelRow}>
            <input
              className={styles.configInput}
              value={config.ollama_model}
              onChange={e => set("ollama_model", e.target.value)}
              placeholder="llama3"
            />
            <button className={styles.fetchModelsBtn} onClick={onFetchModels}>
              <RefreshCw size={12} /> Fetch
            </button>
          </div>
          {ollamaModels.length > 0 && (
            <div className={styles.modelList}>
              {ollamaModels.map(m => (
                <button
                  key={m}
                  className={`${styles.modelChip} ${config.ollama_model === m ? styles.modelChipActive : ""}`}
                  onClick={() => set("ollama_model", m)}
                >
                  {m}
                </button>
              ))}
            </div>
          )}
          <div className={styles.configNote}>
            💡 Recommended models: <strong>llama3</strong>, <strong>mistral</strong>, <strong>phi3</strong>, <strong>gemma2</strong>
            <br />Install: <code>ollama pull llama3</code>
          </div>
        </div>
      )}

      {/* OpenRouter config */}
      {config.provider === "openrouter" && (
        <div className={styles.configSection}>
          <div className={styles.configLabel}>API Key</div>
          <input
            className={styles.configInput}
            type="password"
            value={config.openrouter_api_key || ""}
            onChange={e => set("openrouter_api_key", e.target.value)}
            placeholder="sk-or-..."
          />
          <div className={styles.configLabel}>Model</div>
          <select
            className={styles.configSelect}
            value={config.openrouter_model}
            onChange={e => set("openrouter_model", e.target.value)}
          >
            <option value="openai/gpt-4o-mini">GPT-4o Mini (fast)</option>
            <option value="openai/gpt-4o">GPT-4o</option>
            <option value="anthropic/claude-3-haiku">Claude 3 Haiku (fast)</option>
            <option value="anthropic/claude-sonnet-4-5">Claude Sonnet</option>
            <option value="mistralai/mistral-7b-instruct">Mistral 7B</option>
            <option value="meta-llama/llama-3.1-8b-instruct">Llama 3.1 8B</option>
          </select>
        </div>
      )}

      {/* TTS config */}
      <div className={styles.configSection}>
        <div className={styles.configLabel}>Voice (TTS)</div>
        <div className={styles.toggleRow}>
          <span>Enable voice alerts</span>
          <label className={styles.toggle}>
            <input type="checkbox" checked={config.tts_enabled} onChange={e => set("tts_enabled", e.target.checked)} />
            <span className={styles.toggleSlider} />
          </label>
        </div>
        {config.tts_enabled && (
          <>
            <div className={styles.configLabel}>Engine</div>
            <select className={styles.configSelect} value={config.tts_engine} onChange={e => set("tts_engine", e.target.value)}>
              <option value="say">say (macOS built-in)</option>
              <option value="powershell">PowerShell SAPI (Windows)</option>
              <option value="espeak">espeak-ng (Linux)</option>
              <option value="piper">Piper TTS (high quality, cross-platform)</option>
              <option value="festival">Festival (Linux)</option>
            </select>
            <div className={styles.configLabel}>Speech rate (wpm): {config.tts_rate}</div>
            <input type="range" min={100} max={300} value={config.tts_rate}
              onChange={e => set("tts_rate", Number(e.target.value))}
              className={styles.configRange}
            />
            <div className={styles.configLabel}>Voice name (optional)</div>
            <input className={styles.configInput} value={config.tts_voice}
              onChange={e => set("tts_voice", e.target.value)}
              placeholder="e.g. Samantha, en-US-Neural2-C"
            />
          </>
        )}
      </div>

      {/* Alert thresholds */}
      <div className={styles.configSection}>
        <div className={styles.configLabel}>Alert Thresholds</div>
        <ThresholdRow label="CPU warn %" value={config.cpu_warn_pct} onChange={v => set("cpu_warn_pct", v)} min={50} max={95} color="var(--warn)" />
        <ThresholdRow label="CPU critical %" value={config.cpu_critical_pct} onChange={v => set("cpu_critical_pct", v)} min={70} max={99} color="var(--danger)" />
        <ThresholdRow label="RAM warn %" value={config.ram_warn_pct} onChange={v => set("ram_warn_pct", v)} min={50} max={95} color="var(--warn)" />
        <ThresholdRow label="RAM critical %" value={config.ram_critical_pct} onChange={v => set("ram_critical_pct", v)} min={70} max={99} color="var(--danger)" />
        <ThresholdRow label="Disk critical %" value={config.disk_critical_pct} onChange={v => set("disk_critical_pct", v)} min={70} max={99} color="var(--danger)" />
        <ThresholdRow label="CPU temp °C" value={config.temp_critical_c} onChange={v => set("temp_critical_c", v)} min={60} max={110} color="var(--color-entertainment)" />
        <div className={styles.configLabel}>Alert cooldown (seconds): {config.alert_cooldown_secs}</div>
        <input type="range" min={30} max={900} value={config.alert_cooldown_secs}
          onChange={e => set("alert_cooldown_secs", Number(e.target.value))}
          className={styles.configRange}
        />
        <div className={styles.configLabel}>Proactive summary every (minutes, 0=off): {config.summary_interval_mins}</div>
        <input type="range" min={0} max={120} value={config.summary_interval_mins}
          onChange={e => set("summary_interval_mins", Number(e.target.value))}
          className={styles.configRange}
        />
      </div>

      <div className={styles.configActions}>
        <button className={styles.checkBtn} onClick={onCheckProvider}>
          <Zap size={13} /> Test connection
        </button>
        <button className={styles.cancelBtn} onClick={onCancel}>Cancel</button>
        <button className={styles.saveBtn} onClick={onSave}>
          <CheckCircle2 size={13} /> Save
        </button>
      </div>
    </div>
  );
}

function ThresholdRow({ label, value, onChange, min, max, color }: {
  label: string; value: number; onChange: (v: number) => void; min: number; max: number; color: string;
}) {
  return (
    <div className={styles.thresholdRow}>
      <span className={styles.thresholdLabel}>{label}</span>
      <input type="range" min={min} max={max} value={value}
        onChange={e => onChange(Number(e.target.value))}
        className={styles.configRange}
        style={{ accentColor: color }}
      />
      <span className={styles.thresholdVal} style={{ color }}>{value}</span>
    </div>
  );
}

const QUICK_PROMPTS = [
  "How's my system health?",
  "What have I been working on today?",
  "How productive was I this week?",
  "What app am I spending the most time in?",
];
