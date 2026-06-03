use std::path::PathBuf;

use crate::config::AgentConfig;

/// Represents a resolved agent with its configuration.
#[derive(Debug, Clone)]
pub struct Agent {
    pub name: String,
    pub skills_path: PathBuf,
}

impl Agent {
    /// Create an Agent from an AgentConfig.
    /// Warns if skills_path does not exist, but does not crash.
    pub fn from_config(cfg: &AgentConfig) -> Self {
        let skills_path = cfg.skills_path.clone();
        if !skills_path.exists() {
            eprintln!(
                "warning: skills path '{}' for agent '{}' does not exist",
                skills_path.display(),
                cfg.name
            );
        }
        Agent {
            name: cfg.name.clone(),
            skills_path,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AgentConfig;

    #[test]
    fn test_from_config() {
        let cfg = AgentConfig {
            name: "test".to_string(),
            skills_path: PathBuf::from("/tmp/nonexistent_path_12345"),
        };
        let agent = Agent::from_config(&cfg);
        assert_eq!(agent.name, "test");
        assert_eq!(agent.skills_path, PathBuf::from("/tmp/nonexistent_path_12345"));
    }

    #[test]
    fn test_from_config_existing_path() {
        let cfg = AgentConfig {
            name: "tmp".to_string(),
            skills_path: PathBuf::from("/tmp"),
        };
        let agent = Agent::from_config(&cfg);
        assert_eq!(agent.name, "tmp");
        assert!(agent.skills_path.exists());
    }
}
