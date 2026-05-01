//! # Language Games and Forms of Life
//!
//! Late Wittgenstein's central insight: "the meaning of a word is its use
//! in the language." (PI §43)
//!
//! A **language game** (Sprachspiel) is a complete form of linguistic activity,
//! embedded in a **form of life** (Lebensform) — the shared practices and
//! activities that give language its meaning.
//!
//! Examples of language games: giving orders, describing objects, reporting events,
//! speculating about events, making up stories, play-acting, singing, guessing riddles,
//! making jokes, translating, asking / thanking / cursing / greeting / praying.

use serde::{Deserialize, Serialize};

// ─── FormOfLife ──────────────────────────────────────────────────────────────

/// A form of life — the background practices that ground a language game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FormOfLife {
    Scientific,
    Moral,
    Mathematical,
    Aesthetic,
    Religious,
    Everyday,
    Legal,
    Technical,
    Philosophical,
    Narrative,
    Unknown,
}

impl FormOfLife {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Scientific => "Scientific",
            Self::Moral => "Moral/Ethical",
            Self::Mathematical => "Mathematical",
            Self::Aesthetic => "Aesthetic/Artistic",
            Self::Religious => "Religious/Theological",
            Self::Everyday => "Everyday/Practical",
            Self::Legal => "Legal/Juridical",
            Self::Technical => "Technical/Engineering",
            Self::Philosophical => "Philosophical",
            Self::Narrative => "Narrative/Literary",
            Self::Unknown => "Unknown",
        }
    }

    pub fn signal_terms(&self) -> &'static [&'static str] {
        match self {
            Self::Scientific => &[
                "hypothesis",
                "experiment",
                "data",
                "theory",
                "evidence",
                "empirical",
                "observation",
                "measurement",
                "results",
                "methodology",
                // Common physics/chemistry/biology vocabulary
                "causes",
                "because",
                "temperature",
                "pressure",
                "energy",
                "force",
                "reaction",
                "velocity",
                "mass",
                "gravity",
                "chemical",
                "biological",
                "boils",
                "melts",
                "accelerates",
                "decays",
                "evolves",
                "radiates",
                "frequency",
                "wavelength",
                "molecule",
                "atom",
                "cell",
                "organism",
            ],
            Self::Moral => &[
                "ought", "should", "duty", "virtue", "ethics", "right", "wrong", "moral", "good",
                "evil", "just", "fair",
            ],
            Self::Mathematical => &[
                "theorem",
                "proof",
                "axiom",
                "equation",
                "function",
                "number",
                "calculate",
                "derive",
                "lemma",
                "corollary",
            ],
            Self::Aesthetic => &[
                "beautiful",
                "sublime",
                "artistic",
                "aesthetic",
                "style",
                "form",
                "expression",
                "creativity",
                "art",
            ],
            Self::Religious => &[
                "god",
                "sacred",
                "holy",
                "prayer",
                "faith",
                "belief",
                "soul",
                "sin",
                "grace",
                "divine",
                "scripture",
            ],
            Self::Everyday => &[
                "today", "dinner", "work", "home", "family", "friend", "tomorrow", "weather",
                "buy", "do",
            ],
            Self::Legal => &[
                "law",
                "legal",
                "court",
                "judge",
                "verdict",
                "rights",
                "obligation",
                "contract",
                "statute",
                "jurisdiction",
            ],
            Self::Technical => &[
                "algorithm",
                "system",
                "code",
                "function",
                "module",
                "interface",
                "protocol",
                "architecture",
                "implementation",
            ],
            Self::Philosophical => &[
                "being",
                "existence",
                "consciousness",
                "knowledge",
                "truth",
                "reason",
                "mind",
                "concept",
                "epistemology",
                "ontology",
            ],
            Self::Narrative => &[
                "story",
                "character",
                "plot",
                "narrative",
                "chapter",
                "protagonist",
                "happened",
                "once upon",
                "told",
            ],
            Self::Unknown => &[],
        }
    }
}

// ─── LanguageGame ─────────────────────────────────────────────────────────────

/// A language game — a complete form of linguistic activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageGame {
    pub name: String,
    pub form_of_life: FormOfLife,
    pub rules: Vec<String>,
    pub typical_moves: Vec<String>,
    pub confidence: f64,
}

impl LanguageGame {
    pub fn for_form_of_life(form: FormOfLife, confidence: f64) -> Self {
        let (rules, typical_moves) = form_rules_and_moves(form);
        Self {
            name: form.name().to_string(),
            form_of_life: form,
            rules,
            typical_moves,
            confidence,
        }
    }
}

fn form_rules_and_moves(form: FormOfLife) -> (Vec<String>, Vec<String>) {
    match form {
        FormOfLife::Scientific => (
            vec![
                "Claims must be testable and falsifiable".to_string(),
                "Evidence must be publicly verifiable".to_string(),
                "Theories must make predictions".to_string(),
            ],
            vec![
                "Formulate hypothesis".to_string(),
                "Design experiment".to_string(),
                "Report results".to_string(),
            ],
        ),
        FormOfLife::Moral => (
            vec![
                "Actions are assessed as right or wrong".to_string(),
                "Reasons must be given for moral claims".to_string(),
                "Universal applicability is expected".to_string(),
            ],
            vec![
                "Assert ought".to_string(),
                "Give moral reason".to_string(),
                "Apply principle".to_string(),
            ],
        ),
        FormOfLife::Mathematical => (
            vec![
                "Claims must be provable from axioms".to_string(),
                "No exceptions to proven theorems".to_string(),
                "Definitions are stipulative and precise".to_string(),
            ],
            vec![
                "Define term".to_string(),
                "State theorem".to_string(),
                "Prove or refute".to_string(),
            ],
        ),
        FormOfLife::Philosophical => (
            vec![
                "Claims must be conceptually coherent".to_string(),
                "Definitions and distinctions matter".to_string(),
                "Counter-examples are decisive".to_string(),
            ],
            vec![
                "Analyze concept".to_string(),
                "Draw distinction".to_string(),
                "Raise objection".to_string(),
            ],
        ),
        _ => (
            vec!["Context-dependent rules apply".to_string()],
            vec!["Typical moves depend on context".to_string()],
        ),
    }
}

// ─── GameAnalysis ────────────────────────────────────────────────────────────

/// The result of analyzing which language game is being played.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameAnalysis {
    /// All detected language games, sorted by confidence.
    pub detected_games: Vec<LanguageGame>,
    /// The most likely language game.
    pub primary_game: Option<LanguageGame>,
    /// Whether the text mixes language games (potential source of confusion).
    pub is_mixed: bool,
    pub interpretation_note: String,
}

// ─── GameDetector ─────────────────────────────────────────────────────────────

/// Detects which language game(s) a text belongs to.
pub struct GameDetector;

impl GameDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze(&self, text: &str) -> GameAnalysis {
        let text_lower = text.to_lowercase();
        let words: Vec<&str> = text_lower.split_whitespace().collect();
        let total = words.len().max(1);

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

        let mut games: Vec<LanguageGame> = forms
            .iter()
            .filter_map(|&form| {
                let hits = words
                    .iter()
                    .filter(|&&w| form.signal_terms().contains(&w))
                    .count();
                if hits > 0 {
                    let confidence = (hits as f64 / total as f64 * 10.0).min(1.0);
                    Some(LanguageGame::for_form_of_life(form, confidence))
                } else {
                    None
                }
            })
            .collect();

        games.sort_by(|a, b| b.confidence.total_cmp(&a.confidence));

        let primary_game = games.first().cloned();
        let is_mixed = games.len() > 1 && {
            let top = games[0].confidence;
            games
                .get(1)
                .map(|g| g.confidence / top > 0.5)
                .unwrap_or(false)
        };

        let interpretation_note = match (is_mixed, &primary_game) {
            (true, _) => "Mixed language games detected — be cautious of cross-game confusion. \
                         Terms may mean different things in different games."
                .to_string(),
            (false, Some(g)) => format!(
                "Primary language game: {}. Interpret terms within this context.",
                g.name
            ),
            (false, None) => "No specific language game strongly detected. \
                             Applying general everyday language game interpretation."
                .to_string(),
        };

        GameAnalysis {
            detected_games: games,
            primary_game,
            is_mixed,
            interpretation_note,
        }
    }
}

impl Default for GameDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scientific_game_detected() {
        let text =
            "The hypothesis was tested through rigorous empirical observation and measurement";
        let detector = GameDetector::new();
        let analysis = detector.analyze(text);
        let scientific = analysis
            .detected_games
            .iter()
            .find(|g| g.form_of_life == FormOfLife::Scientific);
        assert!(scientific.is_some());
    }

    #[test]
    fn moral_game_detected() {
        let text = "We ought to act morally and fulfill our duty to do what is right";
        let detector = GameDetector::new();
        let analysis = detector.analyze(text);
        let moral = analysis
            .detected_games
            .iter()
            .find(|g| g.form_of_life == FormOfLife::Moral);
        assert!(moral.is_some());
    }

    #[test]
    fn primary_game_is_highest_confidence() {
        let text = "The theorem can be proven from the axioms by mathematical induction";
        let detector = GameDetector::new();
        let analysis = detector.analyze(text);
        if let (Some(primary), Some(first)) =
            (&analysis.primary_game, analysis.detected_games.first())
        {
            assert_eq!(primary.form_of_life, first.form_of_life);
        }
    }
}
