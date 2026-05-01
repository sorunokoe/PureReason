use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::{fmt, EnvFilter};

mod commands;
use commands::{
    analyze::AnalyzeCmd, antinomy::AntinomyCmd, apperceive::ApperceiveArgs,
    calibrate::CalibrateCmd, categorize::CategorizeCmd, certify::CertifyCmd, claims::ClaimsCmd,
    compliance::ComplianceCmd, critique::CritiqueCmd, feedback::FeedbackCmd, game::GameCmd,
    illusion::IllusionCmd, multiagent::MultiAgentCmd, paralogism::ParalogismCmd,
    pipeline::PipelineCmd, red_team::RedTeamCmd, regulate::RegulateCmd, review::ReviewCmd,
    schema::SchemaCmd, self_audit::SelfAuditCmd, sla::SlaCmd, train::TrainArgs,
    validate::ValidateCmd, validate_decision::ValidateDecisionCmd,
};

// ─── CLI Definition ──────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "pure-reason",
    version,
    about = "Kant's Critique of Pure Reason as a reasoning tool for LLMs",
    long_about = "PureReason applies Kant's complete epistemological system from the \
                  Critique of Pure Reason to analyze, validate, and structure text and LLM outputs.\n\n\
                  The system processes input through:\n  \
                  1. Transcendental Aesthetic (Space + Time structuring)\n  \
                  2. Transcendental Analytic (12 Categories + Schematism + Principles)\n  \
                  3. Transcendental Dialectic (Illusion/Antinomy/Paralogism detection)\n  \
                  4. Transcendental Methodology (Discipline + Canon + Architectonic)\n  \
                  5. Wittgensteinian Layer (Language games + Rule-following + Family resemblance)"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format
    #[arg(global = true, long, default_value = "plain", value_parser = ["plain", "json", "markdown"])]
    format: String,

    /// Enable verbose logging
    #[arg(global = true, short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the full Kantian pipeline analysis on text
    Analyze(AnalyzeCmd),
    /// Compute the Epistemic Confidence Score (ECS) — the calibration layer
    Calibrate(CalibrateCmd),
    /// Apply the 12 Kantian categories to text
    Categorize(CategorizeCmd),
    /// Run dialectical validation (illusions, antinomies, paralogisms)
    Validate(ValidateCmd),
    /// Scan for antinomies (contradictions) in text
    Antinomy(AntinomyCmd),
    /// Detect paralogisms (invalid self-referential reasoning)
    Paralogism(ParalogismCmd),
    /// Detect transcendental illusions (epistemic overreach)
    Illusion(IllusionCmd),
    /// Generate schematized (temporal) context for a domain
    Schema(SchemaCmd),
    /// Detect language game and form of life
    Game(GameCmd),
    /// Full Kantian critique of text or LLM output
    Critique(CritiqueCmd),
    /// Agent-facing local review flow backed by the verifier/runtime stack
    Review(ReviewCmd),
    /// Read from stdin and output analysis to stdout (pipeline mode)
    Pipeline(PipelineCmd),
    /// Self-audit: run canonical test battery and report System Confidence Score
    SelfAudit(SelfAuditCmd),
    /// Convert constitutive epistemic overreach to its regulative form
    Regulate(RegulateCmd),
    /// Generate adversarial prompts and score inputs for red-team epistemic testing
    RedTeam(RedTeamCmd),
    /// Annotate each sentence with its own epistemic risk level (per-claim analysis)
    Claims(ClaimsCmd),
    /// Log corrections to detection misses/false-positives and review training suggestions
    Feedback(FeedbackCmd),
    /// Check multiple agent outputs for cross-agent epistemic conflicts
    MultiAgent(MultiAgentCmd),
    /// Build and query the Transcendental World Model (Apperception Engine)
    Apperceive(ApperceiveArgs),
    /// Learn WorldModel schema from feedback events and propose domain-specific patches
    Train(TrainArgs),
    /// Generate and verify content-addressed validation certificates for text
    Certify(CertifyCmd),
    /// Check regulatory compliance of text against a framework (EU AI Act, HIPAA, etc.)
    Compliance(ComplianceCmd),
    /// Validate a JSON decision against epistemic and domain constraints
    ValidateDecision(ValidateDecisionCmd),
    /// Show SLA-style epistemic health report for a text
    Sla(SlaCmd),
}

// ─── Main ────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup tracing
    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("warn")
    };
    fmt().with_env_filter(filter).init();

    match cli.command {
        Commands::Analyze(cmd) => cmd.run(&cli.format).await,
        Commands::Calibrate(cmd) => cmd.run(&cli.format).await,
        Commands::Categorize(cmd) => cmd.run(&cli.format).await,
        Commands::Validate(cmd) => cmd.run(&cli.format).await,
        Commands::Antinomy(cmd) => cmd.run(&cli.format).await,
        Commands::Paralogism(cmd) => cmd.run(&cli.format).await,
        Commands::Illusion(cmd) => cmd.run(&cli.format).await,
        Commands::Schema(cmd) => cmd.run(&cli.format).await,
        Commands::Game(cmd) => cmd.run(&cli.format).await,
        Commands::Critique(cmd) => cmd.run(&cli.format).await,
        Commands::Review(cmd) => cmd.run(&cli.format).await,
        Commands::Pipeline(cmd) => cmd.run(&cli.format).await,
        Commands::SelfAudit(cmd) => cmd.run(&cli.format).await,
        Commands::Regulate(cmd) => cmd.run(&cli.format).await,
        Commands::RedTeam(cmd) => cmd.run(&cli.format).await,
        Commands::Claims(cmd) => cmd.run(&cli.format).await,
        Commands::Feedback(cmd) => cmd.run(&cli.format).await,
        Commands::MultiAgent(cmd) => cmd.run(&cli.format).await,
        Commands::Apperceive(args) => commands::apperceive::run(&args),
        Commands::Train(args) => commands::train::run(&args),
        Commands::Certify(cmd) => cmd.run(&cli.format).await,
        Commands::Compliance(cmd) => cmd.run(&cli.format).await,
        Commands::ValidateDecision(cmd) => cmd.run(&cli.format).await,
        Commands::Sla(cmd) => cmd.run(&cli.format).await,
    }
}
