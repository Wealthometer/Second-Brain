use tauri::State;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::ai::voice_assistant::{
    AssistantConfig, ChatMessage, LLMEngine, VoiceEngine, ProviderStatus,
    AssistantState, Alert, get_current_snapshot,
};

/// Global assistant state — stored in Tauri app state
pub struct AssistantAppState { 
    pub config: Arc<Mutex<AssistantConfig>>,
    pub conversation: Arc<Mutex<Vec<ChatMessage>>>,
    pub recent_alerts: Arc<Mutex<Vec<Alert>>>,
    pub is_speaking: Arc<Mutex<bool>>,
}

impl AssistantAppState {
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(AssistantConfig::default())),
            conversation: Arc::new(Mutex::new(vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: crate::ai::voice_assistant::SYSTEM_PROMPT_PUBLIC.to_string(),
                }
            ])),
            recent_alerts: Arc::new(Mutex::new(Vec::new())),
            is_speaking: Arc::new(Mutex::new(false)),
        }
    }
}

#[tauri::command]
pub async fn get_assistant_config(
    assistant: State<'_, AssistantAppState>,
) -> Result<AssistantConfig, String> {
    Ok(assistant.config.lock().await.clone())
}

#[tauri::command]
pub async fn update_assistant_config(
    assistant: State<'_, AssistantAppState>,
    config: AssistantConfig,
) -> Result<(), String> {
    *assistant.config.lock().await = config;
    Ok(())
}

#[tauri::command]
pub async fn check_llm_provider(
    assistant: State<'_, AssistantAppState>,
) -> Result<ProviderStatus, String> {
    let config = assistant.config.lock().await.clone();
    let engine = LLMEngine::new(config);
    Ok(engine.check_provider().await)
}

#[tauri::command]
pub async fn chat_with_assistant(
    assistant: State<'_, AssistantAppState>,
    message: String,
    speak_response: bool,
) -> Result<String, String> {
    let config = assistant.config.lock().await.clone();

    // Get system context to inject
    let context_msg = if let Ok(snap) = get_current_snapshot().await {
        format!(
            "\n[System context: CPU {:.1}%, RAM {:.1}%, Disk {:.1}%]",
            snap.cpu_usage, snap.ram_percent, snap.disk_percent
        )
    } else {
        String::new()
    };

    // Build message with context
    let user_msg = ChatMessage {
        role: "user".to_string(),
        content: format!("{}{}", message, context_msg),
    };

    {
        let mut conv = assistant.conversation.lock().await;
        conv.push(user_msg);
    }

    let conv_snapshot = assistant.conversation.lock().await.clone();
    let llm = LLMEngine::new(config.clone());
    let response = llm.chat(&conv_snapshot).await.map_err(|e| e.to_string())?;

    {
        let mut conv = assistant.conversation.lock().await;
        conv.push(ChatMessage {
            role: "assistant".to_string(),
            content: response.clone(),
        });
        // Keep conversation bounded
        if conv.len() > 30 {
            let system = conv[0].clone();
            *conv = std::iter::once(system)
                .chain(conv.iter().skip(conv.len() - 20).cloned())
                .collect();
        }
    }

    // Speak the response if requested
    if speak_response {
        let voice = VoiceEngine::new(config);
        let resp_clone = response.clone();
        tokio::spawn(async move {
            voice.speak(&resp_clone).await.ok();
        });
    }

    Ok(response)
}

#[tauri::command]
pub async fn speak_text(
    assistant: State<'_, AssistantAppState>,
    text: String,
) -> Result<(), String> {
    let config = assistant.config.lock().await.clone();
    let voice = VoiceEngine::new(config);
    tokio::spawn(async move {
        voice.speak(&text).await.ok();
    });
    Ok(())
}

#[tauri::command]
pub async fn clear_conversation(
    assistant: State<'_, AssistantAppState>,
) -> Result<(), String> {
    let mut conv = assistant.conversation.lock().await;
    let system = conv.first().cloned().unwrap_or(ChatMessage {
        role: "system".to_string(),
        content: crate::ai::voice_assistant::SYSTEM_PROMPT_PUBLIC.to_string(),
    });
    *conv = vec![system];
    Ok(())
}

#[tauri::command]
pub async fn get_assistant_state(
    assistant: State<'_, AssistantAppState>,
) -> Result<AssistantState, String> {
    let config = assistant.config.lock().await.clone();
    let llm = LLMEngine::new(config.clone());
    let status = llm.check_provider().await;

    Ok(AssistantState {
        is_speaking: *assistant.is_speaking.lock().await,
        is_listening: false,
        recent_alerts: assistant.recent_alerts.lock().await.clone(),
        conversation: assistant.conversation.lock().await
            .iter()
            .filter(|m| m.role != "system")
            .cloned()
            .collect(),
        provider_status: status,
    })
}

#[tauri::command]
pub async fn get_recent_alerts(
    assistant: State<'_, AssistantAppState>,
) -> Result<Vec<Alert>, String> {
    Ok(assistant.recent_alerts.lock().await.clone())
}

#[tauri::command]
pub async fn acknowledge_alert(
    assistant: State<'_, AssistantAppState>,
    alert_id: String,
) -> Result<(), String> {
    let mut alerts = assistant.recent_alerts.lock().await;
    if let Some(a) = alerts.iter_mut().find(|a| a.id == alert_id) {
        a.acknowledged = true;
    }
    Ok(())
}

#[tauri::command]
pub async fn list_ollama_models(
    assistant: State<'_, AssistantAppState>,
) -> Result<Vec<String>, String> {
    let config = assistant.config.lock().await.clone();
    let client = reqwest::Client::new();
    let url = format!("{}/api/tags", config.ollama_url);

    let resp = client.get(&url)
        .send()
        .await
        .map_err(|_| "Ollama not running".to_string())?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| e.to_string())?;

    let models: Vec<String> = resp["models"]
        .as_array()
        .map(|arr| arr.iter()
            .filter_map(|m| m["name"].as_str().map(String::from))
            .collect())
        .unwrap_or_default();

    Ok(models)
}

#[tauri::command]
pub async fn get_system_alert_check() -> Result<Vec<Alert>, String> {
    use crate::ai::voice_assistant::AlertEngine;
    let config = AssistantConfig::default();
    let mut engine = AlertEngine::new(config.clone());

    let snap = get_current_snapshot().await.map_err(|e| e.to_string())?;
    Ok(engine.check_system_stats(&snap))
}
