# skill-manager

**A unified CLI for managing skills across AI coding agents.**

[![Build Status](https://img.shields.io/github/actions/workflow/status/hermes98761234/skill-manager/ci.yml?branch=main)](https://github.com/hermes98761234/skill-manager)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

---

## Why?

AI coding agents like Claude Code and Hermes each have their own skill directory — different paths, different formats, no way to move or share skills between them. **skill-manager** (`sm`) gives you one CLI that talks to all of them: list, inspect, move, copy, enable, disable, and remove skills across any registered agent.

## Supported Agents

| Agent | Default Skills Path | Auto-detected |
|-------|---------------------|---------------|
| Claude Code | `~/.claude/skills/` | Yes |
| Hermes | `~/.hermes/skills/` | Yes |
| Any other | Configurable | Via `sm agents add` |

## Installation

```bash
# Build from source (until published to crates.io)
git clone https://github.com/hermes98761234/skill-manager.git
cd skill-manager
cargo install --path .
```

Requires Rust 1.70+ and Cargo.

## Quick Start

```bash
# List all skills across all agents
$ sm list
 NAME              AGENT    STATUS
 writing-plans     claude   enabled
 code-review       claude   enabled
 debugging         hermes   enabled
 git-workflow      hermes   disabled

# Show details for a specific skill
$ sm show debugging
 Name:     debugging
 Agent:    hermes
 Status:   enabled
 Path:     /home/user/.hermes/skills/debugging/SKILL.md
 Size:     3.2 KB

# Move a skill from Claude to Hermes
$ sm move code-review --from claude --to hermes
 Move code-review from claude to hermes? [y/N] y
 Moved code-review → hermes

# Copy instead of move (keeps original)
$ sm copy debugging --from hermes --to claude

# Temporarily disable a skill
$ sm disable git-workflow
 Disabled git-workflow (hermes)

# Re-enable it
$ sm enable git-workflow
 Enabled git-workflow (hermes)

# Open a skill in your editor
$ sm edit debugging        # opens in $EDITOR / vim

# Remove a skill (with confirmation)
$ sm remove old-skill
 Remove old-skill from hermes? [y/N] y
 Removed old-skill
```

## Command Reference

| Command | Description |
|---------|-------------|
| `sm list [--agent <name>] [--status <filter>]` | List skills. `--status` accepts `enabled`, `disabled`, or `all` (default: `enabled`) |
| `sm show <name> [--agent <name>]` | Show detailed info for a skill |
| `sm move <name> --from <agent> --to <agent> [--force]` | Move a skill between agents (removes from source) |
| `sm copy <name> --from <agent> --to <agent>` | Copy a skill to another agent (keeps original) |
| `sm edit <name> [--agent <name>]` | Open SKILL.md in `$EDITOR` (defaults to vim) |
| `sm disable <name> [--agent <name>]` | Disable a skill (renames to `.disabled` suffix) |
| `sm enable <name> [--agent <name>]` | Re-enable a disabled skill |
| `sm remove <name> [--agent <name>] [--force]` | Delete a skill permanently |
| `sm agents list` | Show all registered agents with skill counts |
| `sm agents add <name> <path>` | Register a new agent with its skills directory |
| `sm agents remove <name>` | Unregister an agent |

### Global Flags

| Flag | Description |
|------|-------------|
| `-a, --agent <name>` | Scope command to a specific agent |
| `-j, --json` | Output as JSON |
| `-v, --verbose` | Show detailed paths and metadata |
| `-q, --quiet` | Suppress non-essential output |
| `--dry-run` | Preview changes without executing them |

## Configuration

skill-manager reads `~/.config/skill-manager/agents.toml`. If the file doesn't exist, it auto-generates one with Claude Code and Hermes defaults.

```toml
[[agent]]
name = "claude"
skills_path = "/home/user/.claude/skills"

[[agent]]
name = "hermes"
skills_path = "/home/user/.hermes/skills"

# Add any agent with a skills/ directory
[[agent]]
name = "copilot"
skills_path = "/home/user/.copilot/skills"
```

Each `[[agent]]` entry needs:
- **`name`** — a short identifier used in `--agent` flags and display output
- **`skills_path`** — absolute path to the agent's skills directory (where subdirectories contain `SKILL.md` files)

## How It Works

- **Scanning** — Walks each agent's `skills_path`, reading `SKILL.md` frontmatter (YAML) for metadata.
- **Move** — Uses `fs::rename` (atomic on same filesystem). Prompts unless `--force`.
- **Copy** — Recursively copies the skill directory to the target agent.
- **Disable** — Appends `.disabled` suffix to the skill directory (agent ignores it; `sm` still sees it).
- **Enable** — Strips the `.disabled` suffix. Bails if the basename already exists.
- **Ambiguity** — When a skill name exists in multiple agents, `sm` prompts you to use `--agent` to disambiguate.

## License

MIT
