use anyhow::{bail, Context, Result};
use std::path::Path;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct ProviderSpec {
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub default_model: String,
    #[serde(default)]
    pub api_key: Option<String>,
}

#[derive(Clone, Debug, serde::Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Local,
    Hybrid,
    Cloud,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct RoleRoute {
    #[serde(default)]
    pub mode: Mode,
    #[serde(default = "default_local")]
    pub local_provider: String,
    #[serde(default)]
    pub local_model: String,
    #[serde(default = "default_cloud")]
    pub cloud_provider: String,
    #[serde(default)]
    pub cloud_model: String,
}

fn default_local() -> String {
    "local".to_string()
}
fn default_cloud() -> String {
    "openrouter".to_string()
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct ModelConfig {
    pub providers: std::collections::HashMap<String, ProviderSpec>,
    #[serde(default)]
    pub mode: Mode,
    #[serde(default)]
    pub roles: std::collections::HashMap<String, RoleRoute>,
}

impl ModelConfig {
    pub fn load(path: &Path, secrets_path: &Path, env: &[(&str, String)]) -> Result<Self> {
        if !path.exists() {
            bail!("no model config at {}", path.display());
        }
        let text = std::fs::read_to_string(path)?;
        let mut config: ModelConfig = toml::from_str(&text).context("invalid aiorg.toml")?;
        if secrets_path.exists() {
            let secrets_text = std::fs::read_to_string(secrets_path)?;
            let secrets: std::collections::HashMap<String, ProviderSpec> =
                toml::from_str(&secrets_text).context("invalid secrets.toml")?;
            for (key, spec) in secrets {
                if let Some(existing) = config.providers.get_mut(&key) {
                    existing.api_key = spec.api_key;
                }
            }
        }
        // Env overrides: AIORG_ROUTER_URL / AIORG_MODEL / AIORG_PROVIDER_API_KEY
        for (env_key, value) in env {
            if value.is_empty() {
                continue;
            }
            match *env_key {
                "AIORG_ROUTER_URL" => {
                    if let Some(p) = config.providers.get_mut("local") {
                        p.base_url = value.clone();
                    }
                }
                "AIORG_MODEL" => {
                    if let Some(p) = config.providers.get_mut("local") {
                        p.default_model = value.clone();
                    }
                }
                _ => {}
            }
        }
        Ok(config)
    }

    pub fn cloud_providers(&self) -> Vec<(&str, &ProviderSpec)> {
        self.providers
            .iter()
            .filter(|(name, _)| name.as_str() != "local")
            .filter(|(_, spec)| !spec.base_url.is_empty())
            .map(|(name, spec)| (name.as_str(), spec))
            .collect()
    }
}
