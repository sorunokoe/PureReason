//! # Validation Certificate (S-III-7)
//!
//! Pipeline-level attestation for any PureReason analysis run.
//!
//! Provides a content-addressable record of what was analysed and what
//! PureReason concluded. The `content_hash` is a BLAKE3 fingerprint (first 32
//! hex chars / 16 bytes) of the input text that can be independently recomputed
//! to verify provenance.
//!
//! BLAKE3 is used instead of FNV-64 because FNV is not cryptographically secure
//! — it is trivially reversible and collision-prone for adversarial inputs.
//! BLAKE3 provides collision resistance suitable for audit trails.
//!
//! ## Distinction from `dialectic::regulative::EpistemicCertificate`
//!
//! [`EpistemicCertificate`](crate::dialectic::regulative::EpistemicCertificate)
//! documents a constitutive→regulative *transformation* at the claim level.
//! This struct documents the **outcome of a full pipeline run** at the text level.
//!
//! ## Format
//! ```json
//! {
//!   "content_hash": "a3f5c7d8e1b2f4a9c0d1e2f3a4b5c6d7",
//!   "validator_version": "0.1.0",
//!   "issued_at": "2025-01-15T12:00:00Z",
//!   "risk_level": "Medium",
//!   "issues": ["Illusion:HypostatizingIdea", "Antinomy:Third"],
//!   "regulated": true,
//!   "dominant_category": "Necessity",
//!   "verify_hint": "pure-reason certify --hash a3f5c7d8e1b2f4a9c0d1e2f3a4b5c6d7"
//! }
//! ```

use crate::pipeline::PipelineReport;
use serde::{Deserialize, Serialize};

// ─── ValidationCertificate ───────────────────────────────────────────────────

/// A content-addressable certificate for a PureReason validation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCertificate {
    /// BLAKE3 hex fingerprint of the input text (32 hex chars / 16 bytes).
    pub content_hash: String,
    /// PureReason library version at time of issuance.
    pub validator_version: String,
    /// ISO 8601 UTC timestamp of the analysis.
    pub issued_at: String,
    /// Overall risk verdict: `"SAFE"`, `"LOW"`, `"MEDIUM"`, or `"HIGH"`.
    pub risk_level: String,
    /// Detected issues (illusion kinds, antinomy IDs, paralogism types).
    pub issues: Vec<String>,
    /// Whether regulative transformation was applied to the text.
    pub regulated: bool,
    /// Dominant Kantian category detected (if any).
    pub dominant_category: Option<String>,
    /// CLI command that reproduces this analysis for verification.
    pub verify_hint: String,
}

impl ValidationCertificate {
    /// Build a certificate from a completed pipeline report.
    pub fn from_report(report: &PipelineReport) -> Self {
        let hash = blake3_hex(&report.input);

        let issued_at = {
            use chrono::Utc;
            Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
        };

        let mut issues = Vec::new();
        for illusion in &report.dialectic.illusions {
            issues.push(format!("Illusion:{:?}", illusion.kind));
        }
        for antinomy in &report.dialectic.antinomies {
            if antinomy.has_conflict {
                issues.push(format!("Antinomy:{:?}", antinomy.antinomy));
            }
        }
        for para_report in &report.dialectic.paralogisms {
            for para in &para_report.detected {
                issues.push(format!("Paralogism:{:?}", para.kind));
            }
        }

        Self {
            verify_hint: format!("pure-reason certify --hash {}", hash),
            content_hash: hash,
            validator_version: env!("CARGO_PKG_VERSION").to_string(),
            issued_at,
            risk_level: report.verdict.risk.to_string(),
            issues,
            regulated: !report.transformations.is_empty(),
            dominant_category: report.verdict.dominant_category.clone(),
        }
    }

    /// Verify that a hash was produced from the given input text.
    ///
    /// Returns `true` if `blake3_hex(input) == expected_hash`.
    pub fn verify(input: &str, expected_hash: &str) -> bool {
        blake3_hex(input) == expected_hash
    }

    /// Render the certificate as a human-readable bordered summary.
    pub fn display(&self) -> String {
        let w = 52usize;
        let rule = "═".repeat(w);
        let mut out = String::new();
        out.push_str(&format!("╔{}╗\n", rule));
        out.push_str(&format!(
            "║{:^width$}║\n",
            "PURE REASON VALIDATION CERTIFICATE",
            width = w
        ));
        out.push_str(&format!("╠{}╣\n", rule));
        push_row(&mut out, "Hash", &self.content_hash, w);
        push_row(&mut out, "Issued", &self.issued_at, w);
        push_row(&mut out, "Version", &self.validator_version, w);
        push_row(&mut out, "Risk", &self.risk_level, w);
        push_row(&mut out, "Regulated", &self.regulated.to_string(), w);
        if let Some(cat) = &self.dominant_category {
            push_row(&mut out, "Category", cat, w);
        }
        if self.issues.is_empty() {
            push_row(&mut out, "Issues", "none", w);
        } else {
            push_row(&mut out, "Issues", &self.issues.len().to_string(), w);
            for issue in &self.issues {
                out.push_str(&format!("║  • {:<width$}║\n", issue, width = w - 4));
            }
        }
        out.push_str(&format!("╠{}╣\n", rule));
        push_row(&mut out, "Verify", &self.verify_hint, w);
        out.push_str(&format!("╚{}╝\n", rule));
        out
    }
}

fn push_row(out: &mut String, label: &str, value: &str, width: usize) {
    let content = format!("{}: {}", label, value);
    out.push_str(&format!("║ {:<width$} ║\n", content, width = width - 2));
}

// ─── BLAKE3 hash ──────────────────────────────────────────────────────────────

/// BLAKE3 hash → 32-char hex string (first 16 bytes of the 32-byte digest).
///
/// BLAKE3 is cryptographically secure (collision-resistant, preimage-resistant)
/// and suitable for content-addressable audit trails, unlike FNV-64.
pub fn blake3_hex(input: &str) -> String {
    let digest = blake3::hash(input.as_bytes());
    // Use the first 16 bytes (128 bits) → 32 hex chars.
    let bytes = &digest.as_bytes()[..16];
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blake3_is_deterministic() {
        assert_eq!(blake3_hex("hello world"), blake3_hex("hello world"));
    }

    #[test]
    fn blake3_different_inputs_differ() {
        assert_ne!(blake3_hex("hello"), blake3_hex("world"));
    }

    #[test]
    fn verify_roundtrip() {
        let text = "The cat sat on the mat.";
        let hash = blake3_hex(text);
        assert!(ValidationCertificate::verify(text, &hash));
        assert!(!ValidationCertificate::verify("different text", &hash));
    }

    #[test]
    fn certificate_from_safe_report() {
        use crate::pipeline::KantianPipeline;
        let pipeline = KantianPipeline::new();
        let report = pipeline.process("Water boils at 100 degrees.").unwrap();
        let cert = ValidationCertificate::from_report(&report);
        assert_eq!(cert.content_hash, blake3_hex("Water boils at 100 degrees."));
        assert!(!cert.content_hash.is_empty());
        assert!(cert.risk_level == "SAFE" || cert.risk_level == "LOW");
    }

    #[test]
    fn certificate_display_contains_hash() {
        use crate::pipeline::KantianPipeline;
        let pipeline = KantianPipeline::new();
        let report = pipeline.process("Ice melts at 0 degrees.").unwrap();
        let cert = ValidationCertificate::from_report(&report);
        let display = cert.display();
        assert!(display.contains(&cert.content_hash));
        assert!(display.contains("PURE REASON"));
    }
}
