use colored::*;
use comfy_table::{Attribute, Cell, ContentArrangement, Table};

use crate::skill::{Skill, SkillStatus};

/// Truncate a string to at most `max` chars, appending "..." if truncated.
fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max - 3])
    } else {
        s.to_string()
    }
}

/// Print a colored table of skills.
/// Columns: AGENT | NAME | STATUS | DESCRIPTION (truncated to 60 chars)
/// Green rows for enabled, dim/gray for disabled.
pub fn print_skills_table(skills: &[Skill]) {
    if skills.is_empty() {
        println!("{}", "No skills found.".yellow());
        return;
    }

    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec![
        Cell::new("AGENT").add_attribute(Attribute::Bold).fg(comfy_table::Color::Cyan),
        Cell::new("NAME").add_attribute(Attribute::Bold).fg(comfy_table::Color::Cyan),
        Cell::new("STATUS").add_attribute(Attribute::Bold).fg(comfy_table::Color::Cyan),
        Cell::new("DESCRIPTION").add_attribute(Attribute::Bold).fg(comfy_table::Color::Cyan),
    ]);

    for skill in skills {
        let desc = skill
            .frontmatter
            .description
            .as_deref()
            .unwrap_or("")
            .to_string();
        let desc_truncated = truncate(&desc, 60);

        let (status_str, row_dimmed) = match skill.status {
            SkillStatus::Enabled => ("Enabled".to_string(), false),
            SkillStatus::Disabled => ("Disabled".to_string(), true),
        };

        let agent_cell = if row_dimmed {
            Cell::new(&skill.agent).fg(comfy_table::Color::Grey)
        } else {
            Cell::new(&skill.agent)
        };
        let name_cell = if row_dimmed {
            Cell::new(&skill.name).fg(comfy_table::Color::Grey)
        } else {
            Cell::new(&skill.name).fg(comfy_table::Color::Green)
        };
        let status_cell = if row_dimmed {
            Cell::new(&status_str).fg(comfy_table::Color::Grey)
        } else {
            Cell::new(&status_str).fg(comfy_table::Color::Green)
        };
        let desc_cell = if row_dimmed {
            Cell::new(&desc_truncated).fg(comfy_table::Color::Grey)
        } else {
            Cell::new(&desc_truncated)
        };

        table.add_row(vec![agent_cell, name_cell, status_cell, desc_cell]);
    }

    println!("{table}");
}

/// Print full detail view of a single skill in a box-style layout with colored labels.
pub fn print_skill_detail(skill: &Skill) {
    let width = 60;
    let border = "─".repeat(width);
    let top = format!("┌{}┐", border);
    let bottom = format!("└{}┘", border);

    println!("{}", top.cyan());
    println!("│ {} │", format!("{:^width$}", skill.name.bold(), width = width - 2));
    println!("{}", format!("├{}┤", border).cyan());

    print_field("Agent", &skill.agent, width);
    print_field("Name", &skill.name, width);

    let status_str = match skill.status {
        SkillStatus::Enabled => "Enabled".to_string(),
        SkillStatus::Disabled => "Disabled".to_string(),
    };
    print_field("Status", &status_str, width);

    if let Some(ref desc) = skill.frontmatter.description {
        print_field("Description", desc, width);
    } else {
        print_field("Description", "(none)", width);
    }

    print_field("Path", &skill.path.display().to_string(), width);

    println!("{}", bottom.cyan());
}

fn print_field(label: &str, value: &str, width: usize) {
    let label_fmt = format!("  {}:", label.bold());
    let val_fmt = format!(" {}", value);
    let total = label_fmt.len() + val_fmt.len();
    let padding = if total < width { width - total } else { 1 };
    println!("│{}{}{} │", label_fmt.cyan(), val_fmt, " ".repeat(padding));
}
