use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;

pub struct Provider {
    client: Client,
    base_url: String,
    default_model: String,
    api_key: Option<String>,
}

impl Provider {
    pub fn new(base_url: String, default_model: String) -> Result<Self> {
        Self::new_with_key(base_url, default_model, None)
    }

    pub fn new_with_key(
        base_url: String,
        default_model: String,
        api_key: Option<String>,
    ) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(300))
            .connect_timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            default_model,
            api_key,
        })
    }

    pub async fn health(&self) -> Result<()> {
        let response = self
            .client
            .get(format!("{}/v1/models", self.base_url))
            .send()
            .await?;
        if !response.status().is_success() {
            bail!("provider models returned {}", response.status());
        }
        Ok(())
    }

    pub async fn models(&self) -> Result<Vec<String>> {
        let response = self
            .client
            .get(format!("{}/v1/models", self.base_url))
            .send()
            .await?;
        if !response.status().is_success() {
            bail!("provider models returned {}", response.status());
        }
        let body: Value = response.json().await.context("invalid models JSON")?;
        let mut models = Vec::new();
        if let Some(data) = body.get("data").and_then(Value::as_array) {
            for item in data {
                if let Some(id) = item.get("id").and_then(Value::as_str) {
                    models.push(id.to_string());
                }
            }
        }
        Ok(models)
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
        let mut request = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .json(&payload);
        if let Some(key) = &self.api_key {
            request = request.bearer_auth(key);
        }
        let response = request.send().await?;
        let status = response.status();
        let body: Value = response.json().await.context("invalid router JSON")?;
        if !status.is_success() {
            bail!("provider returned {}: {}", status, body);
        }
        let message = body
            .get("choices")
            .and_then(|v| v.get(0))
            .and_then(|v| v.get("message"))
            .context("provider response has no message")?;
        let content = message.get("content").and_then(Value::as_str).unwrap_or("");
        let reasoning = message
            .get("reasoning_content")
            .and_then(Value::as_str)
            .unwrap_or("");
        let merged = if content.is_empty() {
            reasoning
        } else {
            content
        };
        if merged.trim().is_empty() {
            bail!("provider returned empty content");
        }
        Ok(merged.to_string())
    }
}
