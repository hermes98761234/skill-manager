mod cli;
mod config;
mod agent;
mod skill;
mod scanner;
mod ops;
mod display;

use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;

use crate::agent::Agent;
use crate::config::Config;
use crate::display::{print_skill_detail, print_skills_table};
use crate::scanner::scan_agent;
use crate::scanner::scan_all;
use crate::skill::{Skill, SkillStatus};

/// Resolve a skill by name, optionally scoped to an agent.
/// Exits on not-found or ambiguity (multiple agents have same name).
fn resolve_skill(skill_name: &str, agent_filter: Option<&str>) -> anyhow::Result<Skill> {
    let config = Config::load().context("failed to load config")?;
    let agents: Vec<Agent> = config
        .agents
        .iter()
        .filter(|ac| {
            agent_filter
                .map(|f| ac.name == f)
                .unwrap_or(true)
        })
        .map(Agent::from_config)
        .collect();

    let all_skills = scan_all(&agents);
    let matches: Vec<_> = all_skills
        .into_iter()
        .filter(|s| s.name == skill_name)
        .collect();

    match matches.len() {
        0 => {
            eprintln!(
                "{} skill '{}' not found",
                "error:".red().bold(),
                skill_name
            );
            std::process::exit(1);
        }
        1 => Ok(matches.into_iter().next().unwrap()),
        _ => {
            eprintln!(
                "Multiple matches for '{}'. Use --agent to disambiguate:",
                skill_name
            );
            for s in &matches {
                eprintln!("  {}  ({})", s.name, s.agent);
            }
            std::process::exit(1);
        }
    }
}

fn main() -> Result<()> {
    let args = cli::Cli::parse();
    args.run()
}

/// Handle `sm list`
fn handle_list(agent_filter: Option<&str>, status_filter: &str) -> anyhow::Result<()> {
    let config = Config::load().context("failed to load config")?;
    let agents: Vec<Agent> = config
        .agents
        .iter()
        .filter(|ac| {
            agent_filter
                .map(|f| ac.name == f)
                .unwrap_or(true)
        })
        .map(Agent::from_config)
        .collect();

    let all_skills = scan_all(&agents);

    let filtered: Vec<_> = all_skills
        .into_iter()
        .filter(|s| match status_filter {
            "enabled" => s.status == SkillStatus::Enabled,
            "disabled" => s.status == SkillStatus::Disabled,
            _ => true, // "all"
        })
        .collect();

    print_skills_table(&filtered);
    Ok(())
}

/// Handle `sm show <name>`
fn handle_show(skill_name: &str, agent_filter: Option<&str>) -> anyhow::Result<()> {
    let config = Config::load().context("failed to load config")?;
    let agents: Vec<Agent> = config
        .agents
        .iter()
        .filter(|ac| {
            agent_filter
                .map(|f| ac.name == f)
                .unwrap_or(true)
        })
        .map(Agent::from_config)
        .collect();

    let all_skills = scan_all(&agents);

    let matches: Vec<_> = all_skills
        .into_iter()
        .filter(|s| s.name == skill_name)
        .collect();

    match matches.len() {
        0 => {
            eprintln!(
                "{} skill '{}' not found",
                "error:".red().bold(),
                skill_name
            );
            std::process::exit(1);
        }
        1 => {
            print_skill_detail(&matches[0]);
        }
        _ => {
            eprintln!(
                "Multiple matches for '{}'. Use --agent to disambiguate:",
                skill_name
            );
            for s in &matches {
                eprintln!("  {}  ({})", s.name, s.agent);
            }
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Handle `sm move <name> --from <agent> --to <agent>`
fn handle_move(skill_name: &str, from: &str, to: &str, force: bool) -> anyhow::Result<()> {
    let config = Config::load().context("failed to load config")?;

    // Resolve source agent
    let from_config = config
        .agents
        .iter()
        .find(|a| a.name == from)
        .with_context(|| format!("agent '{}' not found. Use 'sm agents add' to register it.", from))?;
    let from_agent = Agent::from_config(from_config);

    // Resolve target agent
    let to_config = config
        .agents
        .iter()
        .find(|a| a.name == to)
        .with_context(|| format!("agent '{}' not found. Use 'sm agents add' to register it.", to))?;
    let to_agent = Agent::from_config(to_config);

    // Find the skill in the source agent
    let skills = scan_agent(&from_agent);
    let skill = skills
        .iter()
        .find(|s| s.name == skill_name)
        .with_context(|| format!("skill '{}' not found in agent '{}'", skill_name, from))?;

    // Confirm with user unless --force
    if !force {
        eprint!(
            "Move {} from {} to {}? [y/N] ",
            skill_name, from, to
        );
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    ops::move_skill(skill, &to_agent)
}

/// Handle `sm copy <name> --from <agent> --to <agent>`
fn handle_copy(skill_name: &str, from: &str, to: &str) -> anyhow::Result<()> {
    let config = Config::load().context("failed to load config")?;

    // Resolve source agent
    let from_config = config
        .agents
        .iter()
        .find(|a| a.name == from)
        .with_context(|| format!("agent '{}' not found. Use 'sm agents add' to register it.", from))?;
    let from_agent = Agent::from_config(from_config);

    // Resolve target agent
    let to_config = config
        .agents
        .iter()
        .find(|a| a.name == to)
        .with_context(|| format!("agent '{}' not found. Use 'sm agents add' to register it.", to))?;
    let to_agent = Agent::from_config(to_config);

    // Find the skill in the source agent
    let skills = scan_agent(&from_agent);
    let skill = skills
        .iter()
        .find(|s| s.name == skill_name)
        .with_context(|| format!("skill '{}' not found in agent '{}'", skill_name, from))?;

    ops::copy_skill(skill, &to_agent)
}

/// Handle `sm edit <name>`
fn handle_edit(skill_name: &str, agent_filter: Option<&str>) -> anyhow::Result<()> {
    let skill = resolve_skill(skill_name, agent_filter)?;
    ops::edit_skill(&skill)
}

/// Handle `sm remove <name> [--force]`
fn handle_remove(skill_name: &str, agent_filter: Option<&str>, force: bool) -> anyhow::Result<()> {
    let skill = resolve_skill(skill_name, agent_filter)?;

    if !force {
        eprint!("Remove {} from {}? [y/N] ", skill.name, skill.agent);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    ops::remove_skill(&skill)
}

/// Handle `sm disable <name>`
fn handle_disable(skill_name: &str, agent_filter: Option<&str>) -> anyhow::Result<()> {
    let skill = resolve_skill(skill_name, agent_filter)?;
    ops::disable_skill(&skill)
}

/// Handle `sm enable <name>`
fn handle_enable(skill_name: &str, agent_filter: Option<&str>) -> anyhow::Result<()> {
    let skill = resolve_skill(skill_name, agent_filter)?;
    ops::enable_skill(&skill)
}
