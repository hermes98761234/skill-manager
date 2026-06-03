use std::fs;
use std::path::PathBuf;

use crate::agent::Agent;
use crate::skill::{Skill, SkillFrontmatter, SkillStatus};

/// Scan all skills for a single agent.
/// Walks agent.skills_path; each immediate subdirectory is one skill.
/// Parses SKILL.md frontmatter (graceful if missing or bad).
/// Detects .disabled suffix → SkillStatus::Disabled.
pub fn scan_agent(agent: &Agent) -> Vec<Skill> {
    let mut skills = Vec::new();

    let entries = match fs::read_dir(&agent.skills_path) {
        Ok(e) => e,
        Err(_) => return skills,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let dir_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        let (name, status) = if dir_name.ends_with(".disabled") {
            (
                dir_name.trim_end_matches(".disabled").to_string(),
                SkillStatus::Disabled,
            )
        } else {
            (dir_name.clone(), SkillStatus::Enabled)
        };

        let frontmatter = parse_skill_md(&path);

        skills.push(Skill {
            name,
            agent: agent.name.clone(),
            path,
            frontmatter,
            status,
        });
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    skills
}

/// Scan skills for all agents.
pub fn scan_all(agents: &[Agent]) -> Vec<Skill> {
    let mut all = Vec::new();
    for agent in agents {
        all.extend(scan_agent(agent));
    }
    all
}

/// Parse SKILL.md in the given skill directory.
/// Returns default frontmatter if file is missing or unparseable.
fn parse_skill_md(dir: &PathBuf) -> SkillFrontmatter {
    let skill_md = dir.join("SKILL.md");
    match fs::read_to_string(&skill_md) {
        Ok(content) => SkillFrontmatter::parse(&content),
        Err(_) => SkillFrontmatter::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_agent(name: &str, path: &str) -> Agent {
        Agent {
            name: name.to_string(),
            skills_path: PathBuf::from(path),
        }
    }

    #[test]
    fn test_scan_nonexistent_dir() {
        let agent = make_agent("test", "/tmp/no_such_skill_dir_99999");
        let skills = scan_agent(&agent);
        assert!(skills.is_empty());
    }

    #[test]
    fn test_scan_agent_with_skills() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path();

        // Create two skill dirs
        fs::create_dir_all(base.join("skill-a")).unwrap();
        fs::create_dir_all(base.join("skill-b.disabled")).unwrap();

        // Write SKILL.md in one
        fs::write(
            base.join("skill-a").join("SKILL.md"),
            "---\nname: Skill A\ndescription: A test skill\n---\n",
        )
        .unwrap();

        let agent = make_agent("test", &base.to_string_lossy());
        let skills = scan_agent(&agent);

        assert_eq!(skills.len(), 2);

        // skill-a should be enabled with frontmatter
        let a = skills.iter().find(|s| s.name == "skill-a").unwrap();
        assert_eq!(a.status, SkillStatus::Enabled);
        assert_eq!(a.frontmatter.name, Some("Skill A".to_string()));

        // skill-b should be disabled, name stripped
        let b = skills.iter().find(|s| s.name == "skill-b").unwrap();
        assert_eq!(b.status, SkillStatus::Disabled);
    }

    #[test]
    fn test_scan_ignores_files() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path();

        fs::create_dir_all(base.join("real-skill")).unwrap();
        fs::write(base.join("not-a-skill.txt"), "hello").unwrap();

        let agent = make_agent("test", &base.to_string_lossy());
        let skills = scan_agent(&agent);

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "real-skill");
    }
}
