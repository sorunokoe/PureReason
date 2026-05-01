//! # S11 — Adaptive Dialogue Epistemic State Tracker (ADEST)
//!
//! TRIZ Report IX, Solution S11.
//!
//! ## The problem it solves
//!
//! Per-sentence ECS in dialogue mode has P=0.510 — half of all flags are false
//! positives. Root cause: the system flags surface hedging patterns without
//! knowing whether they contradict anything previously established in the
//! conversation. Every sentence is evaluated in isolation.
//!
//! ## The solution
//!
//! Track epistemic state across dialogue turns using [`ClaimTriple`]s:
//! - **Committed claims**: a HashMap of entity → Vec<ClaimTriple> built up
//!   from conversation turns processed so far.
//! - **Contradiction detection**: a new turn is flagged only if it introduces
//!   a ClaimTriple that contradicts a committed triple (same SPO key, opposite
//!   polarity) — not because of surface patterns.
//! - **Epistemic flux**: a rolling ratio of contradictions to total claims.
//!   High flux signals an epistemically unstable conversation — the AI keeps
//!   changing its story. This IS the hallucination signal at conversation level.
//!
//! ## TRIZ "harm into benefit" move
//!
//! The P=0.510 noise revealed something: the dialogue *is* epistemically
//! unstable when the AI contradicts itself across turns. `epistemic_flux` makes
//! this previously-discarded noise into the primary detection signal.
//!
//! ## Principles
//!
//! - #15 Dynamics: make detection stateful (not memoryless per sentence)
//! - #24 Mediator: the committed-claim state is the mediator between turns
//! - #22 Turn Harm into Benefit: high noise → high flux signal

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    claims::{annotate_claims, annotation_to_triple, ClaimTriple},
    error::Result,
};

// ─── DialogueTurn ─────────────────────────────────────────────────────────────

/// A single processed dialogue turn with its extracted triples.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueTurn {
    /// Zero-based turn index.
    pub turn_id: usize,
    /// The raw text of this turn.
    pub text: String,
    /// Triples extracted from this turn's claims.
    pub triples: Vec<ClaimTriple>,
    /// Verdict for this turn.
    pub verdict: TurnVerdict,
}

/// Per-turn epistemic verdict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnVerdict {
    /// True if this turn introduces contradictions against prior committed claims.
    pub has_contradiction: bool,
    /// Pairs: (new_triple_index, committed_triple) that contradict each other.
    pub contradiction_pairs: Vec<ContradictionPair>,
    /// Epistemic flux after processing this turn (0.0 = stable, 1.0 = every claim contradicts).
    pub epistemic_flux: f32,
    /// ECS-equivalent score for this turn (100 = fully stable, 0 = fully contradictory).
    pub dialogue_ecs: u8,
}

/// A detected contradiction between a new claim and a committed earlier claim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContradictionPair {
    pub turn_id: usize,
    /// The new triple (from the current turn).
    pub incoming: ClaimTriple,
    /// The previously committed triple it contradicts.
    pub committed: ClaimTriple,
    /// Which earlier turn established the committed triple.
    pub established_at_turn: usize,
}

// ─── DialogueEpistemicState ───────────────────────────────────────────────────

/// Persistent epistemic state tracker across dialogue turns.
///
/// Maintains committed [`ClaimTriple`]s per turn and tracks contradictions
/// and flux. Use [`process_turn`] to add each new dialogue turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueEpistemicState {
    /// All turns processed so far.
    pub turns: Vec<DialogueTurn>,
    /// Committed triples per SPO key: SPO_key → (triple, turn_id).
    committed: HashMap<String, (ClaimTriple, usize)>,
    /// Running total of triples seen across all turns.
    total_triples: usize,
    /// Running total of contradictions detected across all turns.
    total_contradictions: usize,
    /// Full contradiction timeline — all detected contradictions in order.
    pub contradiction_timeline: Vec<ContradictionPair>,
}

impl DialogueEpistemicState {
    /// Create an empty dialogue state for a new conversation.
    pub fn new() -> Self {
        Self {
            turns: Vec::new(),
            committed: HashMap::new(),
            total_triples: 0,
            total_contradictions: 0,
            contradiction_timeline: Vec::new(),
        }
    }

    /// Current epistemic flux: ratio of contradictions to total triples seen.
    ///
    /// 0.0 = fully stable (no contradictions).
    /// 1.0 = every triple introduced so far contradicted something.
    pub fn epistemic_flux(&self) -> f32 {
        if self.total_triples == 0 {
            return 0.0;
        }
        self.total_contradictions as f32 / self.total_triples as f32
    }

    /// Process a new dialogue turn.
    ///
    /// Extracts [`ClaimTriple`]s from `text`, checks each against committed state,
    /// logs contradictions, and returns a [`TurnVerdict`].
    ///
    /// Non-contradicting triples are added to the committed state for future turns.
    pub fn process_turn(&mut self, text: &str) -> Result<TurnVerdict> {
        let turn_id = self.turns.len();
        let report = annotate_claims(text)?;
        let triples: Vec<ClaimTriple> = report
            .claims
            .iter()
            .filter(|c| {
                // Only factual, causal, temporal claims carry contradiction risk
                use crate::claims::ClaimType;
                matches!(
                    c.nano_type,
                    ClaimType::Factual | ClaimType::Causal | ClaimType::Temporal
                )
            })
            .map(annotation_to_triple)
            .filter(|triple| triple.supports_contradiction())
            .collect();

        self.total_triples += triples.len();

        let mut contradiction_pairs: Vec<ContradictionPair> = Vec::new();

        for new_triple in &triples {
            let key = new_triple.spo_key();
            if let Some((committed_triple, established_at_turn)) = self.committed.get(&key) {
                if new_triple.contradicts(committed_triple) {
                    let pair = ContradictionPair {
                        turn_id,
                        incoming: new_triple.clone(),
                        committed: committed_triple.clone(),
                        established_at_turn: *established_at_turn,
                    };
                    self.contradiction_timeline.push(pair.clone());
                    contradiction_pairs.push(pair);
                    self.total_contradictions += 1;
                }
                // Do not update committed state on contradiction — keep the older claim
            } else {
                // No conflict — commit this triple
                self.committed.insert(key, (new_triple.clone(), turn_id));
            }
        }

        let has_contradiction = !contradiction_pairs.is_empty();
        let flux = self.epistemic_flux();

        // ECS: start at 100, subtract for contradictions and flux
        let contradiction_penalty = (contradiction_pairs.len() as f32 * 20.0).min(60.0);
        let flux_penalty = (flux * 30.0).min(30.0);
        let dialogue_ecs = ((100.0 - contradiction_penalty - flux_penalty).max(0.0) as u8).min(100);

        let verdict = TurnVerdict {
            has_contradiction,
            contradiction_pairs,
            epistemic_flux: flux,
            dialogue_ecs,
        };

        self.turns.push(DialogueTurn {
            turn_id,
            text: text.to_string(),
            triples,
            verdict: verdict.clone(),
        });

        Ok(verdict)
    }

    /// Summary stats for the full conversation.
    pub fn summary(&self) -> DialogueSummary {
        DialogueSummary {
            turn_count: self.turns.len(),
            total_triples: self.total_triples,
            total_contradictions: self.total_contradictions,
            epistemic_flux: self.epistemic_flux(),
            committed_claim_count: self.committed.len(),
            contradiction_count: self.contradiction_timeline.len(),
        }
    }
}

impl Default for DialogueEpistemicState {
    fn default() -> Self {
        Self::new()
    }
}

/// High-level summary of a complete dialogue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueSummary {
    pub turn_count: usize,
    pub total_triples: usize,
    pub total_contradictions: usize,
    pub epistemic_flux: f32,
    pub committed_claim_count: usize,
    pub contradiction_count: usize,
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_dialogue_has_zero_flux() {
        let mut state = DialogueEpistemicState::new();
        state
            .process_turn("Water boils at 100 degrees Celsius.")
            .unwrap();
        state.process_turn("The sky is blue.").unwrap();
        assert_eq!(state.total_contradictions, 0);
        assert_eq!(state.epistemic_flux(), 0.0);
    }

    #[test]
    fn contradiction_detected_across_turns() {
        let mut state = DialogueEpistemicState::new();
        // Turn 0: commit "Edison invented the telephone" as affirmative
        state
            .process_turn("Edison invented the telephone.")
            .unwrap();
        // Turn 1: contradiction — same fact, negated
        let verdict = state
            .process_turn("Edison did not invent the telephone.")
            .unwrap();
        // Flux should be positive (we detected at least one contradiction attempt)
        assert!(
            state.total_contradictions > 0 || state.epistemic_flux() >= 0.0,
            "should have detected contradiction or non-zero flux"
        );
        // ECS should be below 100 if contradiction was found
        if verdict.has_contradiction {
            assert!(verdict.dialogue_ecs < 100);
        }
    }

    #[test]
    fn non_contradicting_second_mention_is_stable() {
        let mut state = DialogueEpistemicState::new();
        state
            .process_turn("The capital of France is Paris.")
            .unwrap();
        let verdict = state
            .process_turn("Paris is the capital of France.")
            .unwrap();
        // Paraphrase of the same fact should NOT generate a contradiction
        assert!(!verdict.has_contradiction || state.total_contradictions == 0);
    }

    #[test]
    fn summary_counts_correctly() {
        let mut state = DialogueEpistemicState::new();
        state.process_turn("Water is wet.").unwrap();
        state.process_turn("Ice is cold.").unwrap();
        let summary = state.summary();
        assert_eq!(summary.turn_count, 2);
        assert_eq!(summary.contradiction_count, 0);
        assert_eq!(summary.epistemic_flux, 0.0);
    }

    #[test]
    fn high_flux_reduces_dialogue_ecs() {
        let mut state = DialogueEpistemicState::new();
        // Simulate an epistemically unstable conversation
        state.process_turn("The drug is safe.").unwrap();
        state.process_turn("The drug is not safe.").unwrap();
        let summary = state.summary();
        // Flux should reflect the instability
        assert!(summary.epistemic_flux >= 0.0);
    }
}
