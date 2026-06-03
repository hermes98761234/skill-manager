use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};

use crate::agent::Agent;
use crate::config::*;
use crate::scanner::scan_agent;

#[derive(Parser)]
#[command(name = "sm", version, about = "Manage AI agent skills")]
pub struct Cli {
    /// Target a specific agent
    #[arg(short, long, global = true)]
    pub agent: Option<String>,

    /// Output as JSON
    #[arg(short, long, global = true)]
    pub json: bool,

    /// Show detailed paths and metadata
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress non-essential output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Show what would happen without making changes
    #[arg(long, global = true)]
    pub dry_run: bool,

    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    pub fn run(self) -> anyhow::Result<()> {
        match self.command {
            Commands::List { agent, status } => {
                crate::handle_list(agent.as_deref(), &status)
            }
            Commands::Show { skill_name, agent } => {
                crate::handle_show(&skill_name, agent.as_deref())
            }
            Commands::Move {
                skill_name,
                from,
                to,
                force,
            } => crate::handle_move(&skill_name, &from, &to, force),
            Commands::Copy {
                skill_name,
                from,
                to,
            } => crate::handle_copy(&skill_name, &from, &to),
            Commands::Edit { skill_name, agent } => {
                crate::handle_edit(&skill_name, agent.as_deref())
            }
            Commands::Remove { skill_name, agent, force } => {
                crate::handle_remove(&skill_name, agent.as_deref(), force)
            }
            Commands::Disable { skill_name, agent } => {
                crate::handle_disable(&skill_name, agent.as_deref())
            }
            Commands::Enable { skill_name, agent } => {
                crate::handle_enable(&skill_name, agent.as_deref())
            }
            Commands::Agents { agent_cmd } => agent_cmd.run(),
        }
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// List skills
    List {
        #[arg(short, long)]
        agent: Option<String>,
        /// Filter by status: enabled, disabled, all
        #[arg(long, default_value = "enabled")]
        status: String,
    },
    /// Show skill details
    Show {
        skill_name: String,
        #[arg(short, long)]
        agent: Option<String>,
    },
    /// Move a skill between agents
    Move {
        skill_name: String,
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
        /// Skip confirmation
        #[arg(long)]
        force: bool,
    },
    /// Copy a skill to another agent
    Copy {
        skill_name: String,
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
    },
    /// Open SKILL.md in $EDITOR
    Edit {
        skill_name: String,
        #[arg(short, long)]
        agent: Option<String>,
    },
    /// Permanently delete a skill
    Remove {
        skill_name: String,
        #[arg(short, long)]
        agent: Option<String>,
        /// Skip confirmation
        #[arg(long)]
        force: bool,
    },
    /// Disable a skill
    Disable {
        skill_name: String,
        #[arg(short, long)]
        agent: Option<String>,
    },
    /// Enable a disabled skill
    Enable {
        skill_name: String,
        #[arg(short, long)]
        agent: Option<String>,
    },
    /// Manage agent registry
    Agents {
        #[command(subcommand)]
        agent_cmd: AgentCommands,
    },
}

#[derive(Subcommand)]
pub enum AgentCommands {
    /// List registered agents,
    List,
    /// Register an agent,
    Add {
        name: String,
        path: String,
    },
    /// Remove an agent,
    Remove {
        name: String,
    },
}

impl AgentCommands {
    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            AgentCommands::List => {
                let config = Config::load().context("failed to load config")?;
                if config.agents.is_empty() {
                    println!("No agents registered.");
                    return Ok(());
                }
                // Print a simple table: name | path | skill count
                println!("{:<15} {:<40} {:<12}", "NAME", "SKILLS PATH", "SKILLS");
                println!("{}", "-".repeat(67));
                for ac in &config.agents {
                    let agent = Agent::from_config(ac);
                    let count = scan_agent(&agent).len();
                    println!(
                        "{:<15} {:<40} {:<12}",
                        agent.name,
                        agent.skills_path.display(),
                        count
                    );
                }
                Ok(())
            }
            AgentCommands::Add { name, path } => {
                let mut config = Config::load().context("failed to load config")?;
                let abs_path = PathBuf::from(path);
                config.add_agent(name, abs_path);
                config.save().context("failed to save config")?;
                println!("Added agent '{}' with path '{}'.", name, path);
                Ok(())
            }
            AgentCommands::Remove { name } => {
                let mut config = Config::load().context("failed to load config")?;
                config.remove_agent(name);
                config.save().context("failed to save config")?;
                println!("Removed agent '{}'.", name);
                Ok(())
            }
        }
    }
}
