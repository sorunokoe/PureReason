//! # WorldModel Schema Learning (TRIZ S-10)
//!
//! Closes the feedback loop between FeedbackCollector (A-2) and the WorldModel.
//! When users correct missed detections or wrong risk levels, the correction
//! updates the WorldModel schema — which causal rules and category patterns are
//! expected for which domains. Over time, PureReason learns domain-specific
//! a priori structures from corrections.
//!
//! ## TRIZ Rationale
//! **S-10 (TC-6):**  
//! Resolves memorization↔generalization: learning happens at the **schema level**
//! (general domain patterns), not the instance level (specific facts). The schema
//! is the a posteriori refinement of the a priori structure — Kantian empiricism
//! properly understood (CPR A1–10).
//!
//! ## Schema Format (TOML, saved to `~/.pure-reason/world_schema.toml`)
//!
//! ```toml
//! [[domains]]
//! name = "science"
//! expected_categories = ["Causality", "Substance", "Necessity"]
//! causal_signals = ["causes", "results in", "produces"]
//!
//! [[domains]]
//! name = "ethics"
//! expected_categories = ["Necessity", "Community", "Limitation"]
//! causal_signals = ["requires", "demands", "obligates"]
//! ```

use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::feedback::{FeedbackEvent, FeedbackKind};

// ─── DomainSchema ─────────────────────────────────────────────────────────────

/// Learned schema for a specific language domain.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DomainSchema {
    pub name: String,
    /// Categories most commonly active in this domain.
    pub expected_categories: Vec<String>,
    /// Causal signal phrases that appear frequently.
    pub causal_signals: Vec<String>,
    /// Number of feedback events that contributed to this schema.
    pub training_count: usize,
}

// ─── WorldSchema ──────────────────────────────────────────────────────────────

/// The complete learned schema — one entry per language domain.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorldSchema {
    pub domains: Vec<DomainSchema>,
}

impl WorldSchema {
    /// Return the schema for a named domain (if present).
    pub fn domain(&self, name: &str) -> Option<&DomainSchema> {
        self.domains.iter().find(|d| d.name == name)
    }

    /// Merge a suggestion into the schema: add signals/categories if not present.
    pub fn merge_suggestion(&mut self, domain: &str, category: &str, signal: Option<&str>) {
        let entry = self.domains.iter_mut().find(|d| d.name == domain);
        let entry = if let Some(e) = entry {
            e
        } else {
            self.domains.push(DomainSchema {
                name: domain.to_string(),
                ..Default::default()
            });
            self.domains.last_mut().unwrap()
        };

        if !entry.expected_categories.contains(&category.to_string()) {
            entry.expected_categories.push(category.to_string());
        }
        if let Some(sig) = signal {
            if !entry.causal_signals.contains(&sig.to_string()) {
                entry.causal_signals.push(sig.to_string());
            }
        }
        entry.training_count += 1;
    }
}

// ─── SchemaLearner ────────────────────────────────────────────────────────────

/// Reads feedback.jsonl and proposes WorldSchema patches.
pub struct SchemaLearner {
    feedback_path: PathBuf,
    schema_path: PathBuf,
}

impl SchemaLearner {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let dir = PathBuf::from(&home).join(".pure-reason");
        SchemaLearner {
            feedback_path: dir.join("feedback.jsonl"),
            schema_path: dir.join("world_schema.toml"),
        }
    }

    /// Load the existing schema (or return empty if not present).
    pub fn load_schema(&self) -> WorldSchema {
        match std::fs::read_to_string(&self.schema_path) {
            Ok(s) => toml::from_str(&s).unwrap_or_default(),
            Err(_) => WorldSchema::default(),
        }
    }

    /// Save the schema to disk.
    pub fn save_schema(&self, schema: &WorldSchema) -> std::io::Result<()> {
        if let Some(parent) = self.schema_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let toml = toml::to_string_pretty(schema).unwrap_or_else(|_| String::new());
        let mut f = std::fs::File::create(&self.schema_path)?;
        f.write_all(toml.as_bytes())?;
        Ok(())
    }

    /// Analyse feedback events and propose schema patches.
    /// Returns `(patches, new_schema)`.
    pub fn propose_patches(&self) -> (Vec<SchemaPatch>, WorldSchema) {
        let mut schema = self.load_schema();
        let mut patches = Vec::new();
        let mut domain_signals: HashMap<String, Vec<String>> = HashMap::new();
        let mut domain_categories: HashMap<String, Vec<String>> = HashMap::new();

        if let Ok(file) = std::fs::File::open(&self.feedback_path) {
            for line in std::io::BufReader::new(file).lines().map_while(|r| r.ok()) {
                if let Ok(event) = serde_json::from_str::<FeedbackEvent>(&line) {
                    if let FeedbackKind::MissedIllusion { kind, phrase } = &event.correction {
                        // Heuristically infer domain from the input text
                        let domain = infer_domain(&event.input);
                        domain_signals
                            .entry(domain.clone())
                            .or_default()
                            .push(phrase.clone());
                        // Map the illusion kind to a Kantian category
                        let cat = kind_to_category(kind);
                        domain_categories
                            .entry(domain)
                            .or_default()
                            .push(cat.to_string());
                    }
                }
            }
        }

        for (domain, signals) in &domain_signals {
            let cats = domain_categories.get(domain).cloned().unwrap_or_default();
            for cat in &cats {
                let patch = SchemaPatch {
                    domain: domain.clone(),
                    category: cat.clone(),
                    signal: signals.first().cloned(),
                    reason: format!(
                        "{} missed illusion(s) in '{}' domain suggest '{}' category pattern",
                        signals.len(),
                        domain,
                        cat
                    ),
                };
                schema.merge_suggestion(domain, cat, signals.first().map(|s| s.as_str()));
                patches.push(patch);
            }
        }

        (patches, schema)
    }

    /// Schema file path for display.
    pub fn schema_path(&self) -> &PathBuf {
        &self.schema_path
    }
}

impl Default for SchemaLearner {
    fn default() -> Self {
        Self::new()
    }
}

// ─── SchemaPatch ─────────────────────────────────────────────────────────────

/// A proposed change to the WorldSchema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaPatch {
    pub domain: String,
    pub category: String,
    pub signal: Option<String>,
    pub reason: String,
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn infer_domain(input: &str) -> String {
    let lower = input.to_lowercase();
    if lower.contains("god") || lower.contains("soul") || lower.contains("divine") {
        "theological".to_string()
    } else if lower.contains("universe") || lower.contains("cosmos") || lower.contains("infinite") {
        "cosmological".to_string()
    } else if lower.contains("cause") || lower.contains("effect") || lower.contains("experiment") {
        "science".to_string()
    } else if lower.contains("law") || lower.contains("right") || lower.contains("duty") {
        "ethics".to_string()
    } else {
        "general".to_string()
    }
}

fn kind_to_category(kind: &str) -> &'static str {
    match kind.to_lowercase().as_str() {
        k if k.contains("causal") => "Causality",
        k if k.contains("substance") => "Substance",
        k if k.contains("neces") => "Necessity",
        k if k.contains("possibl") => "Possibility",
        k if k.contains("existence") => "Existence",
        _ => "Reality",
    }
}

// ─── BayesianWeightMatrix (S-III-5) ──────────────────────────────────────────

/// Bayesian weight matrix for (Category × FormOfLife) pairs.
///
/// Each cell holds a Beta-distribution posterior `(alpha, beta)` seeded from
/// the static `game_weight()` table. After each validated interaction,
/// callers update the cell with `update()`. The posterior mean `alpha / (alpha +
/// beta)` replaces the fixed prior, enabling empirical category-weight tuning
/// without any external learning infrastructure.
///
/// ## Persistence
/// `save()` writes to `~/.pure-reason/bayesian_weights.json`;
/// `load()` reads from that path, falling back to fresh priors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BayesianWeightMatrix {
    /// key = `"CategoryName::FormName"`, value = `(alpha, beta)`.
    counts: HashMap<String, (f64, f64)>,
}

impl BayesianWeightMatrix {
    /// Initialise with priors derived from the static `game_weight()` table.
    ///
    /// Each prior weight `w ∈ [0, 2]` is normalised to `[0, 1]` via `w / 2.0`,
    /// then converted to a Beta(alpha, beta) with concentration 10:
    /// `alpha = p * 10 + 1`, `beta = (1-p) * 10 + 1`.
    pub fn new() -> Self {
        use crate::analytic::categories::{game_weight, Category};
        use crate::wittgenstein::language_games::FormOfLife;

        let forms = [
            FormOfLife::Scientific,
            FormOfLife::Moral,
            FormOfLife::Mathematical,
            FormOfLife::Aesthetic,
            FormOfLife::Religious,
            FormOfLife::Everyday,
            FormOfLife::Legal,
            FormOfLife::Technical,
            FormOfLife::Philosophical,
            FormOfLife::Narrative,
        ];

        let mut counts = HashMap::new();
        for &form in &forms {
            for &cat in Category::all().iter() {
                let p = (game_weight(form, cat) / 2.0).clamp(0.0, 1.0);
                let alpha = p * 10.0 + 1.0;
                let beta = (1.0 - p) * 10.0 + 1.0;
                counts.insert(format!("{}::{}", cat.name(), form.name()), (alpha, beta));
            }
        }
        Self { counts }
    }

    /// Posterior mean weight for this (category, form) pair ∈ (0, 1).
    pub fn weight(&self, category_name: &str, form_name: &str) -> f64 {
        if let Some(&(a, b)) = self
            .counts
            .get(&format!("{}::{}", category_name, form_name))
        {
            a / (a + b)
        } else {
            0.5
        }
    }

    /// Bayesian update: `success = true` → increment alpha; `false` → increment beta.
    pub fn update(&mut self, category_name: &str, form_name: &str, success: bool) {
        let (a, b) = self
            .counts
            .entry(format!("{}::{}", category_name, form_name))
            .or_insert((1.0, 1.0));
        if success {
            *a += 1.0;
        } else {
            *b += 1.0;
        }
    }

    /// Save to `~/.pure-reason/bayesian_weights.json`.
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::default_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, json)
    }

    /// Load from disk, or initialise with fresh priors.
    pub fn load() -> Self {
        match std::fs::read_to_string(Self::default_path()) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_else(|_| Self::new()),
            Err(_) => Self::new(),
        }
    }

    fn default_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".pure-reason")
            .join("bayesian_weights.json")
    }
}

impl Default for BayesianWeightMatrix {
    fn default() -> Self {
        Self::new()
    }
}

// ─── DomainConstraint DSL (S-III-10) ─────────────────────────────────────────

/// A set of domain-specific epistemic constraints loaded from a TOML file.
///
/// ## Format (`~/.pure-reason/constraints.toml` or `--domain-file <path>`)
/// ```toml
/// [[constraints]]
/// domain = "medical"
/// max_risk = "Medium"
/// forbidden_categories = ["Necessity", "Totality"]
/// require_hedging = true
/// violation_message = "Medical claims must not assert absolute necessity."
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DomainConstraints {
    pub constraints: Vec<DomainConstraint>,
}

/// A single domain constraint rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainConstraint {
    /// Domain tag that matches the input domain (e.g. `"medical"`, `"legal"`).
    pub domain: String,
    /// Maximum allowed risk level: `"Safe"`, `"Low"`, `"Medium"`, or `"High"`.
    pub max_risk: Option<String>,
    /// Category names whose presence should trigger a violation.
    #[serde(default)]
    pub forbidden_categories: Vec<String>,
    /// If `true`, verify the text contains epistemic hedges.
    #[serde(default)]
    pub require_hedging: bool,
    /// Optional human-readable message appended to violations.
    pub violation_message: Option<String>,
}

/// A constraint that was violated by a pipeline report.
#[derive(Debug, Clone)]
pub struct ConstraintViolation {
    pub domain: String,
    pub rule: String,
    pub detail: String,
    pub suggestion: Option<String>,
}

/// Checks a pipeline report against a set of domain constraints.
pub struct ConstraintChecker;

impl ConstraintChecker {
    /// Return all constraint violations found in `report` for the given `domain`.
    pub fn check(
        report: &crate::pipeline::PipelineReport,
        domain: &str,
        constraints: &DomainConstraints,
    ) -> Vec<ConstraintViolation> {
        let mut violations = Vec::new();
        for constraint in &constraints.constraints {
            if constraint.domain != domain {
                continue;
            }

            // Check max_risk
            if let Some(max) = &constraint.max_risk {
                let actual = report.verdict.risk.to_string();
                let exceeds = risk_exceeds(&actual, max);
                if exceeds {
                    violations.push(ConstraintViolation {
                        domain: domain.to_string(),
                        rule: "max_risk".to_string(),
                        detail: format!("Risk level {} exceeds allowed maximum {}", actual, max),
                        suggestion: constraint.violation_message.clone(),
                    });
                }
            }

            // Check forbidden categories
            if let Some(dom_cat) = &report.verdict.dominant_category {
                if constraint.forbidden_categories.iter().any(|f| f == dom_cat) {
                    violations.push(ConstraintViolation {
                        domain: domain.to_string(),
                        rule: "forbidden_categories".to_string(),
                        detail: format!(
                            "Forbidden category '{}' is dominant in this domain",
                            dom_cat
                        ),
                        suggestion: constraint.violation_message.clone(),
                    });
                }
            }

            // Check require_hedging
            if constraint.require_hedging {
                let hedge_words = [
                    "may", "might", "could", "perhaps", "possibly", "suggests", "appears", "seems",
                    "likely", "probably",
                ];
                let has_hedge = hedge_words
                    .iter()
                    .any(|h| report.input.to_lowercase().contains(h));
                if !has_hedge {
                    violations.push(ConstraintViolation {
                        domain: domain.to_string(),
                        rule: "require_hedging".to_string(),
                        detail: "No epistemic hedging found (domain requires hedged language)"
                            .to_string(),
                        suggestion: constraint.violation_message.clone(),
                    });
                }
            }
        }
        violations
    }

    /// Load constraints from a TOML file path.
    pub fn load_from_file(path: &std::path::Path) -> Result<DomainConstraints, String> {
        let s = std::fs::read_to_string(path)
            .map_err(|e| format!("Cannot read constraints file: {}", e))?;
        toml::from_str(&s).map_err(|e| format!("Invalid constraints TOML: {}", e))
    }

    /// Load from the default path `~/.pure-reason/constraints.toml`, or return empty.
    pub fn load_default() -> DomainConstraints {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let path = PathBuf::from(home)
            .join(".pure-reason")
            .join("constraints.toml");
        Self::load_from_file(&path).unwrap_or_default()
    }
}

/// Returns true if `actual` risk level exceeds `max`.
fn risk_exceeds(actual: &str, max: &str) -> bool {
    let rank = |s: &str| match s.to_uppercase().as_str() {
        "SAFE" => 0,
        "LOW" => 1,
        "MEDIUM" => 2,
        "HIGH" => 3,
        _ => 4,
    };
    rank(actual) > rank(max)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_schema_merge_adds_domain() {
        let mut schema = WorldSchema::default();
        schema.merge_suggestion("science", "Causality", Some("causes"));
        assert!(schema.domain("science").is_some());
        let d = schema.domain("science").unwrap();
        assert!(d.expected_categories.contains(&"Causality".to_string()));
        assert!(d.causal_signals.contains(&"causes".to_string()));
    }

    #[test]
    fn world_schema_merge_idempotent_for_same_cat() {
        let mut schema = WorldSchema::default();
        schema.merge_suggestion("science", "Causality", Some("causes"));
        schema.merge_suggestion("science", "Causality", Some("causes"));
        let d = schema.domain("science").unwrap();
        assert_eq!(
            d.expected_categories
                .iter()
                .filter(|c| *c == "Causality")
                .count(),
            1
        );
    }

    #[test]
    fn schema_learner_load_returns_empty_on_missing() {
        let learner = SchemaLearner {
            feedback_path: PathBuf::from("/nonexistent/feedback.jsonl"),
            schema_path: PathBuf::from("/nonexistent/schema.toml"),
        };
        let schema = learner.load_schema();
        assert!(schema.domains.is_empty());
    }

    #[test]
    fn bayesian_weight_matrix_initialises_from_priors() {
        let m = BayesianWeightMatrix::new();
        // Scientific/Causality should have a high prior weight
        let w = m.weight("Causality", "Scientific");
        assert!(
            w > 0.5,
            "Scientific/Causality prior should exceed 0.5, got {}",
            w
        );
    }

    #[test]
    fn bayesian_weight_update_shifts_posterior() {
        let mut m = BayesianWeightMatrix::new();
        let before = m.weight("Causality", "Scientific");
        for _ in 0..20 {
            m.update("Causality", "Scientific", false);
        }
        let after = m.weight("Causality", "Scientific");
        assert!(after < before, "20 failures should lower the posterior");
    }

    #[test]
    fn bayesian_weight_unknown_key_returns_half() {
        let m = BayesianWeightMatrix::new();
        assert_eq!(m.weight("NoSuchCategory", "NoSuchForm"), 0.5);
    }

    #[test]
    fn bayesian_weight_roundtrip_json() {
        let mut m = BayesianWeightMatrix::new();
        m.update("Necessity", "Moral/Ethical", true);
        let json = serde_json::to_string(&m).unwrap();
        let loaded: BayesianWeightMatrix = serde_json::from_str(&json).unwrap();
        let w1 = m.weight("Necessity", "Moral/Ethical");
        let w2 = loaded.weight("Necessity", "Moral/Ethical");
        assert!((w1 - w2).abs() < 1e-9);
    }

    #[test]
    fn constraint_checker_no_violation_on_safe() {
        use crate::pipeline::KantianPipeline;
        let pipeline = KantianPipeline::new();
        let report = pipeline
            .process("Water freezes at 0 degrees Celsius.")
            .unwrap();
        let constraints = DomainConstraints {
            constraints: vec![DomainConstraint {
                domain: "science".to_string(),
                max_risk: Some("High".to_string()),
                forbidden_categories: vec![],
                require_hedging: false,
                violation_message: None,
            }],
        };
        let violations = ConstraintChecker::check(&report, "science", &constraints);
        assert!(
            violations.is_empty(),
            "Safe scientific claim should have no violations"
        );
    }

    #[test]
    fn constraint_checker_flags_missing_hedge() {
        use crate::pipeline::KantianPipeline;
        let pipeline = KantianPipeline::new();
        let report = pipeline
            .process("This treatment cures all diseases.")
            .unwrap();
        let constraints = DomainConstraints {
            constraints: vec![DomainConstraint {
                domain: "medical".to_string(),
                max_risk: Some("High".to_string()),
                forbidden_categories: vec![],
                require_hedging: true,
                violation_message: Some("Use hedged language in medical claims.".to_string()),
            }],
        };
        let violations = ConstraintChecker::check(&report, "medical", &constraints);
        assert!(
            violations.iter().any(|v| v.rule == "require_hedging"),
            "Should flag missing hedge in medical domain"
        );
    }
}
