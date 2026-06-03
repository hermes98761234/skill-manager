use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Top-level config file: ~/.config/skill-manager/agents.toml
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "agent", default)]
    pub agents: Vec<AgentConfig>,
}

/// Single agent entry in the config file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub skills_path: PathBuf,
}

impl Config {
    /// Load config from the default path, or return a default config
    /// with built-in agents (claude, hermes).
    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default_config())
        }
    }

    /// Save config to the default path.
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Add an agent to the config.
    pub fn add_agent(&mut self, name: &str, path: PathBuf) {
        if !self.agents.iter().any(|a| a.name == name) {
            self.agents.push(AgentConfig {
                name: name.to_string(),
                skills_path: path,
            });
        }
    }

    /// Remove an agent from the config by name.
    pub fn remove_agent(&mut self, name: &str) {
        self.agents.retain(|a| a.name != name);
    }

    fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("skill-manager")
            .join("agents.toml")
    }

    /// Default config with built-in agents.
    fn default_config() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Config {
            agents: vec![
                AgentConfig {
                    name: "claude".to_string(),
                    skills_path: home.join(".claude").join("skills"),
                },
                AgentConfig {
                    name: "hermes".to_string(),
                    skills_path: home.join(".hermes").join("skills"),
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_has_builtins() {
        let config = Config::default_config();
        assert_eq!(config.agents.len(), 2);
        assert_eq!(config.agents[0].name, "claude");
        assert_eq!(config.agents[1].name, "hermes");
    }

    #[test]
    fn test_add_agent() {
        let mut config = Config::default();
        config.add_agent("test", PathBuf::from("/tmp/skills"));
        assert_eq!(config.agents.len(), 1);
        assert_eq!(config.agents[0].name, "test");
    }

    #[test]
    fn test_add_agent_no_duplicate() {
        let mut config = Config::default();
        config.add_agent("test", PathBuf::from("/tmp/skills"));
        config.add_agent("test", PathBuf::from("/other"));
        assert_eq!(config.agents.len(), 1);
    }

    #[test]
    fn test_remove_agent() {
        let mut config = Config::default();
        config.add_agent("test", PathBuf::from("/tmp/skills"));
        config.add_agent("other", PathBuf::from("/tmp/other"));
        config.remove_agent("test");
        assert_eq!(config.agents.len(), 1);
        assert_eq!(config.agents[0].name, "other");
    }
}
