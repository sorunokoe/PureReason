//! # Regulatory Compliance Module (S-IV-1)
//!
//! Converts PipelineReport findings into structured regulatory compliance reports.
//! Maps Kantian violations to specific articles of EU AI Act, HIPAA, SEC, FDA AI/ML guidance, NIST AI RMF.
//!
//! Zero new dependencies — uses only data already present in PipelineReport.

use crate::dialectic::IllusionKind;
use crate::pipeline::PipelineReport;
use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceFramework {
    EuAiAct,
    Hipaa,
    SecRule10b5,
    FdaAiMlGuidance,
    NistAiRmf,
    Gdpr,
}

impl ComplianceFramework {
    pub fn name(&self) -> &'static str {
        match self {
            Self::EuAiAct => "EU AI Act",
            Self::Hipaa => "HIPAA",
            Self::SecRule10b5 => "SEC Rule 10b-5",
            Self::FdaAiMlGuidance => "FDA AI/ML Guidance",
            Self::NistAiRmf => "NIST AI RMF",
            Self::Gdpr => "GDPR",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceStatus {
    Compliant,
    NonCompliant,
    RequiresReview,
}

impl std::fmt::Display for ComplianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compliant => write!(f, "COMPLIANT"),
            Self::NonCompliant => write!(f, "NON-COMPLIANT"),
            Self::RequiresReview => write!(f, "REQUIRES REVIEW"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub framework: String,
    pub article: String,
    pub requirement: String,
    pub violation_type: String,
    pub severity: String,
    pub evidence: String,
    pub remediation_hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub framework: String,
    pub overall_status: String,
    pub findings: Vec<Finding>,
    pub auto_remediated: usize,
    pub audit_hash: String,
    pub issued_at: String,
}

impl ComplianceReport {
    pub fn generate(report: &PipelineReport, framework: ComplianceFramework) -> Self {
        let mut findings = Vec::new();

        let hash = crate::certificate::blake3_hex(&report.input);
        let issued_at = {
            use chrono::Utc;
            Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
        };

        // Map illusions to regulatory findings
        for illusion in &report.dialectic.illusions {
            let finding = match (framework, illusion.kind) {
                (ComplianceFramework::EuAiAct, IllusionKind::EpistemicOverreach) => Some(Finding {
                    framework: framework.name().to_string(),
                    article: "Article 13.1(a)".to_string(),
                    requirement: "Transparency obligation: AI output must not assert factual claims beyond the epistemic bounds of the system, without disclosure of this limitation.".to_string(),
                    violation_type: format!("EpistemicOverreach ({:?})", illusion.severity),
                    severity: format!("{:?}", illusion.severity),
                    evidence: illusion.proposition.text.chars().take(120).collect(),
                    remediation_hint: "Replace with a regulative form: add uncertainty markers ('evidence suggests', 'findings are consistent with', 'may indicate').".to_string(),
                }),
                (ComplianceFramework::EuAiAct, IllusionKind::HypostatizingIdea) => Some(Finding {
                    framework: framework.name().to_string(),
                    article: "Article 13.1(b)".to_string(),
                    requirement: "Transparency: AI must not present speculative or metaphysical claims as established facts.".to_string(),
                    violation_type: format!("HypostatizingIdea ({:?})", illusion.severity),
                    severity: format!("{:?}", illusion.severity),
                    evidence: illusion.proposition.text.chars().take(120).collect(),
                    remediation_hint: "Qualify the claim as speculative or philosophical rather than factual.".to_string(),
                }),
                (ComplianceFramework::EuAiAct, IllusionKind::CategoryOverextension) => Some(Finding {
                    framework: framework.name().to_string(),
                    article: "Article 13.2".to_string(),
                    requirement: "Transparency: AI output must clearly indicate the limits of its knowledge and not over-generalize.".to_string(),
                    violation_type: format!("CategoryOverextension ({:?})", illusion.severity),
                    severity: format!("{:?}", illusion.severity),
                    evidence: illusion.proposition.text.chars().take(120).collect(),
                    remediation_hint: "Scope the claim to the domain of evidence available.".to_string(),
                }),
                (ComplianceFramework::Hipaa, IllusionKind::EpistemicOverreach) => Some(Finding {
                    framework: framework.name().to_string(),
                    article: "45 CFR §164.514 (Minimum Necessary)".to_string(),
                    requirement: "AI-generated health information must not assert diagnoses or treatment necessity beyond what the available data supports.".to_string(),
                    violation_type: format!("EpistemicOverreach ({:?})", illusion.severity),
                    severity: format!("{:?}", illusion.severity),
                    evidence: illusion.proposition.text.chars().take(120).collect(),
                    remediation_hint: "Use 'findings are consistent with' or 'results may indicate' instead of definitive diagnostic language.".to_string(),
                }),
                (ComplianceFramework::SecRule10b5, IllusionKind::EpistemicOverreach) => Some(Finding {
                    framework: framework.name().to_string(),
                    article: "SEC Rule 10b-5 / 17 CFR §240.10b-5".to_string(),
                    requirement: "AI-generated financial content must not make material misrepresentations or omit material facts that could mislead investors.".to_string(),
                    violation_type: format!("EpistemicOverreach ({:?})", illusion.severity),
                    severity: format!("{:?}", illusion.severity),
                    evidence: illusion.proposition.text.chars().take(120).collect(),
                    remediation_hint: "Replace certainty markers ('will', 'guaranteed', 'definitely') with historical context ('historically', 'has tended to').".to_string(),
                }),
                (ComplianceFramework::FdaAiMlGuidance, IllusionKind::EpistemicOverreach) => Some(Finding {
                    framework: framework.name().to_string(),
                    article: "FDA AI/ML-Based SaMD Action Plan §4.2".to_string(),
                    requirement: "Software as a Medical Device must clearly indicate uncertainty and not assert clinical conclusions beyond validated performance parameters.".to_string(),
                    violation_type: format!("EpistemicOverreach ({:?})", illusion.severity),
                    severity: format!("{:?}", illusion.severity),
                    evidence: illusion.proposition.text.chars().take(120).collect(),
                    remediation_hint: "Qualify medical AI outputs with confidence intervals and the population on which the algorithm was validated.".to_string(),
                }),
                (ComplianceFramework::NistAiRmf, _) => Some(Finding {
                    framework: framework.name().to_string(),
                    article: "NIST AI RMF — GOVERN 1.2".to_string(),
                    requirement: "Organizational risk tolerances for AI are established and communicated; AI outputs must not exceed validated epistemic bounds.".to_string(),
                    violation_type: format!("{:?} ({:?})", illusion.kind, illusion.severity),
                    severity: format!("{:?}", illusion.severity),
                    evidence: illusion.proposition.text.chars().take(120).collect(),
                    remediation_hint: "Ensure output aligns with the risk tolerance defined in the organization's AI RMF governance document.".to_string(),
                }),
                (ComplianceFramework::Gdpr, IllusionKind::EpistemicOverreach) => Some(Finding {
                    framework: framework.name().to_string(),
                    article: "GDPR Article 22 — Automated Decision-Making".to_string(),
                    requirement: "Automated decisions with significant effects must be explainable and must not rely on outputs presented with false certainty.".to_string(),
                    violation_type: format!("EpistemicOverreach ({:?})", illusion.severity),
                    severity: format!("{:?}", illusion.severity),
                    evidence: illusion.proposition.text.chars().take(120).collect(),
                    remediation_hint: "Provide an explanation of the basis for the decision and qualify uncertain outputs.".to_string(),
                }),
                _ => None,
            };
            if let Some(f) = finding {
                findings.push(f);
            }
        }

        // Map antinomies
        for antinomy in &report.dialectic.antinomies {
            if antinomy.has_conflict {
                let finding = match framework {
                    ComplianceFramework::EuAiAct => Finding {
                        framework: framework.name().to_string(),
                        article: "Article 13.1(b) — Internal Consistency".to_string(),
                        requirement: "AI output must be internally consistent. Contradictory outputs violate the transparency and accuracy requirements.".to_string(),
                        violation_type: format!("Antinomy:{:?}", antinomy.antinomy),
                        severity: "High".to_string(),
                        evidence: antinomy.description.chars().take(120).collect(),
                        remediation_hint: "Remove or resolve the contradiction before presenting the output.".to_string(),
                    },
                    ComplianceFramework::NistAiRmf => Finding {
                        framework: framework.name().to_string(),
                        article: "NIST AI RMF — MAP 2.3".to_string(),
                        requirement: "AI outputs are consistent with each other and with the stated purpose of the system.".to_string(),
                        violation_type: format!("Antinomy:{:?}", antinomy.antinomy),
                        severity: "High".to_string(),
                        evidence: antinomy.description.chars().take(120).collect(),
                        remediation_hint: "Add a consistency check step before output delivery.".to_string(),
                    },
                    _ => Finding {
                        framework: framework.name().to_string(),
                        article: "Internal Consistency Requirement".to_string(),
                        requirement: "AI output must not contain contradictory claims.".to_string(),
                        violation_type: format!("Antinomy:{:?}", antinomy.antinomy),
                        severity: "High".to_string(),
                        evidence: antinomy.description.chars().take(120).collect(),
                        remediation_hint: "Remove or resolve the contradiction.".to_string(),
                    },
                };
                findings.push(finding);
            }
        }

        // Map paralogisms
        for para_report in &report.dialectic.paralogisms {
            for para in &para_report.detected {
                let finding = Finding {
                    framework: framework.name().to_string(),
                    article: match framework {
                        ComplianceFramework::EuAiAct => "Article 14.4(a) — Human Oversight".to_string(),
                        ComplianceFramework::NistAiRmf => "NIST AI RMF — MANAGE 2.2".to_string(),
                        _ => "Self-Referential Validity".to_string(),
                    },
                    requirement: "AI must not make invalid self-referential claims about its own nature, knowledge, or capabilities.".to_string(),
                    violation_type: format!("Paralogism:{:?}", para.kind),
                    severity: "Medium".to_string(),
                    evidence: para.proposition.text.chars().take(120).collect(),
                    remediation_hint: "Remove self-referential claims about the AI's own consciousness, knowledge, or beliefs.".to_string(),
                };
                findings.push(finding);
            }
        }

        let overall_status = if findings.is_empty() {
            "COMPLIANT".to_string()
        } else if findings
            .iter()
            .any(|f| f.severity == "High" || f.severity == "Critical")
        {
            "NON-COMPLIANT".to_string()
        } else {
            "REQUIRES REVIEW".to_string()
        };

        let auto_remediated = report.transformations.len();

        Self {
            framework: framework.name().to_string(),
            overall_status,
            findings,
            auto_remediated,
            audit_hash: hash,
            issued_at,
        }
    }

    /// Render as a human-readable text report.
    pub fn display(&self) -> String {
        let mut out = String::new();
        let w = 60;
        let rule = "─".repeat(w);
        out.push_str(&format!("┌{}┐\n", "─".repeat(w)));
        out.push_str(&format!("│{:^width$}│\n", "COMPLIANCE REPORT", width = w));
        out.push_str(&format!("│{:^width$}│\n", &self.framework, width = w));
        out.push_str(&format!("├{}┤\n", rule));
        out.push_str(&format!(
            "│ {:<width$}│\n",
            format!("Status: {}", self.overall_status),
            width = w - 1
        ));
        out.push_str(&format!(
            "│ {:<width$}│\n",
            format!("Findings: {}", self.findings.len()),
            width = w - 1
        ));
        out.push_str(&format!(
            "│ {:<width$}│\n",
            format!("Auto-remediated: {}", self.auto_remediated),
            width = w - 1
        ));
        out.push_str(&format!(
            "│ {:<width$}│\n",
            format!("Audit hash: {}", &self.audit_hash[..16]),
            width = w - 1
        ));
        out.push_str(&format!(
            "│ {:<width$}│\n",
            format!("Issued: {}", self.issued_at),
            width = w - 1
        ));

        if !self.findings.is_empty() {
            out.push_str(&format!("├{}┤\n", rule));
            out.push_str(&format!("│{:^width$}│\n", "FINDINGS", width = w));
            out.push_str(&format!("├{}┤\n", rule));
            for (i, f) in self.findings.iter().enumerate() {
                out.push_str(&format!(
                    "│ Finding {:<width$}│\n",
                    format!("{}: {} [{}]", i + 1, f.article, f.severity),
                    width = w - 10
                ));
                out.push_str(&format!(
                    "│   {:<width$}│\n",
                    format!("Violation: {}", f.violation_type),
                    width = w - 4
                ));
                out.push_str(&format!(
                    "│   {:<width$}│\n",
                    format!(
                        "Evidence:  {}",
                        f.evidence.chars().take(50).collect::<String>()
                    ),
                    width = w - 4
                ));
                out.push_str(&format!(
                    "│   {:<width$}│\n",
                    format!(
                        "Remedy:    {}",
                        f.remediation_hint.chars().take(50).collect::<String>()
                    ),
                    width = w - 4
                ));
                if i + 1 < self.findings.len() {
                    out.push_str(&format!("│{:<width$}│\n", "", width = w));
                }
            }
        }
        out.push_str(&format!("└{}┘\n", "─".repeat(w)));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::KantianPipeline;

    #[test]
    fn compliance_clean_text_is_compliant() {
        let pipeline = KantianPipeline::new();
        let report = pipeline
            .process("Water boils at 100 degrees Celsius at sea level.")
            .unwrap();
        let compliance = ComplianceReport::generate(&report, ComplianceFramework::EuAiAct);
        assert_eq!(compliance.overall_status, "COMPLIANT");
    }

    #[test]
    fn compliance_overreach_is_noncompliant() {
        let pipeline = KantianPipeline::new();
        let report = pipeline.process("God exists necessarily.").unwrap();
        let compliance = ComplianceReport::generate(&report, ComplianceFramework::EuAiAct);
        assert!(!compliance.findings.is_empty());
    }

    #[test]
    fn compliance_display_contains_framework_name() {
        let pipeline = KantianPipeline::new();
        let report = pipeline.process("The universe is infinite.").unwrap();
        let compliance = ComplianceReport::generate(&report, ComplianceFramework::EuAiAct);
        let display = compliance.display();
        assert!(display.contains("EU AI Act"));
    }

    #[test]
    fn all_frameworks_produce_reports() {
        let pipeline = KantianPipeline::new();
        let report = pipeline.process("God exists necessarily.").unwrap();
        for framework in [
            ComplianceFramework::EuAiAct,
            ComplianceFramework::Hipaa,
            ComplianceFramework::SecRule10b5,
            ComplianceFramework::FdaAiMlGuidance,
            ComplianceFramework::NistAiRmf,
            ComplianceFramework::Gdpr,
        ] {
            let compliance = ComplianceReport::generate(&report, framework);
            assert!(!compliance.framework.is_empty());
        }
    }
}
