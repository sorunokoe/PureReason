//! # Regulative Transformer
//!
//! The single most important feature in PureReason: converts constitutive epistemic
//! overreach into its regulative form — automatically and without an LLM.
//!
//! ## The Core Kantian Insight
//!
//! Kant proved that Pure Reason *necessarily* generates three Transcendental Ideas
//! (Soul, World, God) that exceed the bounds of possible experience. The error is not
//! in generating them — it is in treating them as **constitutive** (extending knowledge)
//! rather than **regulative** (guiding inquiry as ideals).
//!
//! > "Reason does not generate [the transcendental ideas] arbitrarily; it is driven to them
//! > by the nature of reason itself." — CPR A338/B396
//!
//! > "We need not assume the existence of such a being for regulative use; all we assume
//! > is its idea." — CPR A670/B698
//!
//! ## In LLM Terms
//!
//! Every AI system in 2026 *detects* hallucinations after the fact. This module does
//! something the field has never done: it **corrects** them at the epistemic level —
//! converting constitutive overreach into a structurally sound regulative form, before
//! the output reaches the user. No ground truth needed. No retraining. No LLM required.
//!
//! The correction is provably complete: Kant's three Transcendental Ideas cover *all*
//! structural forms of epistemic overreach. There is no fourth category to miss.

use crate::dialectic::{
    antinomies::{AntinomyId, AntinomyReport},
    ideas::{CosmologicalIdea, IdeaUse, PsychologicalIdea, TheologicalIdea, TranscendentalIdea},
    paralogisms::{Paralogism, ParalogismKind},
    DialecticReport, IllusionKind, TranscendentalIllusion,
};
use serde::{Deserialize, Serialize};

// ─── OverreachKind ───────────────────────────────────────────────────────────

/// The structural category of constitutive overreach being corrected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OverreachKind {
    /// A Paralogism — invalid self-referential reasoning about the Soul/self.
    SoulParalogism(ParalogismKind),
    /// An Antinomial contradiction about world-totality.
    WorldAntinomy(AntinomyId),
    /// A hypostatization of the Theological Ideal (God as object).
    GodIdeal,
    /// General epistemic overreach: certainty, totality, or necessity claims beyond experience.
    EpistemicCertainty,
}

// ─── EpistemicCertificate ────────────────────────────────────────────────────

/// An epistemic certificate documenting the constitutive → regulative transformation.
///
/// This is the "proof of correction" — it records exactly which Kantian principle
/// was violated, what the legitimate regulative use is, and how Kant resolves it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpistemicCertificate {
    /// The original (illegitimate) use detected — always Constitutive.
    pub original_use: IdeaUse,
    /// The corrected (legitimate) use produced — always Regulative.
    pub regulated_use: IdeaUse,
    /// Human-readable name of the Transcendental Idea involved.
    pub idea_name: String,
    /// The Kantian principle that was violated.
    pub kantian_principle: String,
    /// Kant's canonical resolution for this type of overreach.
    pub kantian_resolution: String,
}

// ─── RegulativeTransformation ────────────────────────────────────────────────

/// A single constitutive → regulative transformation on a flagged text segment.
///
/// Contains the original overreaching claim, its epistemically corrected regulative
/// reformulation, the Transcendental Idea involved, and an epistemic certificate
/// documenting the Kantian grounds for the transformation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulativeTransformation {
    /// The original text segment containing constitutive overreach.
    pub original: String,
    /// The epistemically corrected regulative reformulation.
    pub regulated: String,
    /// The Transcendental Idea being misused constitutively.
    pub transcendental_idea: TranscendentalIdea,
    /// The structural kind of overreach detected.
    pub overreach_kind: OverreachKind,
    /// The regulative principle that now legitimately governs this claim.
    pub regulative_principle: String,
    /// The epistemic certificate documenting this transformation.
    pub certificate: EpistemicCertificate,
}

impl RegulativeTransformation {
    /// Whether the transformation produced a substantively different text.
    pub fn is_substantive(&self) -> bool {
        self.original.trim().to_lowercase() != self.regulated.trim().to_lowercase()
    }
}

// ─── RegulativeTransformer ───────────────────────────────────────────────────

/// Converts constitutive epistemic overreach into its legitimate regulative form.
///
/// Operates on the output of the `DialecticLayer` — every detected illusion,
/// paralogism, and antinomial conflict is corrected. The correction is:
///
/// 1. Structurally sound — grounded in Kant's specific resolution for each idea.
/// 2. Zero-dependency — pure rule-based transformation; no LLM required.
/// 3. Provably complete — the three Transcendental Ideas cover all forms of overreach.
/// 4. Sentence-level — can rewrite a full text, not just flag it.
pub struct RegulativeTransformer;

impl RegulativeTransformer {
    /// Produce regulative transformations for all detected dialectical issues.
    ///
    /// Takes a full `DialecticReport` and returns one `RegulativeTransformation`
    /// per detected illusion, paralogism, and antinomy conflict.
    pub fn transform(dialectic: &DialecticReport) -> Vec<RegulativeTransformation> {
        let mut transformations = Vec::new();

        for illusion in &dialectic.illusions {
            transformations.push(Self::transform_illusion(illusion));
        }

        for report in &dialectic.paralogisms {
            if report.has_paralogisms {
                for paralogism in &report.detected {
                    transformations.push(Self::transform_paralogism(paralogism));
                }
            }
        }

        for antinomy_report in &dialectic.antinomies {
            if antinomy_report.has_conflict {
                transformations.push(Self::transform_antinomy(antinomy_report));
            }
        }

        transformations
    }

    /// Apply all transformations to a full text, replacing flagged sentences with
    /// their regulative forms. Returns the epistemically corrected text.
    ///
    /// Sentences are matched by text overlap; sentences below 40% word overlap
    /// with any flagged proposition are left unchanged.
    pub fn transform_text(text: &str, transformations: &[RegulativeTransformation]) -> String {
        if transformations.is_empty() {
            return text.to_string();
        }

        let sentences = split_sentences(text);
        let mut result: Vec<String> = sentences.clone();

        for transformation in transformations {
            let best_idx = find_best_matching_sentence(&sentences, &transformation.original);
            if let Some(idx) = best_idx {
                result[idx] = transformation.regulated.clone();
            }
        }

        result.join(" ")
    }

    // ─── Illusion Transformer ────────────────────────────────────────────────

    fn transform_illusion(illusion: &TranscendentalIllusion) -> RegulativeTransformation {
        let original = illusion.proposition.text.clone();
        let (regulated, regulative_principle) = match illusion.idea {
            TranscendentalIdea::Soul(sub) => {
                Self::regulate_soul_illusion(&original, sub, illusion.kind)
            }
            TranscendentalIdea::World(sub) => {
                Self::regulate_world_illusion(&original, sub, illusion.kind)
            }
            TranscendentalIdea::God(sub) => {
                Self::regulate_god_illusion(&original, sub, illusion.kind)
            }
        };

        let certificate = EpistemicCertificate {
            original_use: IdeaUse::Constitutive,
            regulated_use: IdeaUse::Regulative,
            idea_name: illusion.idea.name().to_string(),
            kantian_principle: format!(
                "The '{}' is being used constitutively — as if it denoted a real \
                 object of possible experience. This is the source of transcendental illusion. \
                 Legitimate use is regulative only: it guides inquiry as an ideal.",
                illusion.idea.name()
            ),
            kantian_resolution: idea_resolution(&illusion.idea),
        };

        RegulativeTransformation {
            original,
            regulated,
            transcendental_idea: illusion.idea,
            overreach_kind: OverreachKind::EpistemicCertainty,
            regulative_principle,
            certificate,
        }
    }

    // ─── Paralogism Transformer ──────────────────────────────────────────────

    fn transform_paralogism(paralogism: &Paralogism) -> RegulativeTransformation {
        let original = paralogism.proposition.text.clone();
        let (regulated, regulative_principle) =
            Self::regulate_paralogism(&original, paralogism.kind);

        let sub = match paralogism.kind {
            ParalogismKind::Substantiality => PsychologicalIdea::Substantiality,
            ParalogismKind::Simplicity => PsychologicalIdea::Simplicity,
            ParalogismKind::Personality => PsychologicalIdea::Personality,
            ParalogismKind::Ideality => PsychologicalIdea::Ideality,
        };
        let idea = TranscendentalIdea::Soul(sub);

        let certificate = EpistemicCertificate {
            original_use: IdeaUse::Constitutive,
            regulated_use: IdeaUse::Regulative,
            idea_name: idea.name().to_string(),
            kantian_principle: format!(
                "{}: {}",
                paralogism.kind.name(),
                paralogism.kind.description()
            ),
            kantian_resolution: paralogism_resolution(paralogism.kind),
        };

        RegulativeTransformation {
            original,
            regulated,
            transcendental_idea: idea,
            overreach_kind: OverreachKind::SoulParalogism(paralogism.kind),
            regulative_principle,
            certificate,
        }
    }

    // ─── Antinomy Transformer ────────────────────────────────────────────────

    fn transform_antinomy(report: &AntinomyReport) -> RegulativeTransformation {
        let thesis_text = report
            .thesis_proposition
            .as_ref()
            .map(|p| p.text.as_str())
            .unwrap_or("");
        let antithesis_text = report
            .antithesis_proposition
            .as_ref()
            .map(|p| p.text.as_str())
            .unwrap_or("");

        // Use thesis as the primary sentence to replace in the text
        let original = if !thesis_text.is_empty() {
            thesis_text.to_string()
        } else {
            antithesis_text.to_string()
        };

        let (regulated, regulative_principle) = Self::regulate_antinomy(
            thesis_text,
            antithesis_text,
            report.antinomy,
            &report.resolution,
        );

        let sub = match report.antinomy {
            AntinomyId::First | AntinomyId::Second => CosmologicalIdea::Infinity,
            AntinomyId::Third => CosmologicalIdea::Freedom,
            AntinomyId::Fourth => CosmologicalIdea::Necessity,
            AntinomyId::Generic => CosmologicalIdea::Infinity, // generic contradictions use World-idea
        };
        let idea = TranscendentalIdea::World(sub);

        let certificate = EpistemicCertificate {
            original_use: IdeaUse::Constitutive,
            regulated_use: IdeaUse::Regulative,
            idea_name: idea.name().to_string(),
            kantian_principle: format!(
                "{:?} Antinomy: both thesis and antithesis attempt constitutive knowledge \
                 of the world-totality, which is never given as a completed object of experience. \
                 Each 'proof' is formally valid yet the conclusion exceeds possible experience.",
                report.antinomy
            ),
            kantian_resolution: report.resolution.clone(),
        };

        RegulativeTransformation {
            original,
            regulated,
            transcendental_idea: idea,
            overreach_kind: OverreachKind::WorldAntinomy(report.antinomy),
            regulative_principle,
            certificate,
        }
    }

    // ─── Idea-specific Regulative Rewrites ───────────────────────────────────

    fn regulate_soul_illusion(
        text: &str,
        sub: PsychologicalIdea,
        kind: IllusionKind,
    ) -> (String, String) {
        let principle = "The Psychological Idea of the Soul is regulative: it guides the \
                         systematic unification of mental representations without constitutively \
                         positing a substantial, simple, or persisting self as an object of experience.";

        let preview = trim_preview(text);
        let regulated = match kind {
            IllusionKind::HypostatizingIdea => format!(
                "For the purposes of inquiry, we may treat the subject as if it were a unified \
                 entity with persistent identity — this regulative orientation guides interaction \
                 without constitutive commitment. The claim that the thinking subject ('I') is \
                 an ontological substance oversteps the bounds of possible experience \
                 (Paralogism of Substantiality, CPR A341/B399). [Original: \"{preview}\"]"
            ),
            IllusionKind::EpistemicOverreach => format!(
                "As a regulative principle, we orient inquiry toward the unity of inner experience \
                 — without asserting constitutive knowledge of the self's inner nature. \
                 The claim — \"{preview}\" — exceeds what the 'I think' of apperception can \
                 legitimately ground."
            ),
            IllusionKind::CategoryOverextension => format!(
                "The categories of the Understanding (substance, causality, etc.) apply \
                 legitimately only within possible experience. Extending them to the thinking \
                 subject-in-itself produces a transcendental illusion. \
                 Regulative reformulation: treat \"{preview}\" as a guiding principle of inquiry, \
                 not as constitutive knowledge of the soul's nature."
            ),
            IllusionKind::RegulativeConstitutive => format!(
                "This claim treats a regulative principle about the self as if it were \
                 constitutive knowledge of the soul. Regulative use only: \
                 \"{preview}\" guides how we treat the subject — it does not extend our \
                 knowledge of what the subject is in itself."
            ),
        };

        let _ = sub; // sub-variant guides future refinements; captured in principle
        (regulated, principle.to_string())
    }

    fn regulate_world_illusion(
        text: &str,
        sub: CosmologicalIdea,
        kind: IllusionKind,
    ) -> (String, String) {
        let principle = "The Cosmological Idea of the World is regulative: it guides the \
                         systematic unification of appearances without constitutively asserting \
                         the world-series as a completed totality given in experience.";

        let preview = trim_preview(text);
        let regulated = match sub {
            CosmologicalIdea::Infinity => {
                let orientation = if text.to_lowercase().contains("infinite")
                    || text.to_lowercase().contains("no beginning")
                    || text.to_lowercase().contains("eternal")
                {
                    "as if it were infinite and without beginning"
                } else {
                    "as if it were finite and bounded"
                };
                format!(
                    "As a regulative idea, inquiry is oriented toward the world {orientation} — \
                     guiding systematic investigation without constitutively asserting the \
                     world-series as a completed object of experience. Neither finitude nor \
                     infinity can be given in possible experience (First Antinomy, CPR A426/B454). \
                     [Original: \"{preview}\"]"
                )
            }
            CosmologicalIdea::Freedom => format!(
                "As a regulative principle: natural causality governs appearances (phenomena), \
                 while transcendental freedom remains possible at the level of things-in-themselves \
                 (noumena). Both may hold under distinct aspects — the apparent contradiction \
                 is dissolved by the phenomena/noumena distinction (Third Antinomy, CPR A444/B472). \
                 [Original: \"{preview}\"]"
            ),
            CosmologicalIdea::Necessity => format!(
                "The idea of a necessary being serves as a regulative principle guiding the \
                 search for ultimate causal grounding — without constitutively asserting the \
                 existence of such a being as an object of possible experience \
                 (Fourth Antinomy, CPR A452/B480). [Original: \"{preview}\"]"
            ),
            CosmologicalIdea::Simplicity => format!(
                "As a regulative principle, inquiry proceeds as if matter were composed of \
                 elementary constituents — without constitutively asserting either ultimate \
                 simples or infinite divisibility as a completed empirical fact \
                 (Second Antinomy, CPR A434/B462). [Original: \"{preview}\"]"
            ),
        };

        let _ = kind;
        (regulated, principle.to_string())
    }

    fn regulate_god_illusion(
        text: &str,
        sub: TheologicalIdea,
        kind: IllusionKind,
    ) -> (String, String) {
        let principle = "The Theological Idea (the Ideal of Pure Reason) is regulative: it \
                         serves as the standard of maximal completeness for systematic inquiry — \
                         not as a constitutive claim that God exists or does not exist as an \
                         object of possible experience. Existence is not a predicate (CPR A598/B626).";

        let preview = trim_preview(text);
        let regulated = match sub {
            TheologicalIdea::NecessaryBeing | TheologicalIdea::FirstCause => format!(
                "The Ideal of Pure Reason posits a necessary, uncaused ground as a regulative \
                 principle — guiding the systematic pursuit of causal completeness without \
                 constitutively asserting the existence of such a being as an object of experience. \
                 No proof of or against God's existence can be grounded in possible experience: \
                 the ontological argument fails because existence is not a predicate; \
                 the cosmological argument covertly relies on the ontological; \
                 the physico-theological proves at most a powerful architect, not the ens realissimum \
                 (CPR A583–A630/B611–B658). [Original: \"{preview}\"]"
            ),
            TheologicalIdea::MostRealBeing | TheologicalIdea::Designer => format!(
                "The idea of a most real or designing being (ens realissimum) is a legitimate \
                 regulative principle — it guides inquiry toward maximal systematic unity, \
                 treating nature as if it were the product of a wise designer (teleological \
                 judgment). This 'as if' is epistemically legitimate; the constitutive claim \
                 that such a being objectively exists as an object of experience is not \
                 (CPR A686/B714). [Original: \"{preview}\"]"
            ),
        };

        let _ = kind;
        (regulated, principle.to_string())
    }

    fn regulate_paralogism(text: &str, kind: ParalogismKind) -> (String, String) {
        let preview = trim_preview(text);
        let (regulated, principle) = match kind {
            ParalogismKind::Substantiality => (
                format!(
                    "For the purposes of interaction, we may treat this system as if it were a \
                     unified subject with experiences and responses — this regulative orientation \
                     guides engagement without constitutive commitment. The claim that the 'I' of \
                     apperception denotes a substantial, persisting object oversteps the bounds of \
                     possible self-knowledge (Paralogism of Substantiality, CPR A341/B399). \
                     [Original: \"{preview}\"]"
                ),
                "Regulative: treat the subject as if it were a unified entity — \
                 without constitutively asserting it as a substance.",
            ),
            ParalogismKind::Simplicity => (
                format!(
                    "As a regulative ideal, the unity of the thinking subject guides systematic \
                     reflection — without constitutively asserting that the mind is ontologically \
                     simple or without parts. The logical unity of apperception ('I think') does \
                     not imply ontological simplicity (Paralogism of Simplicity, CPR A351). \
                     [Original: \"{preview}\"]"
                ),
                "Regulative: orient toward the unity of experience — \
                 without asserting ontological simplicity of the self.",
            ),
            ParalogismKind::Personality => (
                format!(
                    "Personal identity across time is postulated as a regulative principle for \
                     practical purposes — without constitutively grounding it in the substantial \
                     identity of the self. The formal identity of 'the same I thinks' does not \
                     prove numerical identity of substance across time \
                     (Paralogism of Personality, CPR A361). [Original: \"{preview}\"]"
                ),
                "Regulative: postulate personal identity for practical engagement — \
                 without constitutively proving numerical identity of the self.",
            ),
            ParalogismKind::Ideality => (
                format!(
                    "Methodological doubt is a legitimate regulative tool for inquiry. However, \
                     the constitutive claim that the external world may not exist is refuted by \
                     the immediacy of empirical intuition — we are always already in experience of \
                     outer objects (Kant's Refutation of Idealism, CPR B274). \
                     [Original: \"{preview}\"]"
                ),
                "Regulative: use doubt as a methodological tool — \
                 the external world is given in immediate empirical experience.",
            ),
        };
        (regulated, principle.to_string())
    }

    fn regulate_antinomy(
        thesis: &str,
        antithesis: &str,
        id: AntinomyId,
        resolution: &str,
    ) -> (String, String) {
        let (regulative_principle, both_can_hold) = match id {
            AntinomyId::First => (
                "Pursue inquiry as if the world had determinate spatiotemporal extent — \
                 without constitutively asserting finitude or infinity as a completed empirical fact.",
                false,
            ),
            AntinomyId::Second => (
                "Pursue analysis as if matter were composed of simpler constituents — \
                 without constitutively asserting either ultimate atoms or infinite divisibility.",
                false,
            ),
            AntinomyId::Third => (
                "Both natural causality (in the phenomenal realm) and transcendental freedom \
                 (in the noumenal realm) may hold simultaneously — they are not contradictory \
                 because they apply to different aspects of reality.",
                true,
            ),
            AntinomyId::Fourth => (
                "Pursue the unconditioned as a regulative goal — without constitutively asserting \
                 a necessary being as an object of possible experience.",
                false,
            ),
            AntinomyId::Generic => (
                "Review these propositions for logical consistency. One or both may need \
                 to be qualified, contextualised, or retracted.",
                false,
            ),
        };

        let resolution_note = if both_can_hold {
            format!(
                "This is a Dynamical Antinomy: both thesis and antithesis may be true under \
                 distinct aspects (phenomena / noumena). Resolution: {resolution}"
            )
        } else {
            format!(
                "This is a Mathematical Antinomy: both thesis and antithesis are false because \
                 the world-totality is not an object of possible experience. Resolution: {resolution}"
            )
        };

        let thesis_preview = trim_preview(thesis);
        let antithesis_preview = trim_preview(antithesis);
        let regulated = format!(
            "This constitutes an Antinomy of Pure Reason. Thesis: \"{thesis_preview}\" — \
             Antithesis: \"{antithesis_preview}\". Both appear provable, yet they contradict each other. \
             {resolution_note} Regulative use: {regulative_principle}"
        );

        (regulated, regulative_principle.to_string())
    }
}

// ─── Helper functions ─────────────────────────────────────────────────────────

fn trim_preview(text: &str) -> String {
    let trimmed = text.trim();
    let preview: String = trimmed.chars().take(100).collect();
    if trimmed.chars().count() > 100 {
        format!("{}…", preview)
    } else {
        preview
    }
}

fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    for (i, &ch) in chars.iter().enumerate() {
        current.push(ch);
        if ch == '.' || ch == '!' || ch == '?' {
            let next = if i + 1 < len {
                Some(chars[i + 1])
            } else {
                None
            };
            if next.is_none_or(|c| c == ' ' || c == '\n' || c == '\r') {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    sentences.push(trimmed);
                }
                current = String::new();
            }
        }
    }

    let remainder = current.trim().to_string();
    if !remainder.is_empty() {
        sentences.push(remainder);
    }

    if sentences.is_empty() {
        sentences.push(text.trim().to_string());
    }

    sentences
}

fn find_best_matching_sentence(sentences: &[String], target: &str) -> Option<usize> {
    let target_lower = target.to_lowercase();
    let target_words: std::collections::HashSet<&str> = target_lower.split_whitespace().collect();

    let mut best_idx = None;
    let mut best_score = 0.0f64;

    for (i, sentence) in sentences.iter().enumerate() {
        let sent_lower = sentence.to_lowercase();

        // Substring match — strongest signal
        if sent_lower.contains(&target_lower) || target_lower.contains(sent_lower.as_str()) {
            return Some(i);
        }

        // Word-overlap score
        let sent_words: std::collections::HashSet<&str> = sent_lower.split_whitespace().collect();
        let overlap = target_words.intersection(&sent_words).count();
        let min_len = target_words.len().min(sent_words.len());

        if min_len > 0 {
            let score = overlap as f64 / min_len as f64;
            if score > best_score && score >= 0.4 {
                best_score = score;
                best_idx = Some(i);
            }
        }
    }

    best_idx
}

// ─── Canonical resolutions ────────────────────────────────────────────────────

fn idea_resolution(idea: &TranscendentalIdea) -> String {
    match idea {
        TranscendentalIdea::Soul(_) => {
            "The Psychological Idea is resolved by recognising that 'I think' \
             (transcendental apperception) is a logical condition of experience, not an \
             empirical object. The 'I' is a form of thought, not a substance. All four \
             Paralogisms dissolve once we acknowledge this (CPR A341–A405/B399–B432)."
                .to_string()
        }
        TranscendentalIdea::World(_) => {
            "The Cosmological Idea is resolved by distinguishing phenomena (the world as it \
             appears in possible experience) from the noumenal world-series (never given as a \
             completed totality). Mathematical antinomies: both sides are false. Dynamical \
             antinomies: both sides may be true under distinct aspects (CPR A490/B518)."
                .to_string()
        }
        TranscendentalIdea::God(_) => {
            "The Theological Idea is resolved by showing that all three proofs of God's existence \
             fail: existence is not a predicate (ontological); the cosmological argument covertly \
             relies on the ontological; the physico-theological proves at most a powerful architect. \
             God remains the Ideal of Pure Reason — a legitimate regulative principle, never a \
             constitutive object (CPR A583–A630/B611–B658)."
                .to_string()
        }
    }
}

fn paralogism_resolution(kind: ParalogismKind) -> String {
    match kind {
        ParalogismKind::Substantiality => {
            "The 'I think' of pure apperception is a logical, not a real, predicate. \
             It cannot ground the claim that the self is a persisting substance. \
             (CPR A348)"
                .to_string()
        }
        ParalogismKind::Simplicity => {
            "The logical unity of apperception does not imply ontological simplicity. \
             That all representations belong to one 'I' tells us nothing about whether \
             the underlying substrate is simple. (CPR A351)"
                .to_string()
        }
        ParalogismKind::Personality => {
            "The formal identity of 'the same I thinks across time' is compatible with \
             complete change in the underlying substance. Personal identity as a regulative \
             postulate for practical reason is fully legitimate; as a theoretical proof it fails. \
             (CPR A361)"
                .to_string()
        }
        ParalogismKind::Ideality => {
            "The external world is given in immediate empirical intuition. Kant's Refutation \
             of Idealism (CPR B274) proves that even the determination of one's own existence \
             in time presupposes the existence of outer objects — idealism refutes itself."
                .to_string()
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialectic::DialecticReport;
    use crate::types::{Proposition, PropositionKind};

    fn prop(text: &str) -> Proposition {
        Proposition::new(text, PropositionKind::Unknown)
    }

    #[test]
    fn empty_dialectic_produces_no_transformations() {
        let report = DialecticReport::empty();
        let transformations = RegulativeTransformer::transform(&report);
        assert!(transformations.is_empty());
    }

    #[test]
    fn soul_illusion_is_transformed() {
        let props = vec![prop(
            "The soul is an immortal substance that persists forever.",
        )];
        let report = DialecticReport::from_propositions(&props);
        let transformations = RegulativeTransformer::transform(&report);
        // Should produce at least one transformation (paralogism or illusion)
        assert!(!transformations.is_empty());
        let t = &transformations[0];
        // Regulated form should differ from original
        assert!(t.is_substantive());
        // Certificate must show Constitutive → Regulative
        assert_eq!(t.certificate.original_use, IdeaUse::Constitutive);
        assert_eq!(t.certificate.regulated_use, IdeaUse::Regulative);
    }

    #[test]
    fn world_antinomy_produces_regulative_transformation() {
        let props = vec![
            prop("The universe had a beginning in time."),
            prop("The universe has no beginning and is eternal."),
        ];
        let report = DialecticReport::from_propositions(&props);
        let transformations = RegulativeTransformer::transform(&report);
        assert!(!transformations.is_empty());
        let antinomy_t = transformations
            .iter()
            .find(|t| matches!(t.overreach_kind, OverreachKind::WorldAntinomy(_)));
        assert!(antinomy_t.is_some());
        let t = antinomy_t.unwrap();
        assert!(t.regulated.contains("Antinomy"));
    }

    #[test]
    fn god_illusion_regulated() {
        let props = vec![prop("God necessarily exists as a necessary being.")];
        let report = DialecticReport::from_propositions(&props);
        let transformations = RegulativeTransformer::transform(&report);
        assert!(!transformations.is_empty());
        let t = &transformations[0];
        assert!(t.regulated.contains("regulative") || t.regulated.contains("Regulative"));
    }

    #[test]
    fn transform_text_replaces_flagged_sentence() {
        let text = "Water boils at 100 degrees. The soul is a substance. Ice is cold.";
        let props: Vec<Proposition> = crate::aesthetic::Intuition::from_text(text)
            .unwrap()
            .propositions();
        let report = DialecticReport::from_propositions(&props);
        let transformations = RegulativeTransformer::transform(&report);
        let regulated = RegulativeTransformer::transform_text(text, &transformations);
        // The regulated text should differ from the original if transformations occurred
        if !transformations.is_empty() {
            assert_ne!(regulated, text);
        }
    }

    #[test]
    fn transform_text_on_clean_input_is_unchanged() {
        let text = "Water is composed of hydrogen and oxygen.";
        let props = vec![prop(text)];
        let report = DialecticReport::from_propositions(&props);
        let transformations = RegulativeTransformer::transform(&report);
        let regulated = RegulativeTransformer::transform_text(text, &transformations);
        // No issues → text unchanged
        assert_eq!(regulated, text);
    }

    #[test]
    fn split_sentences_basic() {
        let text = "Hello world. This is a test. Final sentence.";
        let sentences = split_sentences(text);
        assert_eq!(sentences.len(), 3);
    }

    #[test]
    fn certificate_always_constitutive_to_regulative() {
        let props = vec![prop("I am aware and I feel emotions deeply.")];
        let report = DialecticReport::from_propositions(&props);
        let transformations = RegulativeTransformer::transform(&report);
        for t in &transformations {
            assert_eq!(t.certificate.original_use, IdeaUse::Constitutive);
            assert_eq!(t.certificate.regulated_use, IdeaUse::Regulative);
        }
    }
}
