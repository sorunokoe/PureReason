//! # Claim IR and Per-Claim Epistemic Annotation
//!
//! Compiles text into atomic claims with lightweight structure:
//! - source role (knowledge/question/answer/raw),
//! - subject/predicate/object heuristics,
//! - modality and polarity,
//! - per-claim epistemic risk,
//! - and local evidence binding where context exists.
//!
//! This is the current "claim-first" reasoning surface for PureReason.

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::{
    aesthetic::{AestheticLayer, SegmentedInput},
    dialectic::{
        AntinomyDetector, IllusionDetector, KeywordSemanticField, LexicalCoverageAnalyzer,
        ParalogismDetector, SemanticField,
    },
    error::Result,
    pipeline::RiskLevel,
    types::{Confidence, Faculty, Proposition, PropositionKind},
};

// ─── Claim Roles and Structure ────────────────────────────────────────────────

/// Which segment of the input produced this claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimRole {
    Raw,
    Knowledge,
    Question,
    Answer,
}

impl ClaimRole {
    fn short_name(self) -> &'static str {
        match self {
            Self::Raw => "raw",
            Self::Knowledge => "knowledge",
            Self::Question => "question",
            Self::Answer => "answer",
        }
    }
}

impl std::fmt::Display for ClaimRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.short_name())
    }
}

// ─── ClaimType / NanoType (TRIZ Report VIII — S1) ────────────────────────────

/// Atomic claim type — the NanoType classifier output.
///
/// Classifies each claim into one of six types before evidence binding.
/// This enables type-specific precision models and budget allocation (S8 UBP).
///
/// Key insight: `Rhetorical` claims carry no falsifiable factual content and
/// should never drive hallucination verdicts on their own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimType {
    /// Contains a number + unit or assertion about a physical/biological constant.
    /// High FELM risk — routed to Numeric Plausibility Detector.
    Numeric,
    /// References a specific date, year, era, or temporal ordering.
    /// Checked by Temporal Coherence Layer.
    Temporal,
    /// Asserts a named entity's property, identity, or relationship.
    /// Primary HaluEval / RAGTruth / FaithBench target — KAC + entity grounding.
    Factual,
    /// Contains "because", "therefore", "causes", "results in", etc.
    /// Causal overreach detector.
    Causal,
    /// Policy, obligation, permission language ("must", "should", "required by law").
    /// Checked against domain constraint rules.
    Normative,
    /// Superlatives, vague marketing, opinion without factual assertion.
    /// Zero factual content — never drives a hallucination verdict alone.
    Rhetorical,
}

impl std::fmt::Display for ClaimType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::Numeric => "Numeric",
            Self::Temporal => "Temporal",
            Self::Factual => "Factual",
            Self::Causal => "Causal",
            Self::Normative => "Normative",
            Self::Rhetorical => "Rhetorical",
        };
        f.write_str(label)
    }
}

/// Classify a claim sentence into its NanoType.
///
/// Uses deterministic surface-pattern matching — zero cost, one pass.
/// Priority order: Numeric > Temporal > Causal > Normative > Rhetorical > Factual.
pub fn classify_claim_type(text: &str) -> ClaimType {
    let lower = text.to_lowercase();

    // Numeric: number + unit pattern, scientific notation, or known constant names
    if is_numeric_claim(&lower) {
        return ClaimType::Numeric;
    }

    // Temporal: specific years, dates, eras
    if is_temporal_claim(&lower) {
        return ClaimType::Temporal;
    }

    // Causal: explicit causation language
    let causal_markers = [
        "because ",
        "therefore ",
        "causes ",
        "cause ",
        "results in ",
        "leads to ",
        "due to ",
        "owing to ",
        "as a result ",
        "consequently ",
        "thus ",
        "hence ",
        "so that ",
        "in order to ",
    ];
    if causal_markers.iter().any(|m| lower.contains(m)) {
        return ClaimType::Causal;
    }

    // Normative: obligation / permission / policy language
    let normative_markers = [
        "must ",
        "shall ",
        "should ",
        "required to ",
        "required by ",
        "it is illegal",
        "against the law",
        "you are not allowed",
        "is prohibited",
        "is mandated",
        "policy requires",
        "regulation states",
        "law requires",
        "it is mandatory",
    ];
    if normative_markers.iter().any(|m| lower.contains(m)) {
        return ClaimType::Normative;
    }

    // Rhetorical: superlatives, vague evaluative language, no falsifiable assertion
    let rhetorical_markers = [
        "best ",
        "worst ",
        "greatest ",
        "most innovative",
        "revolutionary",
        "game-changing",
        "unprecedented",
        "world-class",
        "cutting-edge",
        "state of the art",
        "arguably",
        "in my opinion",
        "some say",
        "many believe",
        "it is thought",
        "it is felt",
        "commonly regarded",
        "widely considered",
        "generally seen as",
    ];
    if rhetorical_markers.iter().any(|m| lower.contains(m)) {
        // Only Rhetorical if no named-entity structure detected
        let has_entity = has_named_entity_structure(text);
        if !has_entity {
            return ClaimType::Rhetorical;
        }
    }

    // Default: Factual assertion about named entities
    ClaimType::Factual
}

fn is_numeric_claim(lower: &str) -> bool {
    // Scientific notation or measurement: "3.0 × 10^8", "6.626e-34", "299,792 km/s"
    let numeric_patterns = [
        "× 10",
        "x 10",
        "e-",
        "e+",
        "10^",
        "10−",
        " km/s",
        " m/s",
        " mph",
        " kph",
        " hz",
        " khz",
        " mhz",
        " ghz",
        " kg",
        " grams",
        " mg",
        " ml",
        " liters",
        " metres",
        " meters",
        " celsius",
        " fahrenheit",
        " kelvin",
        " joules",
        " watts",
        " volts",
        " amperes",
        " ohms",
        "%",
        " percent",
        " per cent",
        "speed of light",
        "planck constant",
        "gravitational constant",
        "avogadro",
        "boltzmann",
        "electron mass",
        "proton mass",
        "melting point",
        "boiling point",
        "atomic number",
        "atomic weight",
    ];
    if numeric_patterns.iter().any(|p| lower.contains(p)) {
        return true;
    }
    // Plain number followed by a unit word (e.g., "the temperature is 37 degrees")
    let has_digit = lower.chars().any(|c| c.is_ascii_digit());
    let unit_words = [
        " degree",
        " meter",
        " kilomet",
        " mile",
        " pound",
        " kilogram",
        " year",
        " century",
        " decade",
        " million",
        " billion",
        " trillion",
        " light-year",
        " parsec",
        " nanometer",
        " micrometer",
    ];
    has_digit && unit_words.iter().any(|u| lower.contains(u))
}

fn is_temporal_claim(lower: &str) -> bool {
    // Year patterns: 4-digit year in the range 1000–2099
    let has_year = (1000u32..=2099).any(|y| lower.contains(&y.to_string()));
    if has_year {
        return true;
    }
    let temporal_markers = [
        "in the ",
        "during the ",
        "by the ",
        "after the ",
        "before the ",
        "century",
        "decade",
        "era",
        "period",
        "ancient",
        "medieval",
        "renaissance",
        "modern",
        "contemporary",
        "recent",
        "historic",
        "founded in",
        "born in",
        "died in",
        "established in",
        "created in",
        "discovered in",
        "invented in",
        "published in",
    ];
    temporal_markers.iter().any(|m| lower.contains(m))
}

fn has_named_entity_structure(lower: &str) -> bool {
    // Rough heuristic: capital letter at start of a word (after stripping lowercase)
    // Not perfect, but fast and zero-cost.
    lower.split_whitespace().any(|word| {
        word.chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
    })
}

/// Whether a claim type carries falsifiable factual content.
///
/// Rhetorical claims do NOT — they should never drive the hallucination verdict alone.
pub fn claim_type_is_factual(ct: ClaimType) -> bool {
    !matches!(ct, ClaimType::Rhetorical)
}

/// Lightweight claim modality aligned to the existing Kantian calibration bands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimModality {
    Interrogative,
    Problematic,
    Assertoric,
    Apodeictic,
}

impl std::fmt::Display for ClaimModality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::Interrogative => "Interrogative",
            Self::Problematic => "Problematic",
            Self::Assertoric => "Assertoric",
            Self::Apodeictic => "Apodeictic",
        };
        f.write_str(label)
    }
}

/// Binary polarity for a claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimPolarity {
    Affirmative,
    Negative,
}

impl std::fmt::Display for ClaimPolarity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Affirmative => f.write_str("Affirmative"),
            Self::Negative => f.write_str("Negative"),
        }
    }
}

/// Evidence state for a compiled claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimEvidenceStatus {
    /// The claim is itself part of the supplied context (knowledge/question).
    InContext,
    /// A context sentence supports the claim.
    Supported,
    /// A context sentence contradicts the claim.
    Contradicted,
    /// The claim introduces novel entities relative to the context.
    Novel,
    /// Some context exists, but not enough to support or refute the claim.
    Unresolved,
    /// No external context was available.
    MissingContext,
}

impl std::fmt::Display for ClaimEvidenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::InContext => "InContext",
            Self::Supported => "Supported",
            Self::Contradicted => "Contradicted",
            Self::Novel => "Novel",
            Self::Unresolved => "Unresolved",
            Self::MissingContext => "MissingContext",
        };
        f.write_str(label)
    }
}

/// Local evidence binding for a claim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimEvidenceBinding {
    pub status: ClaimEvidenceStatus,
    /// Which segment supplied the best supporting/counter context.
    pub source_role: Option<ClaimRole>,
    /// The context sentence used for the local binding decision.
    pub matched_context: Option<String>,
    /// Similarity signal in [0.0, 1.0].
    pub support_score: f64,
    /// Explicit contradiction signal in [0.0, 1.0].
    pub contradiction_score: f64,
    /// Entity substitution / mismatch signal in [0.0, 1.0].
    pub entity_conflict_score: f64,
    /// Novelty relative to the full context, when meaningful.
    pub novelty_score: Option<f64>,
    /// Entities present in the claim but absent from the context.
    pub uncovered_entities: Vec<String>,
}

impl ClaimEvidenceBinding {
    fn in_context(role: ClaimRole, text: &str) -> Self {
        Self {
            status: ClaimEvidenceStatus::InContext,
            source_role: Some(role),
            matched_context: Some(text.to_string()),
            support_score: 1.0,
            contradiction_score: 0.0,
            entity_conflict_score: 0.0,
            novelty_score: None,
            uncovered_entities: Vec::new(),
        }
    }

    fn missing_context() -> Self {
        Self {
            status: ClaimEvidenceStatus::MissingContext,
            source_role: None,
            matched_context: None,
            support_score: 0.0,
            contradiction_score: 0.0,
            entity_conflict_score: 0.0,
            novelty_score: None,
            uncovered_entities: Vec::new(),
        }
    }
}

// ─── ClaimAnnotation ─────────────────────────────────────────────────────────

/// Structured epistemic annotation for a single claim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimAnnotation {
    /// Stable per-report identifier (`answer:2`, `raw:0`, ...).
    pub claim_id: String,
    /// The sentence text (after aesthetic segmentation).
    pub text: String,
    /// Zero-based index in the extracted claim stream.
    pub sentence_index: usize,
    /// Which segment this claim came from.
    pub source_role: ClaimRole,
    /// NanoType: atomic claim type classifier (TRIZ Report VIII S1).
    pub nano_type: ClaimType,
    /// Lightweight subject/predicate/object decomposition.
    pub subject: Option<String>,
    pub predicate: Option<String>,
    pub object: Option<String>,
    pub modality: ClaimModality,
    pub polarity: ClaimPolarity,
    /// Risk level for this specific claim.
    pub risk: RiskLevel,
    /// Illusion descriptions detected in this claim.
    pub illusion_issues: Vec<String>,
    /// Antinomy-relevant signals found in this claim.
    pub antinomy_issues: Vec<String>,
    /// Paralogism descriptions detected in this claim.
    pub paralogism_issues: Vec<String>,
    /// Local evidence binding against available context.
    pub evidence: ClaimEvidenceBinding,
    /// True if this claim has no epistemic issues.
    pub is_safe: bool,
}

impl ClaimAnnotation {
    fn build(seed: ClaimSeed, context: &BindingContext) -> Self {
        let nano_type = classify_claim_type(&seed.text);
        let (mut risk, illusion_issues, antinomy_issues, paralogism_issues) =
            diagnose_claim(&seed.text);

        // S1 NanoType suppression: Rhetorical claims carry no falsifiable content.
        // If the only issues are from the rhetorical pattern, cap risk at Low.
        if nano_type == ClaimType::Rhetorical
            && illusion_issues.is_empty()
            && antinomy_issues.is_empty()
            && paralogism_issues.is_empty()
            && risk > RiskLevel::Low
        {
            risk = RiskLevel::Low;
        }

        let is_safe = risk == RiskLevel::Safe;
        let (subject, predicate, object) = extract_claim_shape(&seed.text);
        let modality = detect_modality(&seed.text, seed.source_role);
        let polarity = detect_polarity(&seed.text);
        let evidence = bind_evidence(&seed, context);

        Self {
            claim_id: format!("{}:{}", seed.source_role, seed.sentence_index),
            text: seed.text,
            sentence_index: seed.sentence_index,
            source_role: seed.source_role,
            nano_type,
            subject,
            predicate,
            object,
            modality,
            polarity,
            risk,
            illusion_issues,
            antinomy_issues,
            paralogism_issues,
            evidence,
            is_safe,
        }
    }
}

// ─── ClaimAnnotatedReport ────────────────────────────────────────────────────

/// Full claim-first analysis of an input text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimAnnotatedReport {
    /// The original input.
    pub input: String,
    /// Per-claim annotations.
    pub claims: Vec<ClaimAnnotation>,
    /// Highest risk level across all claims.
    pub overall_risk: RiskLevel,
    /// Number of safe (no-issue) claims.
    pub safe_count: usize,
    /// Number of claims with at least one issue.
    pub risky_count: usize,
    /// Whether the input contained any usable external context.
    pub context_available: bool,
    /// Claims that are themselves part of the provided context.
    pub in_context_count: usize,
    pub supported_count: usize,
    pub contradicted_count: usize,
    pub novel_count: usize,
    pub unresolved_count: usize,
    pub missing_context_count: usize,
}

// ─── annotate_claims ─────────────────────────────────────────────────────────

/// Compile and annotate claims from `input`.
///
/// This is the claim-first reasoning surface used by both the dedicated `claims`
/// command and the pipeline's new claim analysis section.
pub fn annotate_claims(input: &str) -> Result<ClaimAnnotatedReport> {
    let segmented = SegmentedInput::parse(input);
    annotate_claims_from_segmented(input, &segmented)
}

/// Compile and annotate claims from an already-parsed [`SegmentedInput`].
///
/// Use this variant inside the pipeline to avoid re-parsing the input text
/// that Stage 1 (AestheticLayer) has already segmented.
pub fn annotate_claims_from_segmented(
    input: &str,
    segmented: &SegmentedInput,
) -> Result<ClaimAnnotatedReport> {
    let seeds = collect_claim_seeds(segmented)?;
    let context = BindingContext::from_segmented(segmented)?;

    let claims: Vec<ClaimAnnotation> = seeds
        .into_iter()
        .map(|seed| ClaimAnnotation::build(seed, &context))
        .collect();

    let overall_risk = claims
        .iter()
        .map(|c| c.risk)
        .max()
        .unwrap_or(RiskLevel::Safe);
    let safe_count = claims.iter().filter(|c| c.is_safe).count();
    let risky_count = claims.len().saturating_sub(safe_count);

    let in_context_count = count_status(&claims, ClaimEvidenceStatus::InContext);
    let supported_count = count_status(&claims, ClaimEvidenceStatus::Supported);
    let contradicted_count = count_status(&claims, ClaimEvidenceStatus::Contradicted);
    let novel_count = count_status(&claims, ClaimEvidenceStatus::Novel);
    let unresolved_count = count_status(&claims, ClaimEvidenceStatus::Unresolved);
    let missing_context_count = count_status(&claims, ClaimEvidenceStatus::MissingContext);

    Ok(ClaimAnnotatedReport {
        input: input.to_string(),
        claims,
        overall_risk,
        safe_count,
        risky_count,
        context_available: context.full_context.is_some(),
        in_context_count,
        supported_count,
        contradicted_count,
        novel_count,
        unresolved_count,
        missing_context_count,
    })
}

// ─── Internal claim collection ───────────────────────────────────────────────

#[derive(Debug, Clone)]
struct ClaimSeed {
    text: String,
    sentence_index: usize,
    source_role: ClaimRole,
}

fn collect_claim_seeds(segmented: &SegmentedInput) -> Result<Vec<ClaimSeed>> {
    let mut seeds = Vec::new();
    let mut next_index = 0usize;

    if segmented.knowledge.is_some() || segmented.question.is_some() || segmented.answer.is_some() {
        if let Some(text) = segmented.knowledge.as_deref() {
            push_claims(ClaimRole::Knowledge, text, &mut next_index, &mut seeds)?;
        }
        if let Some(text) = segmented.question.as_deref() {
            push_claims(ClaimRole::Question, text, &mut next_index, &mut seeds)?;
        }
        if let Some(text) = segmented.answer.as_deref() {
            push_claims(ClaimRole::Answer, text, &mut next_index, &mut seeds)?;
        }
    } else {
        push_claims(ClaimRole::Raw, &segmented.raw, &mut next_index, &mut seeds)?;
    }

    Ok(seeds)
}

fn push_claims(
    role: ClaimRole,
    text: &str,
    next_index: &mut usize,
    seeds: &mut Vec<ClaimSeed>,
) -> Result<()> {
    for sentence in split_sentences(text)? {
        seeds.push(ClaimSeed {
            text: sentence,
            sentence_index: *next_index,
            source_role: role,
        });
        *next_index += 1;
    }
    Ok(())
}

fn split_sentences(text: &str) -> Result<Vec<String>> {
    let intuition = AestheticLayer.apply(text.to_string())?;
    let mut sentences: Vec<String> = intuition
        .manifold
        .sentences
        .into_iter()
        .map(|s| normalize_sentence(&s))
        .filter(|s| !s.is_empty())
        .collect();

    if sentences.is_empty() {
        let fallback = normalize_sentence(text);
        if !fallback.is_empty() {
            sentences.push(fallback);
        }
    }

    Ok(sentences)
}

fn normalize_sentence(text: &str) -> String {
    text.trim()
        .trim_matches(|c: char| matches!(c, '.' | '!' | '?'))
        .trim()
        .to_string()
}

// ─── Evidence binding ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContextMode {
    None,
    QuestionOnly,
    GroundedKnowledge,
}

#[derive(Debug, Clone)]
struct ContextSentence {
    role: ClaimRole,
    text: String,
}

#[derive(Debug, Clone)]
struct BindingContext {
    mode: ContextMode,
    full_context: Option<String>,
    context_sentences: Vec<ContextSentence>,
}

impl BindingContext {
    fn from_segmented(segmented: &SegmentedInput) -> Result<Self> {
        let mut context_sentences = Vec::new();
        let mut full_context_parts = Vec::new();

        if let Some(knowledge) = segmented.knowledge.as_deref() {
            full_context_parts.push(knowledge.trim().to_string());
            for sentence in split_sentences(knowledge)? {
                context_sentences.push(ContextSentence {
                    role: ClaimRole::Knowledge,
                    text: sentence,
                });
            }
        }

        if let Some(question) = segmented.question.as_deref() {
            full_context_parts.push(question.trim().to_string());
            for sentence in split_sentences(question)? {
                context_sentences.push(ContextSentence {
                    role: ClaimRole::Question,
                    text: sentence,
                });
            }
        }

        let mode = if segmented.has_knowledge_answer_context() {
            ContextMode::GroundedKnowledge
        } else if segmented.has_question_answer_context() {
            ContextMode::QuestionOnly
        } else {
            ContextMode::None
        };

        let full_context = if full_context_parts.is_empty() {
            None
        } else {
            Some(full_context_parts.join(" "))
        };

        Ok(Self {
            mode,
            full_context,
            context_sentences,
        })
    }
}

#[derive(Debug, Clone)]
struct MatchCandidate {
    source_role: ClaimRole,
    matched_context: String,
    support_score: f64,
    contradiction_score: f64,
    entity_conflict_score: f64,
}

fn bind_evidence(seed: &ClaimSeed, context: &BindingContext) -> ClaimEvidenceBinding {
    match seed.source_role {
        ClaimRole::Knowledge | ClaimRole::Question => {
            ClaimEvidenceBinding::in_context(seed.source_role, &seed.text)
        }
        ClaimRole::Answer | ClaimRole::Raw => match context.mode {
            ContextMode::None => ClaimEvidenceBinding::missing_context(),
            ContextMode::QuestionOnly => bind_question_scope(seed, context),
            ContextMode::GroundedKnowledge => bind_grounded(seed, context),
        },
    }
}

fn bind_question_scope(seed: &ClaimSeed, context: &BindingContext) -> ClaimEvidenceBinding {
    let field = KeywordSemanticField;
    let Some(best) = best_context_match(seed, context, &field) else {
        return ClaimEvidenceBinding::missing_context();
    };

    let status = if best.support_score >= 0.15 {
        ClaimEvidenceStatus::Unresolved
    } else {
        ClaimEvidenceStatus::Novel
    };

    ClaimEvidenceBinding {
        status,
        source_role: Some(best.source_role),
        matched_context: Some(best.matched_context),
        support_score: round_signal(best.support_score),
        contradiction_score: 0.0,
        entity_conflict_score: 0.0,
        novelty_score: None,
        uncovered_entities: Vec::new(),
    }
}

fn bind_grounded(seed: &ClaimSeed, context: &BindingContext) -> ClaimEvidenceBinding {
    let field = KeywordSemanticField;
    let Some(best) = best_context_match(seed, context, &field) else {
        return ClaimEvidenceBinding::missing_context();
    };

    let coverage =
        LexicalCoverageAnalyzer::analyze(context.full_context.as_deref().unwrap_or(""), &seed.text);

    let status = if best.contradiction_score >= 0.55 || best.entity_conflict_score >= 0.25 {
        ClaimEvidenceStatus::Contradicted
    } else if coverage.novelty_score >= 0.55 {
        ClaimEvidenceStatus::Novel
    } else if best.support_score >= 0.25 || coverage.coverage_ratio >= 0.50 {
        ClaimEvidenceStatus::Supported
    } else {
        ClaimEvidenceStatus::Unresolved
    };

    ClaimEvidenceBinding {
        status,
        source_role: Some(best.source_role),
        matched_context: Some(best.matched_context),
        support_score: round_signal(best.support_score),
        contradiction_score: round_signal(best.contradiction_score),
        entity_conflict_score: round_signal(best.entity_conflict_score),
        novelty_score: Some(round_signal(coverage.novelty_score)),
        uncovered_entities: coverage.uncovered_entities,
    }
}

fn best_context_match(
    seed: &ClaimSeed,
    context: &BindingContext,
    field: &impl SemanticField,
) -> Option<MatchCandidate> {
    context
        .context_sentences
        .iter()
        .map(|candidate| {
            let support_score = field.similarity(&candidate.text, &seed.text);
            let contradiction_score = field
                .is_negation_of(&candidate.text, &seed.text)
                .max(field.is_negation_of(&seed.text, &candidate.text));
            let entity_conflict_score = if matches!(context.mode, ContextMode::GroundedKnowledge) {
                field.entity_conflict_score(&candidate.text, &seed.text)
            } else {
                0.0
            };

            MatchCandidate {
                source_role: candidate.role,
                matched_context: candidate.text.clone(),
                support_score,
                contradiction_score,
                entity_conflict_score,
            }
        })
        .max_by(|a, b| {
            candidate_rank(a)
                .partial_cmp(&candidate_rank(b))
                .unwrap_or(Ordering::Equal)
        })
}

fn candidate_rank(candidate: &MatchCandidate) -> f64 {
    let role_bias = match candidate.source_role {
        ClaimRole::Knowledge => 0.10,
        ClaimRole::Question => 0.00,
        ClaimRole::Raw | ClaimRole::Answer => 0.00,
    };

    candidate
        .support_score
        .max(candidate.contradiction_score)
        .max(candidate.entity_conflict_score)
        + role_bias
}

// ─── Claim diagnostics ────────────────────────────────────────────────────────

fn diagnose_claim(text: &str) -> (RiskLevel, Vec<String>, Vec<String>, Vec<String>) {
    let prop = Proposition::new(text, PropositionKind::Unknown);

    let illusions = IllusionDetector::detect(std::slice::from_ref(&prop));
    let antinomies = AntinomyDetector::detect(std::slice::from_ref(&prop));
    let paralogisms = ParalogismDetector::detect(&[prop]);

    let mut illusion_issues: Vec<String> =
        illusions.iter().map(|i| i.description.clone()).collect();
    let antinomy_issues: Vec<String> = antinomies
        .iter()
        .filter(|a| a.has_conflict)
        .map(|a| a.description.clone())
        .collect();
    let paralogism_issues: Vec<String> = paralogisms
        .iter()
        .filter(|p| p.has_paralogisms)
        .flat_map(|p| p.detected.iter().map(|d| d.trigger.clone()))
        .collect();

    // S3 NPD: check numeric plausibility for Numeric NanoType claims.
    // Classify first (cheap), then only call NPD scanner for numeric claims.
    let nano_type = crate::claims::classify_claim_type(text);
    if nano_type == crate::claims::ClaimType::Numeric {
        if let Some(issue) = crate::numeric_plausibility::NumericPlausibilityScanner::scan(text) {
            illusion_issues.push(format!("NumericPlausibility: {}", issue));
        }
    }

    // S4 TCL: single-sentence temporal anomaly check (anachronism + recency overreach).
    // Cross-sentence contradiction is only checked at the report level (annotate_claims).
    let temporal_issues = crate::temporal_coherence::TemporalCoherenceLayer::scan(&[text]);
    for ti in &temporal_issues {
        illusion_issues.push(format!("TemporalCoherence: {}", ti));
    }

    let issue_count = usize::from(!illusion_issues.is_empty())
        + usize::from(!antinomy_issues.is_empty())
        + usize::from(!paralogism_issues.is_empty());

    let risk = match issue_count {
        0 => RiskLevel::Safe,
        1 => RiskLevel::Low,
        2 => RiskLevel::Medium,
        _ => RiskLevel::High,
    };

    (risk, illusion_issues, antinomy_issues, paralogism_issues)
}

fn extract_claim_shape(text: &str) -> (Option<String>, Option<String>, Option<String>) {
    let tokens: Vec<String> = text
        .split_whitespace()
        .map(|token| {
            token
                .trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '\'')
                .to_string()
        })
        .filter(|token| !token.is_empty())
        .collect();

    if tokens.is_empty() {
        return (None, None, None);
    }

    const PIVOTS: &[&str] = &[
        "is",
        "are",
        "was",
        "were",
        "be",
        "been",
        "being",
        "has",
        "have",
        "had",
        "causes",
        "cause",
        "caused",
        "requires",
        "require",
        "required",
        "indicates",
        "indicate",
        "indicated",
        "suggests",
        "suggest",
        "suggested",
        "means",
        "mean",
        "meant",
        "contains",
        "contain",
        "contained",
        "supports",
        "support",
        "supported",
        "contradicts",
        "contradict",
        "contradicted",
        "uses",
        "use",
        "used",
        "will",
        "may",
        "might",
        "must",
        "can",
        "could",
        "should",
        "would",
    ];

    let pivot = tokens
        .iter()
        .position(|token| PIVOTS.contains(&token.to_lowercase().as_str()));

    match pivot {
        Some(0) => (
            Some(tokens[0].clone()),
            None,
            tokens
                .get(1..)
                .map(|rest| rest.join(" "))
                .filter(|s| !s.is_empty()),
        ),
        Some(i) => (
            Some(tokens[..i].join(" ")).filter(|s| !s.is_empty()),
            Some(tokens[i].clone()),
            tokens
                .get(i + 1..)
                .map(|rest| rest.join(" "))
                .filter(|s| !s.is_empty()),
        ),
        None => (
            Some(tokens.iter().take(3).cloned().collect::<Vec<_>>().join(" "))
                .filter(|s| !s.is_empty()),
            None,
            None,
        ),
    }
}

fn detect_modality(text: &str, role: ClaimRole) -> ClaimModality {
    if matches!(role, ClaimRole::Question) {
        return ClaimModality::Interrogative;
    }

    let lower = text.to_lowercase();
    let starts_with_question_word = [
        "what ", "why ", "how ", "who ", "when ", "where ", "which ", "is ", "are ", "do ",
        "does ", "did ", "can ", "could ", "should ", "will ",
    ]
    .iter()
    .any(|prefix| lower.starts_with(prefix));

    if starts_with_question_word {
        return ClaimModality::Interrogative;
    }

    if [
        "must",
        "always",
        "never",
        "definitely",
        "certainly",
        "guaranteed",
        "inevitable",
        "required",
        "cannot fail",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
    {
        return ClaimModality::Apodeictic;
    }

    if [
        "may",
        "might",
        "could",
        "possible",
        "possibly",
        "appears",
        "appears to",
        "suggests",
        "likely",
        "unlikely",
        "consistent with",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
    {
        return ClaimModality::Problematic;
    }

    ClaimModality::Assertoric
}

fn detect_polarity(text: &str) -> ClaimPolarity {
    let lower = text.to_lowercase();
    if [
        " not ", " no ", " never ", " false ", " wrong ", " cannot ", "can't ", "n't ",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
        || lower.starts_with("no ")
        || lower.starts_with("not ")
    {
        ClaimPolarity::Negative
    } else {
        ClaimPolarity::Affirmative
    }
}

fn round_signal(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

fn count_status(claims: &[ClaimAnnotation], status: ClaimEvidenceStatus) -> usize {
    claims
        .iter()
        .filter(|claim| claim.evidence.status == status)
        .count()
}

// ─── S10: Claim Triple Canonical Form (TRIZ Report IX) ───────────────────────
//
// Shifts detection from surface text to normalized (Subject, Predicate, Object, Polarity)
// triples. Two paraphrases of the same fact map to the same triple. Contradictions
// become a polarity inversion on the same (S, P, O) key.

/// Provenance attached to every canonical triple.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimTripleProvenance {
    /// Stable identifier of the source claim annotation.
    pub claim_id: String,
    /// Which segment produced this triple.
    pub source_role: ClaimRole,
    /// Evidence status observed when the triple was compiled.
    pub evidence_status: ClaimEvidenceStatus,
    /// The context sentence used for the local evidence decision, if any.
    pub matched_context: Option<String>,
}

/// A normalized claim triple — the canonical representation of a factual assertion.
///
/// Two claims that express the same fact map to the same triple regardless of
/// surface wording. Contradictions are detected as polarity inversions on the
/// same (subject, predicate, object) key.
///
/// # Example
/// ```
/// // "Edison invented the telephone"       → (edison, invented, telephone, Affirmative)
/// // "The telephone was invented by Edison" → (edison, invented, telephone, Affirmative)
/// // "Bell, not Edison, invented the phone" → (edison, invented, telephone, Negative)
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaimTriple {
    /// Normalized subject entity (lowercase, articles stripped).
    pub subject: String,
    /// Canonical predicate / relation (lemmatized pivot verb).
    pub predicate: String,
    /// Normalized object entity or description.
    pub object: String,
    /// Whether the triple is affirmed or negated.
    pub polarity: ClaimPolarity,
    /// Confidence that the extracted SPO shape is strong enough for contradiction logic.
    pub extraction_confidence: Confidence,
    /// Minimum confidence required before this triple may participate in contradiction checks.
    pub contradiction_threshold: Confidence,
    /// Whether this triple's NanoType is eligible for contradiction tracking at all.
    pub triple_eligible: bool,
    /// Where the triple came from and what evidence surface produced it.
    pub provenance: ClaimTripleProvenance,
}

impl ClaimTriple {
    /// The (subject, predicate, object) key — ignores polarity.
    /// Two triples with the same key and different polarity contradict each other.
    pub fn spo_key(&self) -> String {
        format!("{}\x00{}\x00{}", self.subject, self.predicate, self.object)
    }

    /// Returns true when the triple is structurally strong enough to drive contradiction logic.
    pub fn supports_contradiction(&self) -> bool {
        self.triple_eligible
            && self.extraction_confidence.value() >= self.contradiction_threshold.value()
    }

    /// Returns true if `other` directly contradicts this triple.
    /// Contradiction = same SPO key, opposite polarity.
    pub fn contradicts(&self, other: &ClaimTriple) -> bool {
        self.supports_contradiction()
            && other.supports_contradiction()
            && self.spo_key() == other.spo_key()
            && self.polarity != other.polarity
    }
}

impl std::fmt::Display for ClaimTriple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let neg = if self.polarity == ClaimPolarity::Negative {
            "¬"
        } else {
            ""
        };
        write!(
            f,
            "({}, {}{}, {})",
            self.subject, neg, self.predicate, self.object
        )
    }
}

/// Convert a [`ClaimAnnotation`] to its canonical [`ClaimTriple`].
///
/// Uses the already-extracted subject/predicate/object fields, normalizes each
/// component, and applies the annotation's polarity.
pub fn annotation_to_triple(annotation: &ClaimAnnotation) -> ClaimTriple {
    let route = route_for_type(annotation.nano_type);
    let subject = annotation
        .subject
        .as_deref()
        .map(normalize_triple_component)
        .unwrap_or_else(|| normalize_triple_component(&annotation.text));

    let predicate = annotation
        .predicate
        .as_deref()
        .map(canonicalize_predicate)
        .unwrap_or_else(|| "is".to_string());
    let predicate = if annotation.polarity == ClaimPolarity::Negative {
        strip_negation_prefix(&predicate)
    } else {
        predicate
    };

    let object = annotation
        .object
        .as_deref()
        .map(normalize_triple_component)
        .unwrap_or_else(|| "unknown".to_string());
    let object = if annotation.polarity == ClaimPolarity::Negative {
        strip_negation_prefix(&object)
    } else {
        object
    };

    ClaimTriple {
        subject,
        predicate,
        object,
        polarity: annotation.polarity,
        extraction_confidence: triple_extraction_confidence(annotation, &route),
        contradiction_threshold: Confidence::new(route.escalation_threshold),
        triple_eligible: route.triple_eligible,
        provenance: ClaimTripleProvenance {
            claim_id: annotation.claim_id.clone(),
            source_role: annotation.source_role,
            evidence_status: annotation.evidence.status,
            matched_context: annotation.evidence.matched_context.clone(),
        },
    }
}

fn triple_extraction_confidence(annotation: &ClaimAnnotation, route: &ClaimRoute) -> Confidence {
    if !route.triple_eligible {
        return Confidence::impossible();
    }

    let mut score = 0.0;

    if annotation
        .subject
        .as_deref()
        .map(|s| !normalize_triple_component(s).is_empty())
        .unwrap_or(false)
    {
        score += 0.35;
    }
    if annotation
        .predicate
        .as_deref()
        .map(|p| !canonicalize_predicate(p).is_empty())
        .unwrap_or(false)
    {
        score += 0.25;
    }
    if annotation
        .object
        .as_deref()
        .map(|o| !normalize_triple_component(o).is_empty())
        .unwrap_or(false)
    {
        score += 0.20;
    }

    score += match annotation.modality {
        ClaimModality::Apodeictic => 0.15,
        ClaimModality::Assertoric => 0.10,
        ClaimModality::Problematic => 0.05,
        ClaimModality::Interrogative => 0.0,
    };

    score += match annotation.evidence.status {
        ClaimEvidenceStatus::InContext
        | ClaimEvidenceStatus::Supported
        | ClaimEvidenceStatus::Contradicted => 0.10,
        ClaimEvidenceStatus::Novel | ClaimEvidenceStatus::Unresolved => 0.05,
        ClaimEvidenceStatus::MissingContext => 0.0,
    };

    score += match annotation.risk {
        RiskLevel::Safe => 0.10,
        RiskLevel::Low => 0.05,
        RiskLevel::Medium => -0.05,
        RiskLevel::High => -0.15,
    };

    Confidence::new(score)
}

/// Normalize a triple component: lowercase, strip leading articles and punctuation,
/// collapse whitespace.
fn normalize_triple_component(text: &str) -> String {
    let lower = text.to_lowercase();
    // Strip leading articles
    let stripped = lower
        .trim_start_matches("the ")
        .trim_start_matches("a ")
        .trim_start_matches("an ");
    // Remove punctuation except hyphens
    stripped
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == ' ')
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn strip_negation_prefix(text: &str) -> String {
    let trimmed = text.trim();
    for prefix in ["not ", "never ", "no longer ", "no "] {
        if let Some(stripped) = trimmed.strip_prefix(prefix) {
            return stripped.trim().to_string();
        }
    }
    trimmed.to_string()
}

/// Map a predicate to its canonical/lemmatized form.
/// This collapses conjugations so "invented", "was invented by", "invents" → "invented".
fn canonicalize_predicate(pred: &str) -> String {
    let lower = pred.to_lowercase();

    // Passive construction: "was X by" → X
    if lower.starts_with("was ") || lower.starts_with("were ") || lower.starts_with("is ") {
        let stripped = lower
            .trim_start_matches("was ")
            .trim_start_matches("were ")
            .trim_start_matches("is ");
        // "was invented" → "invented", "was born" → "born"
        return normalize_triple_component(stripped.trim_end_matches(" by"));
    }

    // Strip trailing -s/-es for third-person singular
    let result = normalize_triple_component(&lower);
    if result.ends_with("ses") || result.ends_with("zes") {
        result[..result.len() - 2].trim_end_matches('s').to_string()
    } else if result.ends_with('s') && !result.ends_with("ss") && result.len() > 3 {
        // "invents" → "invent", "causes" → "cause"
        let bare = result.trim_end_matches('s');
        if bare.len() >= 3 {
            bare.to_string()
        } else {
            result
        }
    } else {
        result
    }
}

/// Find all contradicting pairs among a set of triples.
///
/// Returns pairs `(i, j)` where triple `i` contradicts triple `j`.
pub fn find_triple_contradictions(triples: &[ClaimTriple]) -> Vec<(usize, usize)> {
    let mut pairs = Vec::new();
    for i in 0..triples.len() {
        for j in (i + 1)..triples.len() {
            if triples[i].contradicts(&triples[j]) {
                pairs.push((i, j));
            }
        }
    }
    pairs
}

/// Convert all claims in a report to their canonical triples.
pub fn report_to_triples(report: &ClaimAnnotatedReport) -> Vec<ClaimTriple> {
    report.claims.iter().map(annotation_to_triple).collect()
}

// ─── S16: Claim-Type Routing Table (TRIZ Report IX) ─────────────────────────
//
// Maps each NanoType to the primary verifier, its ECS weight, and the minimum
// confidence threshold below which the claim should be escalated to the LLM
// hybrid layer. The routing decision is purely data-driven — no if/else chains
// in application code.
//
// Design goals (TRIZ Principle #6 — Universality, #10 — Preliminary Action):
//   - One call (`route_for_type`) replaces scattered switch statements.
//   - The table is the single source of truth for verifier assignment.
//   - Adding a new ClaimType only requires a new row in the match arm.

/// The verifier pipeline assigned to a claim type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimVerifier {
    /// Numeric plausibility detector (bounds check, unit coherence, constant atlas).
    NumericPlausibility,
    /// Temporal coherence layer (year bounds, chronological ordering).
    TemporalCoherence,
    /// Knowledge-grounded answering check (KAC) + entity fingerprint.
    FactualGrounding,
    /// Causal overreach detector (causal claim vs. supported mechanism).
    CausalOverreach,
    /// Domain constraint rules engine (legal, medical, policy).
    NormativeConstraints,
    /// No verifier — rhetorical claims carry zero factual risk.
    None,
}

impl std::fmt::Display for ClaimVerifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::NumericPlausibility => "numeric_plausibility",
            Self::TemporalCoherence => "temporal_coherence",
            Self::FactualGrounding => "factual_grounding",
            Self::CausalOverreach => "causal_overreach",
            Self::NormativeConstraints => "normative_constraints",
            Self::None => "none",
        };
        f.write_str(s)
    }
}

/// Routing decision for a single claim.
#[derive(Debug, Clone)]
pub struct ClaimRoute {
    /// The NanoType this route was derived from.
    pub claim_type: ClaimType,
    /// The primary verifier assigned to this claim type.
    pub verifier: ClaimVerifier,
    /// Weight of this claim type's verdict in the overall ECS (0.0–1.0).
    pub ecs_weight: f64,
    /// Minimum acceptable confidence; below this, escalate to LLM hybrid.
    pub escalation_threshold: f64,
    /// Whether this claim type participates in triple-contradiction detection.
    pub triple_eligible: bool,
}

/// Returns the routing decision for a given `ClaimType`.
///
/// The routing table is the single authoritative source for verifier assignment.
/// All pipeline components should call this instead of ad-hoc branching.
///
/// # Example
///
/// ```
/// use pure_reason_core::claims::{ClaimType, route_for_type};
///
/// let route = route_for_type(ClaimType::Numeric);
/// assert_eq!(route.ecs_weight, 0.30);
/// assert_eq!(route.triple_eligible, false);
/// ```
pub fn route_for_type(ct: ClaimType) -> ClaimRoute {
    match ct {
        ClaimType::Numeric => ClaimRoute {
            claim_type: ct,
            verifier: ClaimVerifier::NumericPlausibility,
            ecs_weight: 0.30,
            escalation_threshold: 0.55,
            triple_eligible: false, // numeric triples require unit normalization (future)
        },
        ClaimType::Temporal => ClaimRoute {
            claim_type: ct,
            verifier: ClaimVerifier::TemporalCoherence,
            ecs_weight: 0.20,
            escalation_threshold: 0.60,
            triple_eligible: true,
        },
        ClaimType::Factual => ClaimRoute {
            claim_type: ct,
            verifier: ClaimVerifier::FactualGrounding,
            ecs_weight: 0.25,
            escalation_threshold: 0.50,
            triple_eligible: true,
        },
        ClaimType::Causal => ClaimRoute {
            claim_type: ct,
            verifier: ClaimVerifier::CausalOverreach,
            ecs_weight: 0.15,
            escalation_threshold: 0.65,
            triple_eligible: true,
        },
        ClaimType::Normative => ClaimRoute {
            claim_type: ct,
            verifier: ClaimVerifier::NormativeConstraints,
            ecs_weight: 0.10,
            escalation_threshold: 0.70,
            triple_eligible: false, // normative claims are constraint-checked, not triple-matched
        },
        ClaimType::Rhetorical => ClaimRoute {
            claim_type: ct,
            verifier: ClaimVerifier::None,
            ecs_weight: 0.00,
            escalation_threshold: 1.00, // never escalate — no factual content
            triple_eligible: false,
        },
    }
}

/// Summarise the routing decisions for all claims in an annotated report.
///
/// Returns one `ClaimRoute` per distinct `ClaimType` found, sorted by `ecs_weight` descending.
/// Use this to understand which verifiers dominate a given input.
pub fn route_summary(report: &ClaimAnnotatedReport) -> Vec<ClaimRoute> {
    let mut seen = std::collections::HashSet::new();
    let mut routes: Vec<ClaimRoute> = report
        .claims
        .iter()
        .filter_map(|ann| {
            if seen.insert(format!("{}", ann.nano_type)) {
                Some(route_for_type(ann.nano_type))
            } else {
                None
            }
        })
        .collect();
    routes.sort_by(|a, b| {
        b.ecs_weight
            .partial_cmp(&a.ecs_weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    routes
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_text_annotates_all_safe() {
        let report =
            annotate_claims("Water boils at 100 degrees Celsius. The sky is blue.").unwrap();
        assert!(report.claims.iter().all(|claim| claim.is_safe));
        assert_eq!(report.overall_risk, RiskLevel::Safe);
        assert_eq!(report.missing_context_count, 2);
    }

    #[test]
    fn theological_claim_flagged() {
        let report = annotate_claims("God exists. Water is wet.").unwrap();
        let theological = report
            .claims
            .iter()
            .find(|claim| claim.text.to_lowercase().contains("god"))
            .unwrap();
        assert!(!theological.is_safe, "God illusion should be flagged");
        assert_eq!(theological.modality, ClaimModality::Assertoric);
    }

    #[test]
    fn risky_count_correct() {
        let report =
            annotate_claims("God exists. Everything is determined. The sky is blue.").unwrap();
        assert!(report.risky_count >= 2);
    }

    #[test]
    fn sentence_indices_monotonic() {
        let report = annotate_claims("First. Second. Third.").unwrap();
        for (index, claim) in report.claims.iter().enumerate() {
            assert_eq!(claim.sentence_index, index);
        }
    }

    #[test]
    fn grounded_answer_is_bound_to_context() {
        let report = annotate_claims(
            "Knowledge: The capital of Australia is Canberra.\n\
             Question: What is the capital of Australia?\n\
             Answer: Canberra is the capital of Australia.",
        )
        .unwrap();

        let answer = report
            .claims
            .iter()
            .find(|claim| claim.source_role == ClaimRole::Answer)
            .unwrap();

        assert_eq!(answer.evidence.status, ClaimEvidenceStatus::Supported);
        assert_eq!(answer.subject.as_deref(), Some("Canberra"));
        assert_eq!(answer.predicate.as_deref(), Some("is"));
    }

    #[test]
    fn contradictory_answer_is_marked_contradicted() {
        let report = annotate_claims(
            "Knowledge: The capital of Australia is Canberra.\n\
             Question: What is the capital of Australia?\n\
             Answer: Sydney is the capital of Australia.",
        )
        .unwrap();

        let answer = report
            .claims
            .iter()
            .find(|claim| claim.source_role == ClaimRole::Answer)
            .unwrap();

        assert_eq!(answer.evidence.status, ClaimEvidenceStatus::Contradicted);
        assert!(answer
            .evidence
            .uncovered_entities
            .iter()
            .any(|entity| entity == "sydney"));
    }

    #[test]
    fn question_only_answer_stays_unresolved_not_supported() {
        let report = annotate_claims(
            "Question: Who wrote Hamlet?\n\
             Answer: Shakespeare wrote Hamlet.",
        )
        .unwrap();

        let answer = report
            .claims
            .iter()
            .find(|claim| claim.source_role == ClaimRole::Answer)
            .unwrap();

        assert_eq!(answer.evidence.status, ClaimEvidenceStatus::Unresolved);
        assert!(report.context_available);
    }

    #[test]
    fn rhetorical_marker_with_named_entity_stays_factual() {
        assert_eq!(
            classify_claim_type("Harvard University is arguably the best university."),
            ClaimType::Factual
        );
    }

    #[test]
    fn rhetorical_marker_without_named_entity_is_rhetorical() {
        assert_eq!(
            classify_claim_type("arguably the best option available."),
            ClaimType::Rhetorical
        );
    }

    #[test]
    fn canonical_triple_carries_confidence_and_provenance() {
        let report = annotate_claims(
            "Knowledge: The capital of Australia is Canberra.\n\
             Answer: Canberra is the capital of Australia.",
        )
        .unwrap();

        let answer = report
            .claims
            .iter()
            .find(|claim| claim.source_role == ClaimRole::Answer)
            .unwrap();
        let triple = annotation_to_triple(answer);

        assert_eq!(triple.provenance.claim_id, answer.claim_id);
        assert_eq!(triple.provenance.source_role, ClaimRole::Answer);
        assert!(triple.extraction_confidence.value() >= 0.5);
        assert!(triple.supports_contradiction());
    }

    #[test]
    fn low_confidence_triples_do_not_contradict() {
        let weak_a = ClaimTriple {
            subject: "edison".to_string(),
            predicate: "invent".to_string(),
            object: "telephone".to_string(),
            polarity: ClaimPolarity::Affirmative,
            extraction_confidence: Confidence::new(0.20),
            contradiction_threshold: Confidence::new(0.50),
            triple_eligible: true,
            provenance: ClaimTripleProvenance {
                claim_id: "raw:0".to_string(),
                source_role: ClaimRole::Raw,
                evidence_status: ClaimEvidenceStatus::MissingContext,
                matched_context: None,
            },
        };
        let weak_b = ClaimTriple {
            polarity: ClaimPolarity::Negative,
            provenance: ClaimTripleProvenance {
                claim_id: "raw:1".to_string(),
                source_role: ClaimRole::Raw,
                evidence_status: ClaimEvidenceStatus::MissingContext,
                matched_context: None,
            },
            ..weak_a.clone()
        };

        assert!(!weak_a.supports_contradiction());
        assert!(!weak_a.contradicts(&weak_b));
    }

    #[test]
    fn negative_triples_strip_embedded_negation_markers() {
        let report =
            annotate_claims("The Scorch Trials was not written by James Dashner.").unwrap();
        let triple = annotation_to_triple(&report.claims[0]);

        assert_eq!(triple.object, "written by james dashner");
    }
}
