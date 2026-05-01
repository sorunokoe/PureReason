pub mod analyze;
pub mod antinomy;
pub mod apperceive;
pub mod calibrate;
pub mod categorize;
pub mod certify;
pub mod claims;
pub mod compliance;
pub mod critique;
pub mod feedback;
pub mod game;
pub mod illusion;
pub mod multiagent;
pub mod paralogism;
pub mod pipeline;
pub mod red_team;
pub mod regulate;
pub mod review;
pub mod schema;
pub mod self_audit;
pub mod sla;
pub mod train;
pub mod validate;
pub mod validate_decision;

use anyhow::Result;
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Table};
use pure_reason_core::calibration::PipelineCalibration;
use pure_reason_core::claims::annotation_to_triple;
use pure_reason_core::pipeline::PipelineReport;

/// Print a PipelineReport in the requested format.
pub fn print_report(report: &PipelineReport, format: &str) -> Result<()> {
    match format {
        "json" => {
            let json = report.to_json().map_err(|e| anyhow::anyhow!(e))?;
            println!("{}", json);
        }
        "markdown" => {
            println!("{}", report.to_markdown());
        }
        _ => {
            print_plain_report(report);
        }
    }
    Ok(())
}

fn print_plain_report(report: &PipelineReport) {
    let cal = report.calibration();

    println!("{}", "━".repeat(60).dimmed());

    // ECS — prominent top-level display
    let ecs_display = format_ecs(cal.ecs);
    println!(
        "{} {}  {}  {}",
        "ECS:".bold(),
        ecs_display,
        cal.band.label().bold(),
        cal.band.description().dimmed(),
    );
    println!("{} {}", "Epistemic Mode:".bold(), cal.epistemic_mode.cyan());

    println!();
    println!(
        "{} {}",
        "Input:".bold(),
        report.input.chars().take(80).collect::<String>()
    );
    println!("{} {}", "Risk:".bold(), format_risk(&report.verdict.risk));
    println!(
        "{} {:?}",
        "Epistemic Status:".bold(),
        report.epistemic_status
    );

    if let Some(cat) = &report.verdict.dominant_category {
        println!("{} {}", "Dominant Category:".bold(), cat.cyan());
    }
    if let Some(game) = &report.verdict.primary_language_game {
        println!("{} {}", "Language Game:".bold(), game.cyan());
    }

    println!();
    println!("{}", "Aesthetic:".bold().underline());
    println!(
        "  Tokens: {}  |  Sentences: {}  |  Temporal: {}",
        report.intuition_summary.token_count,
        report.intuition_summary.sentence_count,
        report.intuition_summary.temporal_orientation,
    );

    println!();
    println!("{}", "Analytic — Categories:".bold().underline());
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(["Category", "Confidence", "Evidence"]);
    for app in report.understanding.category_analysis.above_threshold(0.01) {
        table.add_row([
            app.category.name(),
            &format!("{:.2}", app.confidence.value()),
            &app.evidence.join(", "),
        ]);
    }
    println!("{table}");

    println!();
    println!("{}", "Dialectic:".bold().underline());
    if report.dialectic.illusions.is_empty()
        && !report.verdict.has_contradictions
        && !report.verdict.has_paralogisms
    {
        println!("  {} No dialectical issues detected.", "✓".green());
    } else {
        if report.verdict.has_illusions {
            println!(
                "  {} {} transcendental illusion(s)",
                "⚠".yellow(),
                report.dialectic.illusions.len()
            );
        }
        if report.verdict.has_contradictions {
            println!("  {} Antinomial contradiction(s) detected", "✗".red());
        }
        if report.verdict.has_paralogisms {
            println!("  {} Paralogism(s) detected", "⚠".yellow());
        }
    }

    println!();
    println!("{}", "Claim Reasoning:".bold().underline());
    let contradiction_ready = report
        .claim_analysis
        .claims
        .iter()
        .filter(|claim| annotation_to_triple(claim).supports_contradiction())
        .count();
    println!(
        "  Claims: {}  |  Contradiction-ready: {}  |  Supported: {}  |  Contradicted: {}  |  Novel: {}  |  Unresolved: {}  |  Missing context: {}",
        report.claim_analysis.claims.len(),
        contradiction_ready,
        report.claim_analysis.supported_count.to_string().green(),
        report.claim_analysis.contradicted_count.to_string().red(),
        report.claim_analysis.novel_count.to_string().yellow(),
        report.claim_analysis.unresolved_count,
        report.claim_analysis.missing_context_count,
    );

    if let Some(dialogue) = &report.dialogue_analysis {
        println!();
        println!("{}", "Dialogue Reasoning:".bold().underline());
        println!(
            "  Turns: {}  |  Dialogue ECS: {}  |  Flux: {:.2}  |  Contradictions: {}",
            dialogue.summary.turn_count,
            format_ecs(dialogue.last_turn.dialogue_ecs),
            dialogue.summary.epistemic_flux,
            dialogue.summary.contradiction_count,
        );
        for pair in dialogue.last_turn.contradiction_pairs.iter().take(3) {
            println!(
                "  {} turn {} contradicts turn {}: {} ↔ {}",
                "↯".red(),
                pair.turn_id,
                pair.established_at_turn,
                pair.incoming,
                pair.committed
            );
        }
    }

    // ECS flags
    if !cal.flags.is_empty() {
        println!();
        println!("{}", "ECS Flags:".bold().underline());
        for flag in &cal.flags {
            println!("  {} {}", "→".red(), flag);
        }
    }

    println!();
    println!("{}", "Summary:".bold().underline());
    println!("  {}", report.summary);
    println!("{}", "━".repeat(60).dimmed());
}

fn format_risk(risk: &pure_reason_core::pipeline::RiskLevel) -> colored::ColoredString {
    use pure_reason_core::pipeline::RiskLevel;
    match risk {
        RiskLevel::Safe => "SAFE".green().bold(),
        RiskLevel::Low => "LOW".yellow(),
        RiskLevel::Medium => "MEDIUM".yellow().bold(),
        RiskLevel::High => "HIGH".red().bold(),
        _ => "RISK".white(),
    }
}

pub fn format_ecs(ecs: u8) -> colored::ColoredString {
    let label = format!("{:3}/100", ecs);
    match ecs {
        80..=100 => label.green().bold(),
        40..=79 => label.yellow().bold(),
        _ => label.red().bold(),
    }
}

/// Read text from argument or stdin.
pub fn get_text(text_arg: &Option<String>) -> Result<String> {
    if let Some(t) = text_arg {
        return Ok(t.clone());
    }
    use std::io::Read;
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;
    Ok(buf.trim().to_string())
}
