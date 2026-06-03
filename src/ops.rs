use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::agent::Agent;
use crate::skill::{Skill, SkillStatus};

/// Recursively copy a directory and all its contents.
pub fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst).with_context(|| format!("failed to create dir {}", dst.display()))?;
    for entry in fs::read_dir(src).with_context(|| format!("failed to read dir {}", src.display()))? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path).with_context(|| {
                format!("failed to copy {} to {}", src_path.display(), dst_path.display())
            })?;
        }
    }
    Ok(())
}

/// Move a skill from one agent to another.
pub fn move_skill(skill: &Skill, target_agent: &Agent) -> Result<()> {
    let dest = target_agent.skills_path.join(&skill.name);
    if dest.exists() {
        bail!(
            "skill '{}' already exists in '{}'",
            skill.name,
            target_agent.name
        );
    }
    fs::rename(&skill.path, &dest).with_context(|| {
        format!(
            "failed to move {} to {}",
            skill.path.display(),
            dest.display()
        )
    })?;
    println!(
        "✓ Moved {} from {} to {}",
        skill.name, skill.agent, target_agent.name
    );
    Ok(())
}

/// Copy a skill to another agent.
pub fn copy_skill(skill: &Skill, target_agent: &Agent) -> Result<()> {
    let dest = target_agent.skills_path.join(&skill.name);
    if dest.exists() {
        bail!(
            "skill '{}' already exists in '{}'",
            skill.name,
            target_agent.name
        );
    }
    copy_dir_all(&skill.path, &dest)?;
    println!("✓ Copied {} to {}", skill.name, target_agent.name);
    Ok(())
}

/// Open SKILL.md in $EDITOR (defaults to nano).
pub fn edit_skill(skill: &Skill) -> Result<()> {
    let editor = env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
    let skill_md = skill.path.join("SKILL.md");
    if !skill_md.exists() {
        bail!("SKILL.md not found at {}", skill_md.display());
    }
    let status = Command::new(&editor)
        .arg(&skill_md)
        .status()
        .with_context(|| format!("failed to launch editor '{}'", editor))?;
    if !status.success() {
        bail!("editor '{}' exited with error", editor);
    }
    println!("✓ Edited {}", skill_md.display());
    Ok(())
}

/// Permanently delete a skill directory.
pub fn remove_skill(skill: &Skill) -> Result<()> {
    fs::remove_dir_all(&skill.path)
        .with_context(|| format!("failed to remove {}", skill.path.display()))?;
    println!("✓ Removed {}", skill.name);
    Ok(())
}

/// Disable a skill by renaming its directory with a .disabled suffix.
pub fn disable_skill(skill: &Skill) -> Result<()> {
    if skill.status == SkillStatus::Disabled {
        bail!("skill '{}' is already disabled", skill.name);
    }
    let new_name = format!("{}.disabled", skill.name);
    let new_path = skill.path.with_file_name(&new_name);
    fs::rename(&skill.path, &new_path)
        .with_context(|| format!("failed to rename to {}", new_path.display()))?;
    println!("✓ Disabled {} (renamed to {})", skill.name, new_name);
    Ok(())
}

/// Enable a disabled skill by stripping the .disabled suffix.
pub fn enable_skill(skill: &Skill) -> Result<()> {
    if skill.status == SkillStatus::Enabled {
        bail!("skill '{}' is already enabled", skill.name);
    }
    let new_path = skill.path.with_file_name(&skill.name);
    fs::rename(&skill.path, &new_path)
        .with_context(|| format!("failed to rename to {}", new_path.display()))?;
    println!("✓ Enabled {}", skill.name);
    Ok(())
}
