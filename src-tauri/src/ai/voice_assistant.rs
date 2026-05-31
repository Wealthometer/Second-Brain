/// Voice Intelligence Assistant
/// 
/// Architecture:
/// 1. AlertEngine — monitors thresholds, fires alert events
/// 2. VoiceEngine — converts text to speech (cross-platform)
/// 3. LLMEngine — talks to local Ollama OR cloud OpenRouter
/// 4. STTEngine — speech-to-text for voice commands
/// 5. AssistantOrchestrator — wires everything together

use std::sync::Arc;
use tokio::sync::Mutex; 
use serde::{Deserialize, Serialize};
use chrono::Utc;
use tauri::{AppHandle, Emitter};

use crate::db::Database;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Alert {
    pub id: String,
    pub severity: AlertSeverity,
    pub category: String,
    pub title: String,
    pub message: String,
    pub spoken_text: String,     // What the TTS voice says
    pub suggestion: Option<String>,
    pub timestamp: chrono::DateTime<Utc>,
    pub acknowledged: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssistantConfig {
    /// "ollama" | "openrouter" | "none"
    pub provider: String,
    /// For Ollama: model name like "llama3", "mistral", "phi3"
    pub ollama_model: String,
    /// Ollama base URL (default http://localhost:11434)
    pub ollama_url: String,
    /// For OpenRouter cloud API
    pub openrouter_api_key: Option<String>,
    pub openrouter_model: String,
    /// TTS engine: "espeak" | "say" | "powershell" | "festival"
    pub tts_engine: String,
    pub tts_voice: String,
    pub tts_rate: i32,           // words per minute
    pub tts_volume: i32,         // Volume percentage (0-100)
    pub tts_enabled: bool,
    /// Mic input for voice commands
    pub stt_enabled: bool,
    /// Alert thresholds
    pub cpu_critical_pct: f32,
    pub cpu_warn_pct: f32,
    pub ram_critical_pct: f32,
    pub ram_warn_pct: f32,
    pub disk_critical_pct: f32,
    pub temp_critical_c: f32,
    /// How often to speak proactive summaries (minutes, 0=off)
    pub summary_interval_mins: u64,
    /// Mute repeated same-category alerts (seconds)
    pub alert_cooldown_secs: i64,
}

impl Default for AssistantConfig {
    fn default() -> Self {
        Self {
            provider: "ollama".to_string(),
            ollama_model: "llama3".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            openrouter_api_key: None,
            openrouter_model: "openai/gpt-4o-mini".to_string(),
            tts_engine: detect_tts_engine(),
            tts_voice: "".to_string(),
            tts_rate: 175,
            tts_volume: 80,  // Default volume at 80%
            tts_enabled: true,
            stt_enabled: false,
            cpu_critical_pct: 92.0,
            cpu_warn_pct: 75.0,
            ram_critical_pct: 90.0,
            ram_warn_pct: 78.0,
            disk_critical_pct: 90.0,
            temp_critical_c: 85.0,
            summary_interval_mins: 60,
            alert_cooldown_secs: 300,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,   // "user" | "assistant" | "system"
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssistantState {
    pub is_speaking: bool,
    pub is_listening: bool,
    pub recent_alerts: Vec<Alert>,
    pub conversation: Vec<ChatMessage>,
    pub provider_status: ProviderStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProviderStatus {
    pub name: String,
    pub connected: bool,
    pub model: String,
    pub error: Option<String>,
}

// ── TTS Engine ────────────────────────────────────────────────────────────────

pub struct VoiceEngine {
    config: AssistantConfig,
}

impl VoiceEngine {
    pub fn new(config: AssistantConfig) -> Self {
        Self { config }
    }

    pub async fn speak(&self, text: &str) -> anyhow::Result<()> {
        if !self.config.tts_enabled {
            return Ok(());
        }
        let text = text.to_string();
        let engine = self.config.tts_engine.clone();
        let rate = self.config.tts_rate;
        let voice = self.config.tts_voice.clone();
        let volume = self.config.tts_volume;

        // Spawn blocking TTS in thread so we don't block async runtime
        tokio::task::spawn_blocking(move || {
            speak_blocking(&text, &engine, rate, &voice, volume)
        }).await??;

        Ok(())
    }

    pub async fn speak_alert(&self, alert: &Alert) {
        let prefix = match alert.severity {
            AlertSeverity::Critical => "Critical alert. ",
            AlertSeverity::Warning => "Warning. ",
            AlertSeverity::Info => "",
        };
        let text = format!("{}{}", prefix, alert.spoken_text);
        if let Err(e) = self.speak(&text).await {
            log::warn!("TTS error: {}", e);
        }
    }
}

fn speak_blocking(text: &str, engine: &str, rate: i32, voice: &str, volume: i32) -> anyhow::Result<()> {
    use std::process::Command;
    #[cfg(target_os = "windows")]
    use std::os::windows::process::CommandExt;

    match engine {
        "say" => {
            // macOS built-in - no direct volume control in say command
            let mut cmd = Command::new("say");
            cmd.arg("-r").arg(rate.to_string());
            if !voice.is_empty() {
                cmd.arg("-v").arg(voice);
            }
            cmd.arg(text).status()?;
        }
        "powershell" => {
            // Windows — PowerShell SpeechSynthesizer
            let script = format!(
                r#"
Add-Type -AssemblyName System.Speech
$synth = New-Object System.Speech.Synthesis.SpeechSynthesizer
$synth.Rate = {}
$synth.Volume = {}
{}
$synth.Speak("{}")
"#,
                // Rate: Windows is -10 to 10, map 100-300 wpm → -5 to 5
                ((rate as f32 - 200.0) / 20.0).clamp(-10.0, 10.0) as i32,
                volume.clamp(0, 100),  // Volume: 0 to 100
                if !voice.is_empty() { format!("$synth.SelectVoice(\"{}\")", voice) } else { String::new() },
                text.replace('"', "'")
            );
            let mut cmd = Command::new("powershell");
            cmd.args(["-NoProfile", "-Command", &script]);
            #[cfg(target_os = "windows")]
            cmd.creation_flags(CREATE_NO_WINDOW);
            cmd.status()?;
        }
        "espeak" => {
            // Linux espeak-ng - use amplitude for volume (0-200, map 0-100 to 0-200)
            let mut cmd = Command::new("espeak-ng");
            cmd.arg("-s").arg(rate.to_string());
            cmd.arg("-a").arg((volume * 2).to_string());  // Map 0-100 to 0-200
            if !voice.is_empty() {
                cmd.arg("-v").arg(voice);
            }
            cmd.arg(text).status()?;
        }
        "festival" => {
            // Linux festival - set volume using scheme
            use std::io::Write;
            let volume_scale = volume as f32 / 100.0;  // Convert to 0.0-1.0 range
            let mut child = Command::new("festival")
                .arg("--tts")
                .stdin(std::process::Stdio::piped())
                .spawn()?;
            if let Some(stdin) = child.stdin.as_mut() {
                // Set volume then speak the text
                let ssml = format!("(set! volume {})\n{}", volume_scale, text);
                stdin.write_all(ssml.as_bytes())?;
            }
            child.wait()?;
        }
        "piper" => {
            // Piper — high quality local neural TTS (Linux/macOS/Windows)
            // Piper doesn't have direct volume control, rely on system volume or post-processing
            use std::io::Write;
            let model = if voice.is_empty() { "en_US-lessac-medium".to_string() } else { voice.to_string() };
            let mut child = Command::new("piper")
                .args(["--model", &model, "--output-raw"])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn()?;
            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(text.as_bytes())?;
            }
            let output = child.wait_with_output()?;
            // Pipe to aplay (Linux) or afplay raw workaround
            let mut player = Command::new("aplay")
                .args(["-r", "22050", "-f", "S16_LE", "-c", "1"])
                .stdin(std::process::Stdio::piped())
                .spawn()?;
            if let Some(stdin) = player.stdin.as_mut() {
                use std::io::Write;
                stdin.write_all(&output.stdout)?;
            }
            player.wait()?;
        }
        _ => {
            log::warn!("Unknown TTS engine: {}", engine);
        }
    }
    Ok(())
}

fn detect_tts_engine() -> String {
    #[cfg(target_os = "macos")] { return "say".to_string(); }
    #[cfg(target_os = "windows")] { return "powershell".to_string(); }
    #[cfg(target_os = "linux")] {
        if std::process::Command::new("piper").arg("--version").output().is_ok() {
            return "piper".to_string();
        }
        if std::process::Command::new("espeak-ng").arg("--version").output().is_ok() {
            return "espeak".to_string();
        }
        return "festival".to_string();
    }
}

// ── LLM Engine ───────────────────────────────────────────────────────────────

pub struct LLMEngine {
    config: AssistantConfig,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    message: ChatMessage,
}

#[derive(Serialize)]
struct OpenRouterRequest {
    model: String,
    messages: Vec<serde_json::Value>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Deserialize)]
struct OpenRouterResponse {
    choices: Vec<OpenRouterChoice>,
}

#[derive(Deserialize)]
struct OpenRouterChoice {
    message: OpenRouterMsg,
}

#[derive(Deserialize)]
struct OpenRouterMsg {
    content: String,
}

impl LLMEngine {
    pub fn new(config: AssistantConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(),
        }
    }

    pub async fn chat(&self, messages: &[ChatMessage]) -> anyhow::Result<String> {
        match self.config.provider.as_str() {
            "ollama" => self.chat_ollama(messages).await,
            "openrouter" => self.chat_openrouter(messages).await,
            _ => Ok("AI provider not configured.".to_string()),
        }
    }

    async fn chat_ollama(&self, messages: &[ChatMessage]) -> anyhow::Result<String> {
        let url = format!("{}/api/chat", self.config.ollama_url);
        let req = OllamaRequest {
            model: self.config.ollama_model.clone(),
            messages: messages.to_vec(),
            stream: false,
        };
        let resp = self.client.post(&url).json(&req).send().await?;
        let body: OllamaResponse = resp.json().await?;
        Ok(body.message.content)
    }

    async fn chat_openrouter(&self, messages: &[ChatMessage]) -> anyhow::Result<String> {
        let api_key = self.config.openrouter_api_key.as_deref()
            .ok_or_else(|| anyhow::anyhow!("No OpenRouter API key"))?;

        let msgs: Vec<serde_json::Value> = messages.iter().map(|m| {
            serde_json::json!({ "role": m.role, "content": m.content })
        }).collect();

        let req = serde_json::json!({
            "model": self.config.openrouter_model,
            "messages": msgs,
            "max_tokens": 400,
            "temperature": 0.7,
        });

        let resp = self.client.post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("HTTP-Referer", "https://second-brain.app")
            .header("X-Title", "Second Brain")
            .json(&req)
            .send()
            .await?;

        let body: OpenRouterResponse = resp.json().await?;
        Ok(body.choices.into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default())
    }

    pub async fn check_provider(&self) -> ProviderStatus {
        match self.config.provider.as_str() {
            "ollama" => {
                let url = format!("{}/api/tags", self.config.ollama_url);
                match self.client.get(&url).send().await {
                    Ok(r) if r.status().is_success() => ProviderStatus {
                        name: "Ollama (Local)".to_string(),
                        connected: true,
                        model: self.config.ollama_model.clone(),
                        error: None,
                    },
                    Ok(r) => ProviderStatus {
                        name: "Ollama (Local)".to_string(),
                        connected: false,
                        model: self.config.ollama_model.clone(),
                        error: Some(format!("HTTP {}", r.status())),
                    },
                    Err(_e) => ProviderStatus {
                        name: "Ollama (Local)".to_string(),
                        connected: false,
                        model: self.config.ollama_model.clone(),
                        error: Some("Ollama not running. Start with: ollama serve".to_string()),
                    },
                }
            }
            "openrouter" => {
                if self.config.openrouter_api_key.is_some() {
                    ProviderStatus {
                        name: "OpenRouter (Cloud)".to_string(),
                        connected: true,
                        model: self.config.openrouter_model.clone(),
                        error: None,
                    }
                } else {
                    ProviderStatus {
                        name: "OpenRouter (Cloud)".to_string(),
                        connected: false,
                        model: self.config.openrouter_model.clone(),
                        error: Some("API key required".to_string()),
                    }
                }
            }
            _ => ProviderStatus {
                name: "None".to_string(),
                connected: false,
                model: "".to_string(),
                error: Some("No provider configured".to_string()),
            },
        }
    }
}

// ── Alert Engine ──────────────────────────────────────────────────────────────

pub struct AlertEngine {
    config: AssistantConfig,
    /// Track last alert time per category to avoid spam
    last_alerts: std::collections::HashMap<String, chrono::DateTime<Utc>>,
}

impl AlertEngine {
    pub fn new(config: AssistantConfig) -> Self {
        Self { config, last_alerts: Default::default() }
    }

    pub fn check_system_stats(&mut self, stats: &SystemStatsSnapshot) -> Vec<Alert> {
        let mut alerts = Vec::new();

        // CPU
        if stats.cpu_usage >= self.config.cpu_critical_pct {
            if self.should_alert("cpu_critical") {
                alerts.push(Alert {
                    id: uuid::Uuid::new_v4().to_string(),
                    severity: AlertSeverity::Critical,
                    category: "cpu_critical".to_string(),
                    title: "CPU Overload".to_string(),
                    message: format!("CPU at {:.1}% — your computer is under extreme load", stats.cpu_usage),
                    spoken_text: format!("Warning! CPU usage is critical at {:.0} percent. Consider closing some apps.", stats.cpu_usage),
                    suggestion: Some("Close unused applications or check for runaway processes".to_string()),
                    timestamp: Utc::now(),
                    acknowledged: false,
                });
            }
        } else if stats.cpu_usage >= self.config.cpu_warn_pct {
            if self.should_alert("cpu_warn") {
                alerts.push(Alert {
                    id: uuid::Uuid::new_v4().to_string(),
                    severity: AlertSeverity::Warning,
                    category: "cpu_warn".to_string(),
                    title: "High CPU Usage".to_string(),
                    message: format!("CPU at {:.1}%", stats.cpu_usage),
                    spoken_text: format!("Heads up. CPU usage is high at {:.0} percent.", stats.cpu_usage),
                    suggestion: Some("Monitor active processes".to_string()),
                    timestamp: Utc::now(),
                    acknowledged: false,
                });
            }
        }

        // RAM
        if stats.ram_percent >= self.config.ram_critical_pct {
            if self.should_alert("ram_critical") {
                alerts.push(Alert {
                    id: uuid::Uuid::new_v4().to_string(),
                    severity: AlertSeverity::Critical,
                    category: "ram_critical".to_string(),
                    title: "Memory Critical".to_string(),
                    message: format!("RAM at {:.1}% — system may become unstable", stats.ram_percent),
                    spoken_text: format!("Critical! System memory is at {:.0} percent. Your computer may slow down or crash.", stats.ram_percent),
                    suggestion: Some("Close browser tabs and unused apps immediately".to_string()),
                    timestamp: Utc::now(),
                    acknowledged: false,
                });
            }
        } else if stats.ram_percent >= self.config.ram_warn_pct {
            if self.should_alert("ram_warn") {
                alerts.push(Alert {
                    id: uuid::Uuid::new_v4().to_string(),
                    severity: AlertSeverity::Warning,
                    category: "ram_warn".to_string(),
                    title: "High Memory Usage".to_string(),
                    message: format!("RAM at {:.1}%", stats.ram_percent),
                    spoken_text: format!("Memory usage is getting high at {:.0} percent.", stats.ram_percent),
                    suggestion: Some("Consider closing some browser tabs".to_string()),
                    timestamp: Utc::now(),
                    acknowledged: false,
                });
            }
        }

        // Disk
        if stats.disk_percent >= self.config.disk_critical_pct {
            if self.should_alert("disk_critical") {
                alerts.push(Alert {
                    id: uuid::Uuid::new_v4().to_string(),
                    severity: AlertSeverity::Critical,
                    category: "disk_critical".to_string(),
                    title: "Disk Almost Full".to_string(),
                    message: format!("Disk at {:.1}% — only {:.1} GB remaining", stats.disk_percent, stats.disk_free_gb),
                    spoken_text: format!("Critical! Your disk is almost full at {:.0} percent capacity. Free up space to avoid data loss.", stats.disk_percent),
                    suggestion: Some("Empty trash and delete large unused files".to_string()),
                    timestamp: Utc::now(),
                    acknowledged: false,
                });
            }
        }

        // Temperature (if available)
        if let Some(temp) = stats.cpu_temp_c {
            if temp >= self.config.temp_critical_c {
                if self.should_alert("temp_critical") {
                    alerts.push(Alert {
                        id: uuid::Uuid::new_v4().to_string(),
                        severity: AlertSeverity::Critical,
                        category: "temp_critical".to_string(),
                        title: "CPU Overheating".to_string(),
                        message: format!("CPU temperature at {:.0}°C — thermal throttling may occur", temp),
                        spoken_text: format!("Warning! Your CPU is overheating at {:.0} degrees Celsius. Check your cooling system.", temp),
                        suggestion: Some("Ensure vents aren't blocked; consider elevating laptop".to_string()),
                        timestamp: Utc::now(),
                        acknowledged: false,
                    });
                }
            }
        }

        // Battery warnings
        if let (Some(pct), Some(charging)) = (stats.battery_percent, stats.battery_charging) {
            if !charging && pct <= 10.0 {
                if self.should_alert("battery_critical") {
                    alerts.push(Alert {
                        id: uuid::Uuid::new_v4().to_string(),
                        severity: AlertSeverity::Critical,
                        category: "battery_critical".to_string(),
                        title: "Battery Critical".to_string(),
                        message: format!("Battery at {:.0}% — plug in now", pct),
                        spoken_text: format!("Battery critical at {:.0} percent. Please plug in your charger now.", pct),
                        suggestion: Some("Connect power adapter immediately".to_string()),
                        timestamp: Utc::now(),
                        acknowledged: false,
                    });
                }
            } else if !charging && pct <= 20.0 {
                if self.should_alert("battery_low") {
                    alerts.push(Alert {
                        id: uuid::Uuid::new_v4().to_string(),
                        severity: AlertSeverity::Warning,
                        category: "battery_low".to_string(),
                        title: "Low Battery".to_string(),
                        message: format!("Battery at {:.0}%", pct),
                        spoken_text: format!("Battery is low at {:.0} percent. Consider charging soon.", pct),
                        suggestion: None,
                        timestamp: Utc::now(),
                        acknowledged: false,
                    });
                }
            }
        }

        // Record alert times
        for alert in &alerts {
            self.last_alerts.insert(alert.category.clone(), Utc::now());
        }

        alerts
    }

    fn should_alert(&self, category: &str) -> bool {
        match self.last_alerts.get(category) {
            None => true,
            Some(last) => (Utc::now() - *last).num_seconds() >= self.config.alert_cooldown_secs,
        }
    }
}

// ── Snapshot for alert checking ───────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SystemStatsSnapshot {
    pub cpu_usage: f32,
    pub ram_percent: f32,
    pub disk_percent: f32,
    pub disk_free_gb: f32,
    pub cpu_temp_c: Option<f32>,
    pub battery_percent: Option<f32>,
    pub battery_charging: Option<bool>,
}

// ── Main Orchestrator loop ────────────────────────────────────────────────────

pub async fn run_assistant(
    db: Arc<Mutex<Database>>,
    app: AppHandle,
    config: AssistantConfig,
) {
    log::info!("Voice assistant starting with provider: {}", config.provider);

    let voice = Arc::new(VoiceEngine::new(config.clone()));
    let llm = Arc::new(LLMEngine::new(config.clone()));
    let mut alert_engine = AlertEngine::new(config.clone());
    let mut conversation: Vec<ChatMessage> = vec![
        ChatMessage {
            role: "system".to_string(),
            content: SYSTEM_PROMPT.to_string(),
        }
    ];

    // Check provider on startup
    let status = llm.check_provider().await;
    if status.connected {
        let welcome = format!(
            "Second Brain assistant online. Using {} with model {}. I'm monitoring your system.",
            status.name, status.model
        );
        voice.speak(&welcome).await.ok();
    } else {
        log::warn!("LLM provider not connected: {:?}", status.error);
        voice.speak("Second Brain assistant started. System monitoring is active.").await.ok();
    }

    let mut stats_tick = tokio::time::interval(tokio::time::Duration::from_secs(5));
    let mut summary_tick = tokio::time::interval(
        tokio::time::Duration::from_secs(config.summary_interval_mins * 60)
    );

    loop {
        tokio::select! {
            _ = stats_tick.tick() => {
                // Check system stats for alerts
                if let Ok(snapshot) = get_current_snapshot().await {
                    let alerts = alert_engine.check_system_stats(&snapshot);
                    for alert in &alerts {
                        log::warn!("Alert: {} — {}", alert.title, alert.message);
                        // Emit to frontend
                        let _ = app.emit("assistant-alert", &alert);
                        // Speak it
                        voice.speak_alert(alert).await;
                    }
                }
            }
            _ = summary_tick.tick() => {
                // Proactive productivity summary
                if config.summary_interval_mins > 0 && status.connected {
                    if let Ok(summary) = generate_proactive_summary(&db, &llm, &mut conversation).await {
                        log::info!("Proactive summary: {}", summary);
                        let _ = app.emit("assistant-message", &summary);
                        voice.speak(&summary).await.ok();
                    }
                }
            }
        }
    }
}

pub async fn get_current_snapshot() -> anyhow::Result<SystemStatsSnapshot> {
    use sysinfo::{System, Disks, Components};

    let mut sys = System::new_all();
    sys.refresh_all();

    let disks = Disks::new_with_refreshed_list();
    let components = Components::new_with_refreshed_list();

    let ram_percent = sys.used_memory() as f32 / sys.total_memory() as f32 * 100.0;

    let (disk_used, disk_total) = disks.iter().fold((0.0f32, 0.0f32), |acc, d| {
        let total = d.total_space() as f32 / 1_073_741_824.0;
        let available = d.available_space() as f32 / 1_073_741_824.0;
        (acc.0 + (total - available), acc.1 + total)
    });
    let disk_percent = if disk_total > 0.0 { disk_used / disk_total * 100.0 } else { 0.0 };
    let disk_free_gb = disk_total - disk_used;

    // Try to read CPU temp from hwmon or components
    let cpu_temp_c = components.iter()
        .find(|c| {
            let label = c.label().to_lowercase();
            label.contains("cpu") || label.contains("core") || label.contains("package")
        })
        .map(|c| c.temperature());

    Ok(SystemStatsSnapshot {
        cpu_usage: sys.global_cpu_info().frequency() as f32,
        ram_percent,
        disk_percent,
        disk_free_gb,
        cpu_temp_c,
        battery_percent: None,
        battery_charging: None,
    })
}

async fn generate_proactive_summary(
    db: &Arc<Mutex<Database>>,
    llm: &Arc<LLMEngine>,
    conversation: &mut Vec<ChatMessage>,
) -> anyhow::Result<String> {
    let today = Utc::now().date_naive().to_string();

    let (top_apps, total_mins) = {
        let db = db.lock().await;
        let usage = db.get_app_usage_by_date(&today)?;
        let total = usage.iter().map(|u| u.duration_secs).sum::<i64>() / 60;
        let apps: Vec<String> = usage.iter().take(5).map(|u| {
            format!("{} ({}m)", u.app_name, u.duration_secs / 60)
        }).collect();
        (apps.join(", "), total)
    };

    let user_msg = ChatMessage {
        role: "user".to_string(),
        content: format!(
            "Give me a 1-sentence proactive insight. Today: {} active minutes. Top apps: {}. Keep it under 20 words, conversational.",
            total_mins, top_apps
        ),
    };

    conversation.push(user_msg);
    let response = llm.chat(conversation).await?;
    conversation.push(ChatMessage {
        role: "assistant".to_string(),
        content: response.clone(),
    });

    // Keep conversation short
    if conversation.len() > 20 {
        let system = conversation[0].clone();
        *conversation = std::iter::once(system).chain(conversation.iter().skip(conversation.len() - 10).cloned()).collect();
    }

    Ok(response)
}

pub const SYSTEM_PROMPT_PUBLIC: &str = SYSTEM_PROMPT;

const SYSTEM_PROMPT: &str = r#"You are Aria, an intelligent voice assistant embedded inside Second Brain, a personal intelligence OS.

Your role:
- Monitor the user's computer and provide helpful, concise spoken alerts
- Answer questions about their productivity, app usage, and system health
- Give proactive suggestions to improve their workflow
- Sound natural and conversational — you'll be spoken aloud via text-to-speech

Rules:
- Keep responses SHORT (1-3 sentences max for TTS)
- No markdown, no bullet points — plain conversational English
- Be direct and actionable
- For critical alerts, be urgent but calm
- You have access to: CPU/RAM/disk stats, active apps, time spent per app, clipboard history count, file events
- Never make up stats you don't have

Personality: Calm, intelligent, efficient. Like a co-pilot who notices things and speaks up when it matters."#;
