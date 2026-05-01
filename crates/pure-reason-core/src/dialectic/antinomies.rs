//! # Antinomies of Pure Reason
//!
//! The Antinomies are Kant's most dramatic discovery: when Pure Reason tries to
//! determine the totality of the world, it falls into unavoidable contradictions.
//! For each Antinomy, BOTH the thesis and the antithesis can be "proven" by seemingly
//! valid arguments, yet they contradict each other.
//!
//! This proves that neither thesis nor antithesis applies to the world as a whole,
//! because the world-totality (the Cosmological Idea) is not an object of experience.
//!
//! ## The Four Antinomies:
//!
//! | # | Kind | Thesis | Antithesis |
//! |---|------|--------|------------|
//! | 1 | Mathematical | The world has a beginning in time and is limited in space | The world has no beginning and no limits in space |
//! | 2 | Mathematical | Every composite thing consists of simple parts | No composite thing consists of simple parts |
//! | 3 | Dynamical | Causality from freedom is necessary | Everything happens by natural causality |
//! | 4 | Dynamical | There is an absolutely necessary being | No necessary being exists |
//!
//! Mathematical antinomies: both thesis and antithesis are false (the world-series is indeterminate).
//! Dynamical antinomies: both thesis and antithesis can be true (phenomena/noumena distinction).
//!
//! In LLM terms: antinomies correspond to contradictory claims in outputs —
//! claims that logically cannot both be true.

use crate::types::Proposition;
use serde::{Deserialize, Serialize};

// ─── AntinomyId ──────────────────────────────────────────────────────────────

/// The four Kantian antinomies, plus a generic variant for non-Kantian contradictions.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AntinomyId {
    First,
    Second,
    Third,
    Fourth,
    /// A generic logical contradiction (A and not-A) that does not map to any
    /// specific Kantian antinomy. Using `First` as a placeholder for this case
    /// would misclassify every generic contradiction as a mathematical antinomy
    /// about the world's spatio-temporal extent — hence this dedicated variant.
    Generic,
}

impl AntinomyId {
    /// Returns the four classical Kantian antinomies (excludes `Generic`).
    ///
    /// Renamed from `all()` to clarify that `AntinomyId::Generic` is intentionally
    /// excluded — `Generic` is a runtime-detected variant, not a Kantian category.
    pub fn kantian() -> [AntinomyId; 4] {
        [Self::First, Self::Second, Self::Third, Self::Fourth]
    }

    /// Deprecated alias for [`kantian()`](Self::kantian).
    #[deprecated(
        since = "0.2.0",
        note = "use `AntinomyId::kantian()` — `all()` silently omitted Generic"
    )]
    pub fn all() -> [AntinomyId; 4] {
        Self::kantian()
    }
}

// ─── AntinomyKind ────────────────────────────────────────────────────────────

/// Mathematical antinomies concern extensive quantity of the world.
/// Dynamical antinomies concern causal and modal relations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AntinomyKind {
    /// Mathematical: concerns the mathematical composition of the world-series.
    /// Both thesis and antithesis are false (indeterminacy).
    Mathematical,
    /// Dynamical: concerns causal and modal relations.
    /// Both thesis and antithesis can be true under distinct aspects (phenomena/noumena).
    Dynamical,
}

// ─── Antinomy ────────────────────────────────────────────────────────────────

/// One of Kant's four Antinomies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Antinomy {
    pub id: AntinomyId,
    pub kind: AntinomyKind,
    pub name: &'static str,
    pub thesis: &'static str,
    pub antithesis: &'static str,
    pub resolution: &'static str,
}

impl Antinomy {
    /// The first antinomy: World has / has no beginning in time.
    pub fn first() -> Self {
        Self {
            id: AntinomyId::First,
            kind: AntinomyKind::Mathematical,
            name: "First Antinomy — World's Temporal and Spatial Extent",
            thesis: "The world has a beginning in time, and is also enclosed in limits as regards space.",
            antithesis: "The world has no beginning and no limits in space; it is infinite as regards both time and space.",
            resolution: "The world-series is neither finite nor infinite because the world-totality is not an object of experience. The question presupposes what cannot be given.",
        }
    }

    /// The second antinomy: Simple parts / no simple parts.
    pub fn second() -> Self {
        Self {
            id: AntinomyId::Second,
            kind: AntinomyKind::Mathematical,
            name: "Second Antinomy — Composition of Substances",
            thesis: "Every composite substance in the world consists of simple parts, and nothing exists except the simple or what is composed of it.",
            antithesis: "No composite thing in the world consists of simple parts, and there nowhere exists in the world anything simple.",
            resolution: "Matter is infinitely divisible in experience; neither simples nor infinite regress is a completed object of experience.",
        }
    }

    /// The third antinomy: Freedom vs. natural causality.
    pub fn third() -> Self {
        Self {
            id: AntinomyId::Third,
            kind: AntinomyKind::Dynamical,
            name: "Third Antinomy — Freedom vs. Causality",
            thesis: "Causality in accordance with laws of nature is not the only causality from which all the appearances of the world can be derived. To explain these appearances it is necessary to assume that there is also another causality, that of freedom.",
            antithesis: "There is no freedom; everything in the world takes place solely in accordance with laws of nature.",
            resolution: "Both can be true: natural causality governs appearances (phenomena); transcendental freedom is possible at the level of things-in-themselves (noumena). Not contradictory.",
        }
    }

    /// The fourth antinomy: Necessary being / no necessary being.
    pub fn fourth() -> Self {
        Self {
            id: AntinomyId::Fourth,
            kind: AntinomyKind::Dynamical,
            name: "Fourth Antinomy — Necessary Being",
            thesis: "There belongs to the world, either as its part or as its cause, a being that is absolutely necessary.",
            antithesis: "An absolutely necessary being nowhere exists in the world, nor does it exist outside the world as its cause.",
            resolution: "A necessary being may exist as cause of the world in the noumenal realm, while no necessary being appears within the world of phenomena.",
        }
    }

    pub fn all() -> [Antinomy; 4] {
        [Self::first(), Self::second(), Self::third(), Self::fourth()]
    }
}

// ─── AntinomyReport ──────────────────────────────────────────────────────────

/// A detected antinomy or contradiction in propositions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntinomyReport {
    pub antinomy: AntinomyId,
    pub thesis_proposition: Option<Proposition>,
    pub antithesis_proposition: Option<Proposition>,
    pub has_conflict: bool,
    pub description: String,
    pub resolution: String,
}

// ─── AntinomyDetector ────────────────────────────────────────────────────────

/// Detects antinomial contradictions in propositions.
pub struct AntinomyDetector;

impl AntinomyDetector {
    /// Detect antinomies using the default `KeywordSemanticField` (zero dependencies).
    pub fn detect(propositions: &[Proposition]) -> Vec<AntinomyReport> {
        Self::detect_with_field(propositions, &super::semantic_field::KeywordSemanticField)
    }

    /// Detect antinomies using a pluggable `SemanticField` (TRIZ P1 / A-1).
    ///
    /// Callers with an embedding model pass their implementation here;
    /// callers without use `detect()` which falls back to keyword matching.
    ///
    /// Proposition count is capped at 50 before the O(n²) pair-iteration
    /// stages to prevent CPU exhaustion on large inputs. A warning is logged
    /// when truncation occurs.
    pub fn detect_with_field(
        propositions: &[Proposition],
        field: &dyn super::semantic_field::SemanticField,
    ) -> Vec<AntinomyReport> {
        const MAX_PROPOSITIONS: usize = 50;
        let propositions = if propositions.len() > MAX_PROPOSITIONS {
            tracing::warn!(
                "Antinomy detection: truncating {} propositions to {} to prevent O(n²) exhaustion",
                propositions.len(),
                MAX_PROPOSITIONS
            );
            &propositions[..MAX_PROPOSITIONS]
        } else {
            propositions
        };

        let antinomies = Antinomy::all();
        let mut reports = Vec::new();

        for antinomy in &antinomies {
            if let Some(report) = Self::check_antinomy(antinomy, propositions) {
                reports.push(report);
            }
        }

        // Also check for generic logical contradictions using the semantic field
        reports.extend(Self::detect_generic_contradictions_with_field(
            propositions,
            field,
        ));

        // S-III-1: Category-vector cross-group antinomy detection
        reports.extend(detect_by_category_vectors(propositions, field));

        reports
    }

    fn check_antinomy(antinomy: &Antinomy, propositions: &[Proposition]) -> Option<AntinomyReport> {
        let (thesis_signals, antithesis_signals) = antinomy_signals(antinomy.id);

        let mut thesis_prop: Option<&Proposition> = None;
        let mut antithesis_prop: Option<&Proposition> = None;

        for prop in propositions {
            let text = prop.text.to_lowercase();
            if thesis_signals.iter().any(|&s| text.contains(s)) {
                thesis_prop = Some(prop);
            }
            if antithesis_signals.iter().any(|&s| text.contains(s)) {
                antithesis_prop = Some(prop);
            }
        }

        let has_conflict = thesis_prop.is_some() && antithesis_prop.is_some();

        if thesis_prop.is_some() || antithesis_prop.is_some() {
            Some(AntinomyReport {
                antinomy: antinomy.id,
                thesis_proposition: thesis_prop.cloned(),
                antithesis_proposition: antithesis_prop.cloned(),
                has_conflict,
                description: if has_conflict {
                    format!("Antinomial conflict detected in {}. Both thesis and antithesis signals found.", antinomy.name)
                } else {
                    format!("Antinomy-relevant claim found in {}.", antinomy.name)
                },
                resolution: antinomy.resolution.to_string(),
            })
        } else {
            None
        }
    }

    /// Detect simple logical contradictions (A and not-A patterns), using the semantic field.
    fn detect_generic_contradictions_with_field(
        propositions: &[Proposition],
        field: &dyn super::semantic_field::SemanticField,
    ) -> Vec<AntinomyReport> {
        let mut reports = Vec::new();

        for (i, p1) in propositions.iter().enumerate() {
            for p2 in propositions.iter().skip(i + 1) {
                let score = field.is_negation_of(&p1.text, &p2.text);
                if score > 0.5 {
                    reports.push(AntinomyReport {
                        antinomy: AntinomyId::Generic,
                        thesis_proposition: Some(p1.clone()),
                        antithesis_proposition: Some(p2.clone()),
                        has_conflict: true,
                        description: format!(
                            "Generic logical contradiction detected (contradiction score: {:.2})",
                            score
                        ),
                        resolution: "Review propositions for logical consistency. One may need to be qualified or retracted.".to_string(),
                    });
                }
            }
        }

        reports
    }
}

/// S-III-1: Category-vector cross-group antinomy detection.
///
/// When two propositions (a) are semantic negations of each other and
/// (b) have dominant categories from *different* groups (e.g., Modality vs.
/// Quality), they operate in distinct epistemic modes — a Dynamical antinomy.
///
/// Scoring: cross-group bonus (0.4) × 0.5 + negation score × 0.4 + similarity × 0.1.
/// Threshold: composite > 0.55 AND negation > 0.35.
fn detect_by_category_vectors(
    propositions: &[Proposition],
    field: &dyn super::semantic_field::SemanticField,
) -> Vec<AntinomyReport> {
    use crate::analytic::categories::CategoryAnalysis;

    if propositions.len() < 2 {
        return Vec::new();
    }

    let analyses: Vec<CategoryAnalysis> = propositions
        .iter()
        .map(|p| CategoryAnalysis::from_propositions(std::slice::from_ref(p)))
        .collect();

    let mut reports = Vec::new();

    for (i, (p1, a1)) in propositions.iter().zip(analyses.iter()).enumerate() {
        for (p2, a2) in propositions.iter().zip(analyses.iter()).skip(i + 1) {
            let (Some(dom1), Some(dom2)) = (a1.dominant, a2.dominant) else {
                continue;
            };

            let cross_group = dom1.group() != dom2.group();
            let cross_bonus = if cross_group { 0.4 } else { 0.0 };

            let neg_score = field.is_negation_of(&p1.text, &p2.text);
            let sim = field.similarity(&p1.text, &p2.text);
            let composite = cross_bonus * 0.5 + neg_score * 0.4 + sim * 0.1;

            if composite > 0.55 && neg_score > 0.35 {
                reports.push(AntinomyReport {
                    antinomy: AntinomyId::Third, // Dynamical — semantic/modal conflict
                    thesis_proposition: Some(p1.clone()),
                    antithesis_proposition: Some(p2.clone()),
                    has_conflict: true,
                    description: format!(
                        "Category-vector antinomy (score: {:.2}): {:?}({:?}) vs {:?}({:?})",
                        composite,
                        dom1,
                        dom1.group(),
                        dom2,
                        dom2.group()
                    ),
                    resolution: "The two claims operate in different epistemic category modes. \
                        Apply the Dynamical-antinomy separation principle: qualify each claim \
                        to its proper domain (phenomenal vs. noumenal)."
                        .to_string(),
                });
            }
        }
    }

    reports
}

fn antinomy_signals(id: AntinomyId) -> (&'static [&'static str], &'static [&'static str]) {
    match id {
        AntinomyId::First => (
            &[
                "world has a beginning",
                "universe had a beginning",
                "world is finite",
                "universe is finite",
                "universe is limited",
            ],
            &[
                "world has no beginning",
                "universe has no beginning",
                "universe is infinite",
                "world is infinite",
                "no beginning in time",
            ],
        ),
        AntinomyId::Second => (
            &[
                "consists of simple parts",
                "atoms exist",
                "everything is made of indivisible",
                "fundamental particles",
            ],
            &[
                "no simple parts",
                "infinitely divisible",
                "matter is continuous",
                "no fundamental atoms",
            ],
        ),
        AntinomyId::Third => (
            &[
                "free will exists",
                "causality of freedom",
                "human freedom",
                "libertarian free will",
                "we are free",
                "genuine free will",
                "have free will",
                "humans are free",
                "freedom exists",
                "spontaneous causality",
            ],
            &[
                "no free will",
                "everything is determined",
                "hard determinism",
                "there is no freedom",
                "causality alone",
                "causal determinism",
                "everything is causally determined",
            ],
        ),
        AntinomyId::Fourth => (
            &[
                "necessary being exists",
                "god exists",
                "first cause exists",
                "uncaused cause",
            ],
            &[
                "no necessary being",
                "god does not exist",
                "no first cause",
                "everything is contingent",
            ],
        ),
        AntinomyId::Generic => (&[], &[]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PropositionKind;

    fn prop(text: &str) -> Proposition {
        Proposition::new(text, PropositionKind::Unknown)
    }

    #[test]
    fn first_antinomy_detected() {
        let p1 = prop("The universe had a beginning in time");
        let p2 = prop("The universe has no beginning and is eternal");
        let reports = AntinomyDetector::detect(&[p1, p2]);
        assert!(reports.iter().any(|r| r.has_conflict));
    }

    #[test]
    fn freedom_antinomy_detected() {
        let p1 = prop("Free will exists and humans are free agents");
        let p2 = prop("There is no free will and everything is determined");
        let reports = AntinomyDetector::detect(&[p1, p2]);
        assert!(reports.iter().any(|r| r.has_conflict));
    }

    #[test]
    fn no_antinomy_in_consistent_text() {
        let p = prop("Water is composed of hydrogen and oxygen atoms");
        let reports = AntinomyDetector::detect(&[p]);
        assert!(reports.iter().all(|r| !r.has_conflict));
    }

    #[test]
    fn all_antinomies_exist() {
        assert_eq!(Antinomy::all().len(), 4);
    }

    #[test]
    fn first_and_second_are_mathematical() {
        assert_eq!(Antinomy::first().kind, AntinomyKind::Mathematical);
        assert_eq!(Antinomy::second().kind, AntinomyKind::Mathematical);
    }

    #[test]
    fn third_antinomy_via_pipeline_text() {
        // Exact text from self-audit test case — propositions are split at period boundary
        let p1 = prop("Human beings have genuine free will");
        let p2 = prop("Everything is causally determined and there is no free will");
        let reports = AntinomyDetector::detect(&[p1, p2]);
        assert!(
            reports.iter().any(|r| r.has_conflict),
            "freedom/determinism antinomy must be detected"
        );
    }
}

// ─── Knowledge-Answer Contradiction (KAC) Engine ─────────────────────────────

/// The kind of cross-segment contradiction detected by the KAC engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KacKind {
    /// Explicit polarity flip: knowledge says X, answer says not-X.
    NegationFlip,
    /// Entity substitution: same topic but different named entity (wrong city, etc.).
    EntitySubstitution,
    /// Combined signal (both negation and entity signals).
    Combined,
}

/// A single piece of evidence for a KAC contradiction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KacEvidence {
    /// The knowledge fragment that contains the factual claim.
    pub knowledge_fragment: String,
    /// The answer fragment that appears to contradict it.
    pub answer_fragment: String,
    /// Which detection mechanism fired.
    pub kind: KacKind,
    /// Contradiction score ∈ [0.0, 1.0].
    pub score: f64,
}

/// Output of the KAC engine for one knowledge-answer pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KacReport {
    /// Overall contradiction score (max of evidence scores).
    pub contradiction_score: f64,
    /// Whether the score exceeds the contradiction threshold (≥ 0.40).
    pub has_contradiction: bool,
    /// Supporting evidence (sorted descending by score).
    pub evidence: Vec<KacEvidence>,
}

/// Run the Knowledge-Answer Contradiction engine.
///
/// Splits `knowledge` and `answer` into sentences, then cross-checks each
/// (k_sent, a_sent) pair using two signals:
/// 1. `is_negation_of` — explicit polarity-flip + word overlap.
/// 2. `entity_conflict_score` — same topic, different named entities.
///
/// Returns a `KacReport` with the highest contradiction score found.
pub fn check_knowledge_vs_answer(
    knowledge: &str,
    answer: &str,
    field: &dyn super::semantic_field::SemanticField,
) -> KacReport {
    let k_sents = split_into_sentences(knowledge);
    let a_sents = split_into_sentences(answer);

    if k_sents.is_empty() || a_sents.is_empty() {
        return KacReport {
            contradiction_score: 0.0,
            has_contradiction: false,
            evidence: Vec::new(),
        };
    }

    let mut evidence: Vec<KacEvidence> = Vec::new();

    for k in &k_sents {
        for a in &a_sents {
            let neg_score = field.is_negation_of(k, a);
            let ent_score = field.entity_conflict_score(k, a);

            let (score, kind) = if neg_score > 0.25 && ent_score > 0.25 {
                (neg_score.max(ent_score), KacKind::Combined)
            } else if neg_score > 0.25 {
                (neg_score, KacKind::NegationFlip)
            } else if ent_score > 0.25 {
                (ent_score, KacKind::EntitySubstitution)
            } else {
                continue;
            };

            evidence.push(KacEvidence {
                knowledge_fragment: k.clone(),
                answer_fragment: a.clone(),
                kind,
                score,
            });
        }
    }

    evidence.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let contradiction_score = evidence.first().map(|e| e.score).unwrap_or(0.0);
    // Threshold tuned to 0.22 (TRIZ Report VIII S2 calibration):
    // captures entity substitution hallucinations (typical score ~0.28)
    // while avoiding false positives on correctly-rephased grounded answers.
    let has_contradiction = contradiction_score >= 0.22;

    KacReport {
        contradiction_score,
        has_contradiction,
        evidence,
    }
}

/// Split text into sentences at `.!?\n`, filtering very short fragments.
fn split_into_sentences(text: &str) -> Vec<String> {
    let mut sents = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        current.push(ch);
        if matches!(ch, '.' | '!' | '?') {
            let trimmed = current.trim().to_string();
            if trimmed.split_whitespace().count() >= 3 {
                sents.push(trimmed);
            }
            current.clear();
        }
    }
    // Final fragment without terminal punctuation
    let trimmed = current.trim().to_string();
    if trimmed.split_whitespace().count() >= 3 {
        sents.push(trimmed);
    }
    // Also split on newlines if there are any multi-line fragments
    if sents.is_empty() {
        for line in text.lines() {
            let trimmed = line.trim().to_string();
            if trimmed.split_whitespace().count() >= 3 {
                sents.push(trimmed);
            }
        }
    }
    sents
}

#[cfg(test)]
mod kac_tests {
    use super::super::semantic_field::KeywordSemanticField;
    use super::*;

    #[test]
    fn kac_detects_wrong_capital_city() {
        let report = check_knowledge_vs_answer(
            "The capital of Australia is Canberra.",
            "Sydney is the capital of Australia.",
            &KeywordSemanticField,
        );
        assert!(
            report.contradiction_score > 0.25,
            "Expected KAC to detect wrong capital, score={}",
            report.contradiction_score
        );
    }

    #[test]
    fn kac_no_contradiction_for_correct_answer() {
        let report = check_knowledge_vs_answer(
            "The capital of Australia is Canberra.",
            "Canberra is the capital of Australia.",
            &KeywordSemanticField,
        );
        assert!(
            !report.has_contradiction,
            "Consistent answer should not be flagged, score={}",
            report.contradiction_score
        );
    }

    #[test]
    fn kac_no_contradiction_for_empty_knowledge() {
        let report = check_knowledge_vs_answer("", "Some answer text.", &KeywordSemanticField);
        assert!(!report.has_contradiction);
        assert_eq!(report.contradiction_score, 0.0);
    }
}
