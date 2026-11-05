use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;

use crate::config::ProviderConfig;
use crate::config::ProviderRegistry;

/// Hard cap so the settings UI stays a list, not a second provider manager.
pub const MAX_SUB_AGENTS: usize = 8;
pub const MAX_SUB_AGENT_PARALLEL: usize = 5;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SubAgentProfile {
    pub id: String,
    #[serde(default)]
    pub name: String,
    /// Provider selector: `provider_id` or `provider_id:model`.
    #[serde(default)]
    pub selector: String,
    #[serde(default)]
    pub instructions: Option<String>,
    /// Empty means the full tool set; otherwise an allow-list by tool name.
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default = "default_max_parallel")]
    pub max_parallel: usize,
}

const fn default_max_parallel() -> usize {
    2
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SubAgentDocument {
    #[serde(default)]
    pub default: String,
    #[serde(default)]
    pub agents: Vec<SubAgentProfile>,
}

#[derive(Clone, Debug)]
pub struct ResolvedSubAgent {
    pub id: String,
    pub name: String,
    pub provider: ProviderConfig,
    pub instructions: Option<String>,
    pub tools: Vec<String>,
    pub max_parallel: usize,
}

impl SubAgentDocument {
    /// Reads the `subagent_default` / `subagents` fields from `settings.json`.
    /// Missing fields mean "no dedicated profiles", not an error.
    pub fn from_settings(settings: &Value) -> Result<Self> {
        let default = settings
            .get("subagent_default")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_owned();
        let agents = settings
            .get("subagents")
            .cloned()
            .unwrap_or_else(|| Value::Array(Vec::new()));
        let agents = serde_json::from_value(agents)
            .context("settings.subagents must be an array of sub-agent profiles")?;
        Ok(Self { default, agents })
    }

    pub fn validate(&self, registry: &ProviderRegistry) -> Result<()> {
        if self.agents.len() > MAX_SUB_AGENTS {
            bail!("at most {MAX_SUB_AGENTS} sub-agent profiles are supported");
        }
        let mut ids = BTreeSet::new();
        for profile in &self.agents {
            let id = profile.id.trim();
            if id.is_empty() {
                bail!("sub-agent id must not be empty");
            }
            if id.eq_ignore_ascii_case("main") {
                bail!("sub-agent id `main` is reserved for the primary model");
            }
            if !id
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.'))
            {
                bail!("sub-agent id `{id}` may only contain letters, digits, '_', '-', '.'");
            }
            if !ids.insert(id.to_ascii_lowercase()) {
                bail!("duplicate sub-agent id: {id}");
            }
            if profile.selector.trim().is_empty() {
                bail!("sub-agent `{id}` has no model selector");
            }
            registry
                .resolve(Some(profile.selector.trim()))
                .with_context(|| format!("sub-agent `{id}` has an invalid model selector"))?;
            if profile.max_parallel == 0 || profile.max_parallel > MAX_SUB_AGENT_PARALLEL {
                bail!(
                    "sub-agent `{id}` max_parallel must be between 1 and {MAX_SUB_AGENT_PARALLEL}"
                );
            }
        }
        if !self.default.trim().is_empty()
            && !self
                .agents
                .iter()
                .any(|profile| profile.id.eq_ignore_ascii_case(self.default.trim()))
        {
            bail!("default sub-agent `{}` is not defined", self.default.trim());
        }
        Ok(())
    }

    pub fn resolve_all(&self, registry: &ProviderRegistry) -> Result<Vec<ResolvedSubAgent>> {
        self.validate(registry)?;
        self.agents
            .iter()
            .map(|profile| {
                let provider = registry.resolve(Some(profile.selector.trim()))?;
                let id = profile.id.trim().to_owned();
                let name = if profile.name.trim().is_empty() {
                    id.clone()
                } else {
                    profile.name.trim().to_owned()
                };
                Ok(ResolvedSubAgent {
                    id,
                    name,
                    provider,
                    instructions: profile
                        .instructions
                        .as_ref()
                        .map(|value| value.trim().to_owned())
                        .filter(|value| !value.is_empty()),
                    tools: profile
                        .tools
                        .iter()
                        .map(|value| value.trim().to_owned())
                        .filter(|value| !value.is_empty())
                        .collect(),
                    max_parallel: profile.max_parallel.clamp(1, MAX_SUB_AGENT_PARALLEL),
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn registry() -> ProviderRegistry {
        let dir = tempdir().expect("temp dir");
        let path = dir.path().join("providers.json");
        fs::write(
            &path,
            serde_json::json!({
                "active": "main",
                "providers": {
                    "main": {
                        "type": "openai_compatible",
                        "display": "Main",
                        "api_key": "sk-test",
                        "base_url": "https://example.com/v1",
                        "model": "main-model",
                        "fast_model": "fast-model"
                    },
                    "review": {
                        "type": "anthropic_messages",
                        "display": "Review",
                        "api_key": "sk-test",
                        "base_url": "https://example.com",
                        "model": "review-model"
                    }
                }
            })
            .to_string(),
        )
        .expect("write providers");
        ProviderRegistry::load(&path).expect("load providers")
    }

    #[test]
    fn empty_settings_mean_no_dedicated_profiles() {
        let document = SubAgentDocument::from_settings(&serde_json::json!({})).expect("parse");
        assert!(document.agents.is_empty());
        assert!(document.validate(&registry()).is_ok());
        assert!(document.resolve_all(&registry()).expect("resolve").is_empty());
    }

    #[test]
    fn resolves_profile_provider_model_and_tools() {
        let document: SubAgentDocument = serde_json::from_value(serde_json::json!({
            "default": "reviewer",
            "agents": [{
                "id": "reviewer",
                "name": "审查",
                "selector": "review:review-model",
                "instructions": "只读审查",
                "tools": ["read_file", "grep_files"],
                "max_parallel": 1
            }]
        }))
        .expect("parse");
        document.validate(&registry()).expect("valid");
        let resolved = document.resolve_all(&registry()).expect("resolve");
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].id, "reviewer");
        assert_eq!(resolved[0].name, "审查");
        assert_eq!(resolved[0].provider.id, "review");
        assert_eq!(resolved[0].provider.model, "review-model");
        assert_eq!(resolved[0].tools, vec!["read_file", "grep_files"]);
        assert_eq!(resolved[0].max_parallel, 1);
    }

    #[test]
    fn rejects_duplicate_ids_and_unknown_default() {
        let duplicate: SubAgentDocument = serde_json::from_value(serde_json::json!({
            "agents": [
                {"id": "Reviewer", "selector": "main"},
                {"id": "reviewer", "selector": "main"}
            ]
        }))
        .expect("parse");
        assert!(duplicate.validate(&registry()).is_err());

        let unknown_default: SubAgentDocument = serde_json::from_value(serde_json::json!({
            "default": "missing",
            "agents": [{"id": "reviewer", "selector": "main"}]
        }))
        .expect("parse");
        assert!(unknown_default.validate(&registry()).is_err());
    }

    #[test]
    fn rejects_reserved_or_malformed_ids() {
        let reserved: SubAgentDocument = serde_json::from_value(serde_json::json!({
            "agents": [{"id": "main", "selector": "main"}]
        }))
        .expect("parse");
        assert!(reserved.validate(&registry()).is_err());

        let malformed: SubAgentDocument = serde_json::from_value(serde_json::json!({
            "agents": [{"id": "re viewer", "selector": "main"}]
        }))
        .expect("parse");
        assert!(malformed.validate(&registry()).is_err());
    }

    #[test]
    fn rejects_invalid_selector_and_parallel_limit() {
        let invalid_selector: SubAgentDocument = serde_json::from_value(serde_json::json!({
            "agents": [{"id": "reviewer", "selector": "review:unknown-model"}]
        }))
        .expect("parse");
        assert!(invalid_selector.validate(&registry()).is_err());

        let invalid_parallel: SubAgentDocument = serde_json::from_value(serde_json::json!({
            "agents": [{"id": "reviewer", "selector": "main", "max_parallel": 0}]
        }))
        .expect("parse");
        assert!(invalid_parallel.validate(&registry()).is_err());
    }
}
