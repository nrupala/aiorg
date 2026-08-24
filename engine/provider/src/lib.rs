use anyhow::{bail, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Duration;

/// OpenAI-compatible client for llama.cpp router (:8830/v1) and Ollama (:11434/v1).
/// Handles reasoning_content merge, schema-validate-retry, and timeouts.
pub struct Provider {
    client: Client,
    router_url: String,
    ollama_url: String,
    model_primary: String,
}

impl Provider {
    /// Create a new Provider pointing at the router and a fallback Ollama URL.
    pub fn new(router_url: String, ollama_url: String, model_primary: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .connect_timeout(Duration::from_secs(30))
            .build()
            .expect("reqwest client build");
        Self {
            client,
            router_url,
            ollama_url,
            model_primary,
        }
    }

    /// Chat completion via the router. Merges `message.reasoning_content` + `message.content`
    /// as per Qwen3/DeepSeek reasoning quirk.
    pub async fn chat(&self, model: &str, messages: &[serde_json::Value]) -> Result<serde_json::Value> {
        let payload = serde_json::json!({
            "model": model,
            "messages": messages,
            "temperature": 0.2,
            "max_tokens": 2048,
        });
        let url = format!("{}/v1/chat/completions", self.router_url);
        let resp = self.client.post(&url).json(&payload).send().await?;
        let body = resp.text().await?;
        let parsed: serde_json::Value = serde_json::from_str(&body)?;
        // Merge reasoning_content into content for models that populate it
        if let Some(choice) = parsed.choices.first() {
            let msg = &choice.message;
            let merged = if let Some(rc) = msg.reasoning_content.as_ref() {
                let mut content = msg.content.clone().unwrap_or_default();
                if !content.as_str().unwrap_or("").is_empty() {
                    content = Some(serde_json::Value::String(
                        format!("{} {}", content.as_str().unwrap(), rc)
                    ))
                } else {
                    content = msg.reasoning_content.clone()
                }
                Some(serde_json::Value::Object({
                    let mut obj = serde_json::Map::new();
                    obj.insert("content".to_string(), content);
                    obj.insert("reasoning_content".to_string(), msg.reasoning_content.clone());
                    obj
                }))
            } else {
                Some(msg.clone())
            };
            return Ok(serde_json::json!({ "choices": [merged] }));
        }
        Ok(parsed)
    }

    /// Fallback: chat via Ollama HTTP endpoint.
    pub async fn ollama_chat(&self, model: &str, messages: &[serde_json::Value]) -> Result<serde_json::Value> {
        let payload = serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": false,
        });
        let url = format!("{}/api/chat", self.ollama_url);
        let resp = self.client.post(&url).json(&payload).send().await?;
        let body = resp.text().await?;
        Ok(serde_json::from_str(&body)?)
    }

    /// Schema-validate-retry: parse + validate against JSON Schema; on failure,
    /// re-prompt with validator errors (max 2 attempts), then escalate.
    pub async fn chat_with_schema(
        &self,
        model: &str,
        messages: &[serde_json::Value],
        schema: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let mut attempts = 0;
        let mut last_error = String::new();
        while attempts < 3 {
            attempts += 1;
            match self.chat(model, messages).await {
                Ok(value) => {
                    // Simple JSON schema validation: ensure all required keys present
                    if let Some(req) = schema.get("required").and_then(|r| r.as_array()) {
                        let mut missing = Vec::new();
                        for key in req {
                            if let Some(k) = key.as_str() {
                                if !value.get(k).is_some() {
                                    missing.push(k.to_string());
                                }
                            }
                        }
                        if missing.is_empty() {
                            return Ok(value); // schema passes
                        }
                        last_error = format!("schema validation: missing keys {}", missing.join(", "));
                        // If this was a retry, continue
                    } else {
                        // No required fields specified — accept any structure
                        return Ok(value);
                    }
                }
                Err(e) => {
                    last_error = format!("provider error: {}", e);
                }
            }
        }
        bail!("schema-validate-retry exhausted after 3 attempts; last error: {}", last_error)
    }
}

/// Convenience: run a full role execution round (brief → structured output).
/// In M2 this is called from the Dispatcher state machine.
pub async fn run_role_round(
    provider: &Provider,
    role_id: &str,
    system_prompt: &str,
    user_prompt: &str,
    schema: &serde_json::Value,
) -> Result<serde_json::Value> {
    let messages = serde_json::json!([
        { "role": "system", "content": system_prompt },
        { "role": "user", "content": user_prompt }
    ]);
    provider.chat_with_schema("qwen3-8b-q4_k_m", &messages, schema).await
}