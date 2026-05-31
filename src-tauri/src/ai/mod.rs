pub mod voice_assistant;
pub mod assistant_commands;

use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)] 
pub struct AIInsight {
    pub insight_type: String,
    pub title: String,
    pub body: String,
    pub score: Option<f32>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterRequest {
    model: String,
    messages: Vec<OpenRouterMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Choice {
    message: OpenRouterMessage,
}

pub struct AIEngine {
    api_key: String,
    client: reqwest::Client,
}

impl AIEngine {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    pub async fn generate_productivity_summary(&self, context: &ProductivityContext) -> Result<AIInsight> {
        let prompt = format!(
            r#"You are analyzing a user's computer activity data to generate productivity insights.

Activity Summary:
- Top apps today: {}
- Total active time: {} minutes  
- Focus sessions (10+ min uninterrupted): {}
- Context switches per hour: {:.1}
- Most productive hour: {}
- Browser tabs opened: {}

Generate a concise productivity summary with:
1. An overall productivity score (0-100)
2. Key insight (1-2 sentences)
3. Top 3 actionable suggestions

Respond ONLY with valid JSON:
{{
  "score": <number 0-100>,
  "insight": "<string>",
  "suggestions": ["<string>", "<string>", "<string>"]
}}"#,
            context.top_apps.join(", "),
            context.active_minutes,
            context.focus_sessions,
            context.context_switches_per_hour,
            context.peak_hour,
            context.browser_tabs,
        );

        let response = self.call_api(&prompt).await?;
        
        // Parse JSON response
        let json: serde_json::Value = serde_json::from_str(&response)
            .map_err(|_| anyhow::anyhow!("Invalid JSON from AI"))?;

        Ok(AIInsight {
            insight_type: "productivity".to_string(),
            title: "Daily Productivity Analysis".to_string(),
            body: json["insight"].as_str().unwrap_or("").to_string(),
            score: json["score"].as_f64().map(|s| s as f32),
            suggestions: json["suggestions"]
                .as_array()
                .map(|arr| arr.iter().filter_map(|s| s.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default(),
        })
    }

    pub async fn analyze_workflow_pattern(&self, app_sequence: &[String]) -> Result<AIInsight> {
        let prompt = format!(
            r#"Analyze this sequence of applications a user switched between today:
{}

Identify:
1. The main workflow pattern (e.g., "development loop", "research mode", "communication heavy")
2. Any inefficiencies or context-switching issues
3. Suggestions to optimize this workflow

Respond ONLY with valid JSON:
{{
  "pattern": "<string>",
  "efficiency": <number 0-100>,
  "insight": "<string>",
  "suggestions": ["<string>", "<string>"]
}}"#,
            app_sequence.join(" → ")
        );

        let response = self.call_api(&prompt).await?;
        let json: serde_json::Value = serde_json::from_str(&response)
            .map_err(|_| anyhow::anyhow!("Invalid JSON"))?;

        Ok(AIInsight {
            insight_type: "workflow".to_string(),
            title: format!("Workflow: {}", json["pattern"].as_str().unwrap_or("Unknown")),
            body: json["insight"].as_str().unwrap_or("").to_string(),
            score: json["efficiency"].as_f64().map(|s| s as f32),
            suggestions: json["suggestions"]
                .as_array()
                .map(|arr| arr.iter().filter_map(|s| s.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default(),
        })
    }

    pub async fn smart_search_enhance(&self, query: &str, results_context: &str) -> Result<String> {
        let prompt = format!(
            r#"The user searched for: "{}"

Here are the raw results from their activity database:
{}

Generate a natural language summary of what they might be looking for and highlight the most relevant results. Keep it under 100 words."#,
            query, results_context
        );

        self.call_api(&prompt).await
    }

    async fn call_api(&self, prompt: &str) -> Result<String> {
        let request = OpenRouterRequest {
            model: "openai/gpt-4o-mini".to_string(),
            messages: vec![OpenRouterMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens: 500,
            temperature: 0.3,
        };

        let response = self.client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", "https://second-brain.app")
            .header("X-Title", "Second Brain")
            .json(&request)
            .send()
            .await?
            .json::<OpenRouterResponse>()
            .await?;

        Ok(response.choices.into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default())
    }
}

#[derive(Debug, Default)]
pub struct ProductivityContext {
    pub top_apps: Vec<String>,
    pub active_minutes: i64,
    pub focus_sessions: i32,
    pub context_switches_per_hour: f32,
    pub peak_hour: String,
    pub browser_tabs: i32,
}
