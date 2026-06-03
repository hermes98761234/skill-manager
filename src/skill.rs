use serde::Deserialize;
use std::path::PathBuf;

/// Parsed YAML frontmatter from SKILL.md.
#[derive(Debug, Default, Clone, Deserialize, PartialEq)]
pub struct SkillFrontmatter {
    pub name: Option<String>,
    pub description: Option<String>,
    pub disabled: Option<bool>,
}

/// Skill enable/disable status.
#[derive(Debug, Clone, PartialEq)]
pub enum SkillStatus {
    Enabled,
    Disabled,
}

/// A single skill discovered from an agent's skill directory.
#[derive(Debug, Clone)]
pub struct Skill {
    /// Skill name (dir name, stripped of .disabled suffix)
    pub name: String,
    /// Agent this skill belongs to
    pub agent: String,
    /// Full path to the skill directory
    pub path: PathBuf,
    /// Parsed frontmatter from SKILL.md
    pub frontmatter: SkillFrontmatter,
    /// Current status
    pub status: SkillStatus,
}

impl SkillFrontmatter {
    /// Parse frontmatter from SKILL.md content.
    /// Strips the --- delimiters and deserializes the YAML.
    pub fn parse(content: &str) -> Self {
        // Find the first ---
        let Some(start) = content.find("---") else {
            return Self::default();
        };
        // Skip past the first ---
        let after_first = &content[start + 3..];
        // Find the closing ---
        let Some(end) = after_first.find("---") else {
            return Self::default();
        };
        let yaml_str = &after_first[..end];
        serde_yaml::from_str(yaml_str).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_frontmatter() {
        let content = "---\nname: test-skill\ndescription: A test skill\ndisabled: false\n---\n# Some content\n";
        let fm = SkillFrontmatter::parse(content);
        assert_eq!(fm.name, Some("test-skill".to_string()));
        assert_eq!(fm.description, Some("A test skill".to_string()));
        assert_eq!(fm.disabled, Some(false));
    }

    #[test]
    fn test_parse_partial_frontmatter() {
        let content = "---\nname: partial-skill\n---\n# Content\n";
        let fm = SkillFrontmatter::parse(content);
        assert_eq!(fm.name, Some("partial-skill".to_string()));
        assert_eq!(fm.description, None);
    }

    #[test]
    fn test_parse_no_frontmatter() {
        let content = "# Just a markdown file\nNo frontmatter here.\n";
        let fm = SkillFrontmatter::parse(content);
        assert_eq!(fm.name, None);
        assert_eq!(fm.description, None);
        assert_eq!(fm.disabled, None);
    }

    #[test]
    fn test_parse_empty_frontmatter() {
        let content = "---\n---\n# Content\n";
        let fm = SkillFrontmatter::parse(content);
        assert_eq!(fm, SkillFrontmatter::default());
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let content = "---\n: invalid yaml :\n  - broken\n---\n";
        let fm = SkillFrontmatter::parse(content);
        assert_eq!(fm, SkillFrontmatter::default());
    }

    #[test]
    fn test_parse_only_first_frontmatter() {
        let content = "---\nname: first\n---\nContent\n---\nname: second\n---\n";
        let fm = SkillFrontmatter::parse(content);
        assert_eq!(fm.name, Some("first".to_string()));
    }
}
