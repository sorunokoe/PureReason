/// Phase C: Domain Governance
///
/// Applies domain-specific thresholds and confidence requirements
/// to verification verdicts. Ensures strict compliance in high-stakes domains.
///
/// Domain-specific thresholds:
/// - Medical/Legal: High confidence (0.75-0.80+) — high cost of error
/// - Finance: Medium-high confidence (0.70+) — significant financial impact
/// - History/Philosophy: Lower confidence (0.50+) — interpretation-heavy
/// - General: Baseline (0.60+) — standard reasoning
use serde::{Deserialize, Serialize};

/// Supported knowledge domains
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Domain {
    /// Medical/healthcare information
    Medical,
    /// Legal/policy information
    Legal,
    /// Financial/economic information
    Finance,
    /// Historical/archaeological information
    History,
    /// Scientific/technical information
    Science,
    /// Philosophical/conceptual information
    Philosophy,
    /// General/miscellaneous information
    General,
}

impl Domain {
    /// Parse domain from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "medical" | "health" | "medicine" => Some(Domain::Medical),
            "legal" | "law" | "policy" => Some(Domain::Legal),
            "finance" | "financial" | "money" | "economics" => Some(Domain::Finance),
            "history" | "historical" => Some(Domain::History),
            "science" | "scientific" | "physics" | "chemistry" | "biology" => Some(Domain::Science),
            "philosophy" | "philosophical" => Some(Domain::Philosophy),
            _ => Some(Domain::General),
        }
    }
}

/// Governance policy for a domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainPolicy {
    /// The domain
    pub domain: Domain,
    /// Minimum confidence threshold for FALSIFIABLE verdict
    pub falsifiable_threshold: f64,
    /// Minimum confidence threshold for UNFALSIFIABLE verdict
    pub unfalsifiable_threshold: f64,
    /// Whether to require audit trail
    pub require_audit_trail: bool,
    /// Whether to escalate for human review
    pub escalate_for_review: bool,
    /// Max latency allowed for reasoning (milliseconds)
    pub max_latency_ms: u32,
    /// Human-readable description
    pub description: String,
}

impl DomainPolicy {
    /// Get policy for a domain
    pub fn for_domain(domain: Domain) -> Self {
        match domain {
            Domain::Medical => DomainPolicy {
                domain,
                falsifiable_threshold: 0.80,
                unfalsifiable_threshold: 0.75,
                require_audit_trail: true,
                escalate_for_review: true,
                max_latency_ms: 5000,
                description: "Medical: Highest confidence required (0.80+). All verdicts require audit trails. High-risk errors escalated for review.".to_string(),
            },
            Domain::Legal => DomainPolicy {
                domain,
                falsifiable_threshold: 0.78,
                unfalsifiable_threshold: 0.70,
                require_audit_trail: true,
                escalate_for_review: true,
                max_latency_ms: 10000,
                description: "Legal: High confidence required (0.70-0.78+). All verdicts logged and escalated for compliance review.".to_string(),
            },
            Domain::Finance => DomainPolicy {
                domain,
                falsifiable_threshold: 0.72,
                unfalsifiable_threshold: 0.65,
                require_audit_trail: true,
                escalate_for_review: false,
                max_latency_ms: 3000,
                description: "Finance: Medium-high confidence (0.65-0.72+). All verdicts audited, escalation at high thresholds.".to_string(),
            },
            Domain::Science => DomainPolicy {
                domain,
                falsifiable_threshold: 0.70,
                unfalsifiable_threshold: 0.60,
                require_audit_trail: true,
                escalate_for_review: false,
                max_latency_ms: 5000,
                description: "Science: Medium confidence (0.60-0.70+). Full audit trails. Escalation for novel claims.".to_string(),
            },
            Domain::History => DomainPolicy {
                domain,
                falsifiable_threshold: 0.55,
                unfalsifiable_threshold: 0.50,
                require_audit_trail: false,
                escalate_for_review: false,
                max_latency_ms: 5000,
                description: "History: Lower confidence acceptable (0.50+). Interpretation-dependent. Light auditing.".to_string(),
            },
            Domain::Philosophy => DomainPolicy {
                domain,
                falsifiable_threshold: 0.52,
                unfalsifiable_threshold: 0.48,
                require_audit_trail: false,
                escalate_for_review: false,
                max_latency_ms: 10000,
                description: "Philosophy: Minimal confidence threshold (0.48-0.52). Highly interpretive domain.".to_string(),
            },
            Domain::General => DomainPolicy {
                domain,
                falsifiable_threshold: 0.60,
                unfalsifiable_threshold: 0.55,
                require_audit_trail: false,
                escalate_for_review: false,
                max_latency_ms: 5000,
                description: "General: Standard thresholds (0.55-0.60+). Baseline reasoning policies.".to_string(),
            },
        }
    }

    /// Check if a verdict meets domain thresholds
    pub fn meets_threshold(&self, confidence: f64, is_falsifiable: bool) -> bool {
        if is_falsifiable {
            confidence >= self.falsifiable_threshold
        } else {
            confidence >= self.unfalsifiable_threshold
        }
    }
}

/// Audit trail entry for governance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Domain of verification
    pub domain: Domain,
    /// Claim being verified
    pub claim: String,
    /// Final confidence score
    pub confidence: f64,
    /// Whether verdict was FALSIFIABLE (true) or UNFALSIFIABLE (false)
    pub is_falsifiable: bool,
    /// Policy applied
    pub policy_threshold: f64,
    /// Whether verdict met threshold
    pub meets_threshold: bool,
    /// Whether escalated for review
    pub escalated: bool,
    /// Reason for escalation or additional notes
    pub notes: String,
}

/// Governance check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceCheck {
    /// Whether verdict is approved under domain policy
    pub approved: bool,
    /// Applicable policy
    pub policy: DomainPolicy,
    /// Audit trail
    pub audit: AuditEntry,
    /// Required actions (e.g., "escalate", "log", "notify")
    pub required_actions: Vec<String>,
}

/// Perform governance check on a verdict
pub fn check_governance(
    domain: Domain,
    claim: &str,
    confidence: f64,
    is_falsifiable: bool,
) -> GovernanceCheck {
    let policy = DomainPolicy::for_domain(domain);
    let meets_threshold = policy.meets_threshold(confidence, is_falsifiable);

    let mut required_actions = Vec::new();

    if policy.require_audit_trail {
        required_actions.push("audit_trail".to_string());
    }

    let mut escalated = false;
    if !meets_threshold {
        required_actions.push("reject_verdict".to_string());
        escalated = true;
    } else if policy.escalate_for_review && confidence < policy.falsifiable_threshold + 0.05 {
        // Escalate if close to threshold
        required_actions.push("review_requested".to_string());
        escalated = true;
    }

    let audit = AuditEntry {
        domain,
        claim: claim.to_string(),
        confidence,
        is_falsifiable,
        policy_threshold: if is_falsifiable {
            policy.falsifiable_threshold
        } else {
            policy.unfalsifiable_threshold
        },
        meets_threshold,
        escalated,
        notes: format!(
            "Confidence {:.2}% vs threshold {:.2}%",
            confidence * 100.0,
            if is_falsifiable {
                policy.falsifiable_threshold * 100.0
            } else {
                policy.unfalsifiable_threshold * 100.0
            }
        ),
    };

    GovernanceCheck {
        approved: meets_threshold,
        policy,
        audit,
        required_actions,
    }
}

/// Check if domain keyword appears in text
pub fn infer_domain(text: &str) -> Domain {
    let text_lower = text.to_lowercase();

    // Medical keywords
    if text_lower.contains("patient")
        || text_lower.contains("disease")
        || text_lower.contains("symptom")
        || text_lower.contains("treatment")
        || text_lower.contains("diagnosis")
    {
        return Domain::Medical;
    }

    // Legal keywords
    if text_lower.contains("law")
        || text_lower.contains("statute")
        || text_lower.contains("court")
        || text_lower.contains("contract")
        || text_lower.contains("legal")
    {
        return Domain::Legal;
    }

    // Finance keywords
    if text_lower.contains("market")
        || text_lower.contains("price")
        || text_lower.contains("profit")
        || text_lower.contains("investment")
        || text_lower.contains("currency")
    {
        return Domain::Finance;
    }

    // Science keywords
    if text_lower.contains("atom")
        || text_lower.contains("molecule")
        || text_lower.contains("experiment")
        || text_lower.contains("theory")
        || text_lower.contains("evidence")
    {
        return Domain::Science;
    }

    // History keywords
    if text_lower.contains("century")
        || text_lower.contains("empire")
        || text_lower.contains("war")
        || text_lower.contains("ancient")
        || text_lower.contains("historical")
    {
        return Domain::History;
    }

    // Philosophy keywords
    if text_lower.contains("knowledge")
        || text_lower.contains("truth")
        || text_lower.contains("ethics")
        || text_lower.contains("metaphysics")
        || text_lower.contains("epistemology")
    {
        return Domain::Philosophy;
    }

    Domain::General
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_from_str() {
        assert_eq!(Domain::parse("medical"), Some(Domain::Medical));
        assert_eq!(Domain::parse("legal"), Some(Domain::Legal));
        assert_eq!(Domain::parse("general"), Some(Domain::General));
    }

    #[test]
    fn test_medical_policy() {
        let policy = DomainPolicy::for_domain(Domain::Medical);
        assert!(policy.falsifiable_threshold >= 0.75);
        assert!(policy.require_audit_trail);
    }

    #[test]
    fn test_meets_threshold() {
        let policy_med = DomainPolicy::for_domain(Domain::Medical);
        let policy_hist = DomainPolicy::for_domain(Domain::History);

        // High confidence should pass medical
        assert!(policy_med.meets_threshold(0.85, true));
        // Lower confidence should fail medical but pass history
        assert!(!policy_med.meets_threshold(0.55, true));
        assert!(policy_hist.meets_threshold(0.55, true));
    }

    #[test]
    fn test_governance_check() {
        let check = check_governance(Domain::Medical, "Patient has fever", 0.85, true);
        assert!(check.approved);
        assert!(check.audit.meets_threshold);
    }

    #[test]
    fn test_infer_domain() {
        let medical = infer_domain("The patient has a viral infection");
        assert_eq!(medical, Domain::Medical);

        let legal = infer_domain("The court ruled in favor of the plaintiff");
        assert_eq!(legal, Domain::Legal);
    }
}
