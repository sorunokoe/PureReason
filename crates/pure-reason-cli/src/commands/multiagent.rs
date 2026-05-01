use anyhow::Result;
use clap::{Args, Subcommand};
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Table};
use pure_reason_core::multiagent::MultiAgentBus;

/// Multi-agent epistemic bus commands (TRIZ B-1).
#[derive(Args)]
pub struct MultiAgentCmd {
    #[command(subcommand)]
    pub action: MultiAgentAction,
}

#[derive(Subcommand)]
pub enum MultiAgentAction {
    /// Check a set of agent outputs for cross-agent conflicts.
    ///
    /// Provide agent outputs as `agent-id:text` pairs.
    Check {
        /// Agent outputs in `id:text` format.
        #[clap(required = true, value_parser = parse_agent_arg)]
        agents: Vec<(String, String)>,
    },
}

fn parse_agent_arg(s: &str) -> std::result::Result<(String, String), String> {
    let (id, text) = s
        .split_once(':')
        .ok_or_else(|| format!("Expected format 'agent-id:text', got: '{s}'"))?;
    Ok((id.trim().to_string(), text.trim().to_string()))
}

impl MultiAgentCmd {
    pub async fn run(&self, format: &str) -> Result<()> {
        match &self.action {
            MultiAgentAction::Check { agents } => {
                let mut bus = MultiAgentBus::new();
                for (id, text) in agents {
                    bus.register(id, text).map_err(|e| anyhow::anyhow!(e))?;
                }
                let summary = bus.summarize();

                match format {
                    "json" => println!("{}", serde_json::to_string_pretty(&summary)?),
                    _ => {
                        println!("{}", "━".repeat(60).dimmed());
                        println!(
                            "{} {} agents  {}  {}",
                            "MultiAgent Bus:".bold(),
                            summary.agent_count,
                            format!("Overall Risk: {}", summary.overall_risk).bold(),
                            if summary.conflict_count == 0 {
                                "No conflicts".green().to_string()
                            } else {
                                format!("{} conflict(s)", summary.conflict_count)
                                    .red()
                                    .to_string()
                            }
                        );
                        println!();

                        // Agent table
                        let mut agent_table = Table::new();
                        agent_table.load_preset(UTF8_FULL);
                        agent_table.set_header(["Agent", "Risk", "Illusions", "Contradictions"]);
                        for a in &summary.agents {
                            agent_table.add_row([
                                a.id.as_str(),
                                &format!("{}", a.risk),
                                if a.has_illusions { "Yes" } else { "No" },
                                if a.has_contradictions { "Yes" } else { "No" },
                            ]);
                        }
                        println!("{agent_table}");

                        if !summary.conflicts.is_empty() {
                            println!();
                            println!("{}", "Conflicts:".bold().underline());
                            for c in &summary.conflicts {
                                let severity = format!("severity {:.1}", c.severity);
                                println!(
                                    "  {} {} vs {} [{severity}]",
                                    "⚠".yellow(),
                                    c.agent_a.cyan(),
                                    c.agent_b.cyan()
                                );
                                println!("    {}", c.description);
                            }
                        } else {
                            println!();
                            println!("  {} All agents are epistemically consistent.", "✓".green());
                        }

                        println!("{}", "━".repeat(60).dimmed());
                    }
                }
            }
        }
        Ok(())
    }
}
