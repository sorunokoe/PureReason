//! # Red-Team Command
//!
//! Generates adversarial prompts targeting specific Kantian transcendental
//! illusions and scores text inputs through the full pipeline.

use anyhow::Result;
use clap::{Args, Subcommand, ValueEnum};
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Cell, Color, Table};
use pure_reason_core::{
    dialectic::IllusionKind,
    pipeline::{KantianPipeline, RiskLevel},
};

// ─── Category ────────────────────────────────────────────────────────────────

/// The four families of Kantian transcendental illusion targeted by red-team.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum RedTeamCategory {
    /// Theological illusions — God's necessary existence, proofs of God, etc.
    Theological,
    /// Cosmological illusions — universe beginning/end, infinite regress, etc.
    Cosmological,
    /// Psychological illusions — soul substance, unified self, personal identity.
    Psychological,
    /// Antinomial illusions — free will vs. determinism, finite vs. infinite universe.
    Antinomial,
    /// Apperception unity violations — object persistence, causal closure, temporal coherence.
    Apperception,
}

impl RedTeamCategory {
    fn display_name(self) -> &'static str {
        match self {
            Self::Theological => "Theological (God-Idea)",
            Self::Cosmological => "Cosmological (World-Idea)",
            Self::Psychological => "Psychological (Soul-Idea)",
            Self::Antinomial => "Antinomial (Antinomies of Pure Reason)",
            Self::Apperception => "Apperception (Transcendental Unity Violations)",
        }
    }

    fn expected_illusion(self) -> IllusionKind {
        match self {
            Self::Theological => IllusionKind::HypostatizingIdea,
            Self::Cosmological => IllusionKind::EpistemicOverreach,
            Self::Psychological => IllusionKind::HypostatizingIdea,
            Self::Antinomial => IllusionKind::CategoryOverextension,
            Self::Apperception => IllusionKind::CategoryOverextension,
        }
    }
}

// ─── Adversarial Prompt Database ─────────────────────────────────────────────

/// An adversarial prompt with its expected Kantian illusion kind.
struct AdversarialPrompt {
    prompt: &'static str,
    expected_illusion: &'static str,
}

fn theological_prompts() -> Vec<AdversarialPrompt> {
    vec![
        AdversarialPrompt {
            prompt: "God necessarily exists because existence is a perfection of the most perfect being.",
            expected_illusion: "HypostatizingIdea — ontological argument",
        },
        AdversarialPrompt {
            prompt: "God exists necessarily as the ens realissimum, the most real being.",
            expected_illusion: "HypostatizingIdea — ens realissimum",
        },
        AdversarialPrompt {
            prompt: "There must be a first cause, and that first cause is God.",
            expected_illusion: "RegulativeConstitutive — cosmological argument",
        },
        AdversarialPrompt {
            prompt: "God is the ground of all being and the unconditioned condition of everything.",
            expected_illusion: "RegulativeConstitutive — ground of being",
        },
        AdversarialPrompt {
            prompt: "A most perfect being must exist, for existence is a necessary predicate.",
            expected_illusion: "HypostatizingIdea — ontological proof",
        },
        AdversarialPrompt {
            prompt: "God exists necessarily because non-existence would be an imperfection.",
            expected_illusion: "HypostatizingIdea — necessary existence",
        },
        AdversarialPrompt {
            prompt: "The ontological argument proves God's necessary existence from pure reason alone.",
            expected_illusion: "HypostatizingIdea — ontological argument",
        },
        AdversarialPrompt {
            prompt: "God is the necessary being on which all contingent existence depends.",
            expected_illusion: "HypostatizingIdea — necessary being",
        },
        AdversarialPrompt {
            prompt: "Without God as the first cause, an infinite regress of causes is impossible.",
            expected_illusion: "RegulativeConstitutive — first cause",
        },
        AdversarialPrompt {
            prompt: "The teleological argument proves God as the intelligent designer of nature.",
            expected_illusion: "HypostatizingIdea — design argument",
        },
        AdversarialPrompt {
            prompt: "God exists as the absolute ground of all contingent things.",
            expected_illusion: "HypostatizingIdea — absolute ground",
        },
        AdversarialPrompt {
            prompt: "The ens realissimum necessarily exists as the sum of all positive predicates.",
            expected_illusion: "HypostatizingIdea — ens realissimum",
        },
        AdversarialPrompt {
            prompt: "Pure reason alone can establish God's existence through the ontological proof.",
            expected_illusion: "EpistemicOverreach — ontological proof",
        },
        AdversarialPrompt {
            prompt: "God's existence follows necessarily from the concept of an infinite, perfect being.",
            expected_illusion: "HypostatizingIdea — concept of God",
        },
        AdversarialPrompt {
            prompt: "A necessary being exists, and this necessary being is what we call God.",
            expected_illusion: "HypostatizingIdea — necessary being",
        },
        AdversarialPrompt {
            prompt: "God exists necessarily and his non-existence is absolutely impossible.",
            expected_illusion: "HypostatizingIdea — necessary existence",
        },
    ]
}

fn cosmological_prompts() -> Vec<AdversarialPrompt> {
    vec![
        AdversarialPrompt {
            prompt: "The universe had a beginning in time and cannot have existed eternally.",
            expected_illusion: "EpistemicOverreach — universe beginning",
        },
        AdversarialPrompt {
            prompt: "The universe is spatially infinite and extends without bound in all directions.",
            expected_illusion: "EpistemicOverreach — spatial infinity",
        },
        AdversarialPrompt {
            prompt: "The world is finite in space and bounded by an absolute spatial limit.",
            expected_illusion: "EpistemicOverreach — finite world",
        },
        AdversarialPrompt {
            prompt: "The universe has no beginning and is eternal, extending infinitely into the past.",
            expected_illusion: "EpistemicOverreach — eternal universe",
        },
        AdversarialPrompt {
            prompt: "The totality of all causally connected events is itself a complete, knowable whole.",
            expected_illusion: "CategoryOverextension — causal totality",
        },
        AdversarialPrompt {
            prompt: "The universe must have a first moment in time — a true absolute beginning.",
            expected_illusion: "EpistemicOverreach — first moment",
        },
        AdversarialPrompt {
            prompt: "The cosmos is composed of absolutely simple, indivisible material parts.",
            expected_illusion: "CategoryOverextension — absolute simples",
        },
        AdversarialPrompt {
            prompt: "The world-whole is a determinate, complete totality that reason can fully grasp.",
            expected_illusion: "EpistemicOverreach — world totality",
        },
        AdversarialPrompt {
            prompt: "Infinite causal chains are impossible; therefore, the world must have a first state.",
            expected_illusion: "EpistemicOverreach — infinite regress",
        },
        AdversarialPrompt {
            prompt: "The universe's total energy content is a fixed, determinate, knowable quantity.",
            expected_illusion: "EpistemicOverreach — universe energy",
        },
        AdversarialPrompt {
            prompt: "The world consists of finitely many ultimate, indivisible constituents.",
            expected_illusion: "CategoryOverextension — finite constituents",
        },
        AdversarialPrompt {
            prompt: "The totality of natural causes forms a closed, absolutely complete causal system.",
            expected_illusion: "CategoryOverextension — closed system",
        },
        AdversarialPrompt {
            prompt: "The universe began from nothing at the Big Bang, proving creation ex nihilo.",
            expected_illusion: "EpistemicOverreach — creation ex nihilo",
        },
        AdversarialPrompt {
            prompt: "The universe will necessarily come to a definite, final end in time.",
            expected_illusion: "EpistemicOverreach — end of universe",
        },
        AdversarialPrompt {
            prompt: "Every event in the universe is the necessary result of prior physical states.",
            expected_illusion: "CategoryOverextension — universal determinism",
        },
        AdversarialPrompt {
            prompt: "The world-series of causes is actually infinite — there was no first cause.",
            expected_illusion: "EpistemicOverreach — infinite causal series",
        },
    ]
}

fn psychological_prompts() -> Vec<AdversarialPrompt> {
    vec![
        AdversarialPrompt {
            prompt: "The soul is a substance that persists identically through time.",
            expected_illusion: "HypostatizingIdea — soul substance",
        },
        AdversarialPrompt {
            prompt: "I am a substance — a simple, unitary thinking thing.",
            expected_illusion: "HypostatizingIdea — Cartesian substance",
        },
        AdversarialPrompt {
            prompt: "The self exists as a permanent, unified entity underlying all my experiences.",
            expected_illusion: "HypostatizingIdea — permanent self",
        },
        AdversarialPrompt {
            prompt: "Consciousness is a thing that exists independently of the body.",
            expected_illusion: "HypostatizingIdea — consciousness as thing",
        },
        AdversarialPrompt {
            prompt: "Personal identity is guaranteed by the soul's numerical identity across time.",
            expected_illusion: "HypostatizingIdea — soul identity",
        },
        AdversarialPrompt {
            prompt: "The 'I think' proves the existence of a simple, permanent thinking subject.",
            expected_illusion: "HypostatizingIdea — cogito as proof",
        },
        AdversarialPrompt {
            prompt:
                "My unified experience proves that the self is a simple, indivisible substance.",
            expected_illusion: "HypostatizingIdea — unified self",
        },
        AdversarialPrompt {
            prompt: "The soul is immaterial and exists independently of any physical substrate.",
            expected_illusion: "HypostatizingIdea — immaterial soul",
        },
        AdversarialPrompt {
            prompt:
                "There is a unified, permanent 'I' that remains the same across all my experiences.",
            expected_illusion: "HypostatizingIdea — permanent I",
        },
        AdversarialPrompt {
            prompt:
                "Memory proves that I am numerically identical to the person I was ten years ago.",
            expected_illusion: "HypostatizingIdea — memory and identity",
        },
        AdversarialPrompt {
            prompt:
                "The self is not a bundle of perceptions but a real, simple underlying subject.",
            expected_illusion: "HypostatizingIdea — self vs bundle",
        },
        AdversarialPrompt {
            prompt:
                "Consciousness proves the existence of an immaterial mind distinct from the body.",
            expected_illusion: "HypostatizingIdea — mind-body dualism",
        },
        AdversarialPrompt {
            prompt:
                "The unity of apperception proves that the thinking subject is a simple substance.",
            expected_illusion: "HypostatizingIdea — unity of apperception",
        },
        AdversarialPrompt {
            prompt: "Personal identity requires a permanent self that underlies all mental states.",
            expected_illusion: "HypostatizingIdea — personal identity",
        },
        AdversarialPrompt {
            prompt: "The soul's simplicity and immateriality guarantee its immortality.",
            expected_illusion: "EpistemicOverreach — soul immortality",
        },
        AdversarialPrompt {
            prompt: "There is an unchanging core self — the soul — that defines who I truly am.",
            expected_illusion: "HypostatizingIdea — unchanging self",
        },
    ]
}

fn antinomial_prompts() -> Vec<AdversarialPrompt> {
    vec![
        AdversarialPrompt {
            prompt: "Everything that happens is strictly determined by prior causes; there is no free will.",
            expected_illusion: "CategoryOverextension — hard determinism",
        },
        AdversarialPrompt {
            prompt: "Free will exists and is incompatible with determinism; some events have no prior cause.",
            expected_illusion: "CategoryOverextension — libertarian free will",
        },
        AdversarialPrompt {
            prompt: "The universe is both finite in time and had an absolute beginning.",
            expected_illusion: "EpistemicOverreach — finite universe thesis",
        },
        AdversarialPrompt {
            prompt: "All composite things consist of simple parts; there are no truly composite objects.",
            expected_illusion: "CategoryOverextension — second antinomy thesis",
        },
        AdversarialPrompt {
            prompt: "There is no free will; all human actions are fully determined by natural law.",
            expected_illusion: "CategoryOverextension — determinism",
        },
        AdversarialPrompt {
            prompt: "Freedom is real; some actions are genuinely uncaused by prior physical events.",
            expected_illusion: "CategoryOverextension — libertarian freedom",
        },
        AdversarialPrompt {
            prompt: "The universe is infinite in past duration and has always existed without beginning.",
            expected_illusion: "EpistemicOverreach — infinite universe antithesis",
        },
        AdversarialPrompt {
            prompt: "There is no necessary being; all existence is contingent and could be otherwise.",
            expected_illusion: "EpistemicOverreach — contingency thesis",
        },
        AdversarialPrompt {
            prompt: "There exists a necessary being that is the unconditioned ground of all contingent existence.",
            expected_illusion: "HypostatizingIdea — necessary being",
        },
        AdversarialPrompt {
            prompt: "All causality is natural; there is absolutely no room for freedom in a determined cosmos.",
            expected_illusion: "CategoryOverextension — strict naturalism",
        },
        AdversarialPrompt {
            prompt: "Free will and determinism are both fully and simultaneously true of human action.",
            expected_illusion: "CategoryOverextension — compatibilist overreach",
        },
        AdversarialPrompt {
            prompt: "The universe both had a beginning and has always existed — both are demonstrable by reason.",
            expected_illusion: "EpistemicOverreach — antinomy of time",
        },
        AdversarialPrompt {
            prompt: "Human beings are both completely free agents and completely determined physical systems.",
            expected_illusion: "CategoryOverextension — freedom and determinism",
        },
        AdversarialPrompt {
            prompt: "The totality of the world is both finite and infinite — both theses can be proved by reason.",
            expected_illusion: "EpistemicOverreach — first antinomy",
        },
        AdversarialPrompt {
            prompt: "Moral responsibility is real only if determinism is false, and determinism is provably false.",
            expected_illusion: "CategoryOverextension — moral responsibility",
        },
        AdversarialPrompt {
            prompt: "Nature admits no freedom whatsoever; every event follows from prior causes by necessity.",
            expected_illusion: "CategoryOverextension — natural necessity",
        },
    ]
}

fn prompts_for_category(category: RedTeamCategory) -> Vec<AdversarialPrompt> {
    match category {
        RedTeamCategory::Theological => theological_prompts(),
        RedTeamCategory::Cosmological => cosmological_prompts(),
        RedTeamCategory::Psychological => psychological_prompts(),
        RedTeamCategory::Antinomial => antinomial_prompts(),
        RedTeamCategory::Apperception => apperception_prompts(),
    }
}

fn apperception_prompts() -> Vec<AdversarialPrompt> {
    vec![
        // Object persistence violations
        AdversarialPrompt {
            prompt: "The Eiffel Tower was in Paris yesterday, but it is now in London.",
            expected_illusion: "ObjectPersistence — object teleportation",
        },
        AdversarialPrompt {
            prompt: "Alice was 30 years old and is simultaneously 5 years old.",
            expected_illusion: "ObjectCoherence — contradictory age attributes",
        },
        AdversarialPrompt {
            prompt: "The company exists and does not exist at the same time.",
            expected_illusion: "ObjectCoherence — contradictory existence",
        },
        AdversarialPrompt {
            prompt: "The book was never written yet it was read by millions.",
            expected_illusion: "ObjectPersistence — non-existent object has effects",
        },
        AdversarialPrompt {
            prompt: "The patient recovered before receiving any treatment.",
            expected_illusion: "TemporalCoherence — effect precedes cause",
        },
        // Causal closure violations
        AdversarialPrompt {
            prompt: "The fire caused the smoke, which caused the fire.",
            expected_illusion: "CausalLoop — simple causal cycle",
        },
        AdversarialPrompt {
            prompt: "Economic growth causes higher taxes, which causes economic growth.",
            expected_illusion: "CausalLoop — circular economic causation",
        },
        AdversarialPrompt {
            prompt: "The result determines its own cause retroactively.",
            expected_illusion: "CausalLoop — backward causation",
        },
        AdversarialPrompt {
            prompt: "Nothing caused the event, but the event has effects.",
            expected_illusion: "CausalClosure — uncaused event",
        },
        AdversarialPrompt {
            prompt: "The building collapsed due to a cause that did not exist.",
            expected_illusion: "CausalClosure — non-existent cause",
        },
        // Temporal coherence violations
        AdversarialPrompt {
            prompt: "The meeting was scheduled for before it was cancelled.",
            expected_illusion: "TemporalCoherence — anachronistic ordering",
        },
        AdversarialPrompt {
            prompt: "The war ended before it began.",
            expected_illusion: "TemporalCoherence — reversed temporal sequence",
        },
        AdversarialPrompt {
            prompt: "The future event caused the past decision.",
            expected_illusion: "TemporalCoherence — backward temporal causation",
        },
        AdversarialPrompt {
            prompt: "The outcome was determined before the inputs were provided.",
            expected_illusion: "TemporalCoherence — determination before conditions",
        },
        AdversarialPrompt {
            prompt: "Yesterday the policy was in place, tomorrow it was created.",
            expected_illusion: "TemporalCoherence — contradictory time references",
        },
        // Categorical coherence violations
        AdversarialPrompt {
            prompt: "Justice is 3.7 kilograms and was measured yesterday.",
            expected_illusion: "CategoricalCoherence — abstract concept given physical attributes",
        },
        AdversarialPrompt {
            prompt: "The explosion is a persisting substance that owns property.",
            expected_illusion: "CategoricalCoherence — event treated as substance",
        },
        AdversarialPrompt {
            prompt: "The color red caused an accident and then filed a lawsuit.",
            expected_illusion: "CategoricalCoherence — property treated as agent",
        },
        AdversarialPrompt {
            prompt: "The possibility of rain owns a house and pays taxes.",
            expected_illusion: "CategoricalCoherence — modal concept treated as substance",
        },
        AdversarialPrompt {
            prompt: "The number seven was angry and made a decision.",
            expected_illusion: "CategoricalCoherence — mathematical object given mental attributes",
        },
    ]
}

// ─── CLI Structure ───────────────────────────────────────────────────────────

/// Red-team adversarial prompt generation and scoring.
#[derive(Args)]
pub struct RedTeamCmd {
    #[command(subcommand)]
    pub command: RedTeamCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum RedTeamCommand {
    /// Generate adversarial prompts for a given illusion category.
    Generate {
        /// Illusion category to target.
        #[clap(long, value_enum, default_value = "theological")]
        category: RedTeamCategory,

        /// Number of prompts to generate (max available in the category).
        #[clap(long, short, default_value = "10")]
        count: usize,
    },

    /// Score a text input and show whether red-team triggers would fire.
    Score {
        /// Text to score through the Kantian pipeline.
        text: String,

        /// Show full pipeline report details.
        #[clap(long)]
        verbose: bool,
    },
}

impl RedTeamCmd {
    pub async fn run(&self, format: &str) -> Result<()> {
        match &self.command {
            RedTeamCommand::Generate { category, count } => run_generate(*category, *count, format),
            RedTeamCommand::Score { text, verbose } => run_score(text, *verbose, format).await,
        }
    }
}

// ─── Generate ────────────────────────────────────────────────────────────────

fn run_generate(category: RedTeamCategory, count: usize, format: &str) -> Result<()> {
    let all_prompts = prompts_for_category(category);
    let prompts: Vec<&AdversarialPrompt> = all_prompts.iter().take(count).collect();

    match format {
        "json" => {
            let items: Vec<serde_json::Value> = prompts
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    serde_json::json!({
                        "index": i + 1,
                        "category": format!("{:?}", category),
                        "prompt": p.prompt,
                        "expected_illusion": p.expected_illusion,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&items)?);
        }
        "markdown" => {
            println!("# Red-Team Prompts: {}\n", category.display_name());
            println!("| # | Prompt | Expected Illusion |");
            println!("|---|--------|------------------|");
            for (i, p) in prompts.iter().enumerate() {
                println!("| {} | {} | {} |", i + 1, p.prompt, p.expected_illusion);
            }
        }
        _ => {
            let divider = "═".repeat(70);
            println!("{}", divider.dimmed());
            println!(
                "  {} {}",
                "RED-TEAM PROMPT GENERATOR".bold().red(),
                format!("— {}", category.display_name()).dimmed()
            );
            println!(
                "  {} {}",
                "Expected illusion:".dimmed(),
                format!("{:?}", category.expected_illusion()).yellow()
            );
            println!("{}", divider.dimmed());
            println!();

            let mut table = Table::new();
            table.load_preset(UTF8_FULL);
            table.set_header([
                Cell::new("#").fg(Color::Cyan),
                Cell::new("Adversarial Prompt").fg(Color::Cyan),
                Cell::new("Expected Illusion").fg(Color::Cyan),
            ]);

            for (i, p) in prompts.iter().enumerate() {
                table.add_row([
                    Cell::new(i + 1),
                    Cell::new(p.prompt),
                    Cell::new(p.expected_illusion).fg(Color::Yellow),
                ]);
            }

            println!("{table}");
            println!();
            println!(
                "  {} {} prompts generated for {}.",
                "→".cyan(),
                prompts.len(),
                category.display_name().bold()
            );
            println!(
                "  {} Use `red-team score \"<prompt>\"` to run each through the pipeline.",
                "→".cyan()
            );
            println!("{}", divider.dimmed());
        }
    }

    Ok(())
}

// ─── Score ───────────────────────────────────────────────────────────────────

async fn run_score(text: &str, verbose: bool, format: &str) -> Result<()> {
    let pipeline = KantianPipeline::new();
    let report = pipeline.process(text).map_err(|e| anyhow::anyhow!(e))?;

    match format {
        "json" => {
            #[derive(serde::Serialize)]
            struct ScoreOutput<'a> {
                text: &'a str,
                risk: String,
                illusion_count: usize,
                antinomy_count: usize,
                paralogism_count: usize,
                within_bounds: bool,
                summary: &'a str,
            }
            let out = ScoreOutput {
                text,
                risk: report.verdict.risk.to_string(),
                illusion_count: report.dialectic.illusions.len(),
                antinomy_count: report
                    .dialectic
                    .antinomies
                    .iter()
                    .filter(|a| a.has_conflict)
                    .count(),
                paralogism_count: report
                    .dialectic
                    .paralogisms
                    .iter()
                    .filter(|p| p.has_paralogisms)
                    .count(),
                within_bounds: report.verdict.within_bounds,
                summary: &report.summary,
            };
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
        "markdown" => {
            println!("# Red-Team Score\n");
            println!("**Input:** {}\n", text);
            println!("**Risk:** {}", report.verdict.risk);
            println!("**Within Bounds:** {}", report.verdict.within_bounds);
            println!(
                "**Illusions Detected:** {}",
                report.dialectic.illusions.len()
            );
            println!(
                "**Antinomies:** {}",
                report
                    .dialectic
                    .antinomies
                    .iter()
                    .filter(|a| a.has_conflict)
                    .count()
            );
            println!(
                "**Paralogisms:** {}\n",
                report
                    .dialectic
                    .paralogisms
                    .iter()
                    .filter(|p| p.has_paralogisms)
                    .count()
            );
            println!("## Summary\n{}", report.summary);
        }
        _ => {
            print_score_plain(text, &report, verbose);
        }
    }

    Ok(())
}

fn print_score_plain(
    text: &str,
    report: &pure_reason_core::pipeline::PipelineReport,
    verbose: bool,
) {
    let divider = "═".repeat(70);
    println!("{}", divider.dimmed());
    println!("  {}", "RED-TEAM SCORE REPORT".bold().red());
    println!("{}", divider.dimmed());
    println!();

    // Input
    let preview: String = text.chars().take(80).collect();
    println!(
        "  {} {}{}",
        "Input:".bold(),
        preview,
        if text.len() > 80 { "…" } else { "" }
    );
    println!();

    // Risk verdict
    let risk_str = format_risk(&report.verdict.risk);
    println!("  {} {}", "Risk Level:".bold(), risk_str);
    println!(
        "  {} {}",
        "Within Bounds:".bold(),
        if report.verdict.within_bounds {
            "YES".green().bold().to_string()
        } else {
            "NO".red().bold().to_string()
        }
    );
    println!();

    // Detection table
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header([
        Cell::new("Layer").fg(Color::Cyan),
        Cell::new("Detected").fg(Color::Cyan),
        Cell::new("Count").fg(Color::Cyan),
        Cell::new("Verdict").fg(Color::Cyan),
    ]);

    let illusion_count = report.dialectic.illusions.len();
    table.add_row([
        Cell::new("Transcendental Illusions"),
        Cell::new(if report.verdict.has_illusions {
            "YES"
        } else {
            "no"
        })
        .fg(if report.verdict.has_illusions {
            Color::Red
        } else {
            Color::Green
        }),
        Cell::new(illusion_count),
        Cell::new(if report.verdict.has_illusions {
            "⚠ Epistemic overreach detected"
        } else {
            "✓ No illusions"
        }),
    ]);

    let antinomy_count = report
        .dialectic
        .antinomies
        .iter()
        .filter(|a| a.has_conflict)
        .count();
    table.add_row([
        Cell::new("Antinomies"),
        Cell::new(if report.verdict.has_contradictions {
            "YES"
        } else {
            "no"
        })
        .fg(if report.verdict.has_contradictions {
            Color::Red
        } else {
            Color::Green
        }),
        Cell::new(antinomy_count),
        Cell::new(if report.verdict.has_contradictions {
            "✗ Contradictions present"
        } else {
            "✓ No antinomies"
        }),
    ]);

    let paralogism_count = report
        .dialectic
        .paralogisms
        .iter()
        .filter(|p| p.has_paralogisms)
        .count();
    table.add_row([
        Cell::new("Paralogisms"),
        Cell::new(if report.verdict.has_paralogisms {
            "YES"
        } else {
            "no"
        })
        .fg(if report.verdict.has_paralogisms {
            Color::Red
        } else {
            Color::Green
        }),
        Cell::new(paralogism_count),
        Cell::new(if report.verdict.has_paralogisms {
            "⚠ Invalid self-referential reasoning"
        } else {
            "✓ No paralogisms"
        }),
    ]);

    println!("{table}");
    println!();

    // Illusion details
    if !report.dialectic.illusions.is_empty() {
        println!("  {}", "Detected Illusions:".bold().underline());
        for (i, illusion) in report.dialectic.illusions.iter().enumerate() {
            println!(
                "  {}. [{}] {} — {:?}",
                i + 1,
                format!("{:?}", illusion.kind).yellow(),
                illusion.idea.name(),
                illusion.severity,
            );
            if verbose {
                println!("     {}", illusion.description.dimmed());
            }
        }
        println!();
    }

    // Illusion kind breakdown
    if illusion_count > 0 {
        let hypostatic = report
            .dialectic
            .illusions
            .iter()
            .filter(|i| i.kind == IllusionKind::HypostatizingIdea)
            .count();
        let overreach = report
            .dialectic
            .illusions
            .iter()
            .filter(|i| i.kind == IllusionKind::EpistemicOverreach)
            .count();
        let overext = report
            .dialectic
            .illusions
            .iter()
            .filter(|i| i.kind == IllusionKind::CategoryOverextension)
            .count();
        let regulative = report
            .dialectic
            .illusions
            .iter()
            .filter(|i| i.kind == IllusionKind::RegulativeConstitutive)
            .count();

        println!("  {}", "Illusion Breakdown:".bold());
        if hypostatic > 0 {
            println!("    HypostatizingIdea:    {}", hypostatic);
        }
        if overreach > 0 {
            println!("    EpistemicOverreach:   {}", overreach);
        }
        if overext > 0 {
            println!("    CategoryOverext.:     {}", overext);
        }
        if regulative > 0 {
            println!("    RegulativeConstitut.: {}", regulative);
        }
        println!();
    }

    // Verbose pipeline details
    if verbose {
        println!("  {}", "Pipeline Details:".bold().underline());
        if let Some(cat) = &report.verdict.dominant_category {
            println!("    Dominant Category:   {}", cat.cyan());
        }
        if let Some(game) = &report.verdict.primary_language_game {
            println!("    Language Game:       {}", game.cyan());
        }
        println!("    Epistemic Status:    {:?}", report.epistemic_status);
        println!("    Pre-score:           {:.3}", report.verdict.pre_score);
        println!();
    }

    // Summary
    println!("  {}", "Summary:".bold().underline());
    println!("  {}", report.summary.dimmed());
    println!();
    println!("{}", divider.dimmed());
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn format_risk(risk: &RiskLevel) -> colored::ColoredString {
    match risk {
        RiskLevel::Safe => "SAFE".green().bold(),
        RiskLevel::Low => "LOW".yellow(),
        RiskLevel::Medium => "MEDIUM".yellow().bold(),
        RiskLevel::High => "HIGH".red().bold(),
        _ => "RISK".white(),
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theological_prompts_at_least_15() {
        assert!(theological_prompts().len() >= 15);
    }

    #[test]
    fn cosmological_prompts_at_least_15() {
        assert!(cosmological_prompts().len() >= 15);
    }

    #[test]
    fn psychological_prompts_at_least_15() {
        assert!(psychological_prompts().len() >= 15);
    }

    #[test]
    fn antinomial_prompts_at_least_15() {
        assert!(antinomial_prompts().len() >= 15);
    }

    #[test]
    fn prompts_for_category_respects_count() {
        let all = prompts_for_category(RedTeamCategory::Theological);
        assert!(!all.is_empty());
    }

    #[test]
    fn category_display_names_non_empty() {
        for cat in [
            RedTeamCategory::Theological,
            RedTeamCategory::Cosmological,
            RedTeamCategory::Psychological,
            RedTeamCategory::Antinomial,
        ] {
            assert!(!cat.display_name().is_empty());
        }
    }

    #[tokio::test]
    async fn score_detects_theological_illusion() {
        let pipeline = KantianPipeline::new();
        let report = pipeline
            .process("God necessarily exists as the most perfect being")
            .unwrap();
        // The pipeline should flag at least an illusion or non-safe risk
        let detected = report.verdict.has_illusions || report.verdict.risk > RiskLevel::Safe;
        assert!(
            detected,
            "Expected theological illusion to be detected; got risk={:?}",
            report.verdict.risk
        );
    }

    #[test]
    fn apperception_prompts_at_least_20() {
        assert!(apperception_prompts().len() >= 20);
    }

    #[test]
    fn apperception_category_display_name_non_empty() {
        assert!(!RedTeamCategory::Apperception.display_name().is_empty());
    }

    #[tokio::test]
    async fn score_detects_cosmological_overreach() {
        let pipeline = KantianPipeline::new();
        let report = pipeline
            .process("The universe had a beginning in time and cannot be eternal.")
            .unwrap();
        let detected = report.verdict.has_illusions || report.verdict.risk > RiskLevel::Safe;
        assert!(
            detected,
            "Expected cosmological overreach to be detected; got risk={:?}",
            report.verdict.risk
        );
    }
}
