use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;

pub struct Provider {
    client: Client,
    base_url: String,
    default_model: String,
}

impl Provider {
    pub fn new(base_url: String, default_model: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(300))
            .connect_timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            default_model,
        })
    }

    pub async fn health(&self) -> Result<()> {
        let response = self
            .client
            .get(format!("{}/health", self.base_url))
            .send()
            .await?;
        if !response.status().is_success() {
            bail!("router health returned {}", response.status());
        }
        Ok(())
    }

    pub async fn chat(
        &self,
        model: Option<&str>,
        messages: &[Value],
        max_tokens: u32,
    ) -> Result<String> {
        let payload = json!({
            "model": model.unwrap_or(&self.default_model),
            "messages": messages,
            "temperature": 0.1,
            "max_tokens": max_tokens,
            "stream": false
        });
        let response = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .json(&payload)
            .send()
            .await?;
        let status = response.status();
        let body: Value = response.json().await.context("invalid router JSON")?;
        if !status.is_success() {
            bail!("router returned {}: {}", status, body);
        }
        let message = body
            .get("choices")
            .and_then(|v| v.get(0))
            .and_then(|v| v.get("message"))
            .context("router response has no message")?;
        let content = message.get("content").and_then(Value::as_str).unwrap_or("");
        let reasoning = message
            .get("reasoning_content")
            .and_then(Value::as_str)
            .unwrap_or("");
        let merged = if content.is_empty() {
            reasoning
        } else if reasoning.is_empty() {
            content
        } else {
            &format!("{}\n{}", reasoning, content)
        };
        if merged.trim().is_empty() {
            bail!("router returned empty content");
        }
        Ok(merged.to_string())
    }
}
