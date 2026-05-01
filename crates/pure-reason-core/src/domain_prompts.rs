//! # Domain-Specific Prompts
//!
//! Quick Win #2: Domain-specific prompt templates to improve reasoning
//!
//! Each domain gets optimized prompts that guide the reasoning pipeline:
//! - **Medical**: Emphasize symptoms → diagnosis → mechanism
//! - **Legal**: Emphasize arguments → precedents → synthesis
//! - **Finance**: Emphasize data → risk assessment → conclusion
//! - **Science**: Emphasize hypothesis → evidence → mechanism
//! - **Code**: Emphasize logic → execution trace → verification

use crate::domain_config::Domain;

/// System prompt template that introduces the domain context.
pub struct PromptTemplate {
    /// Domain this template is for
    pub domain: Domain,
    /// System prompt (instructions)
    pub system: String,
    /// Reasoning guidance
    pub guidance: String,
    /// Verification checklist
    pub verification: String,
}

impl PromptTemplate {
    /// Get prompt template for a domain.
    pub fn for_domain(domain: Domain) -> Self {
        match domain {
            Domain::Medical => Self::medical(),
            Domain::Legal => Self::legal(),
            Domain::Finance => Self::finance(),
            Domain::Science => Self::science(),
            Domain::Code => Self::code(),
            Domain::General => Self::general(),
        }
    }

    /// General fallback prompt.
    pub fn general() -> Self {
        Self {
            domain: Domain::General,
            system: "You are a rational reasoning system. Analyze the given claim carefully, identify assumptions, and assess validity.".to_string(),
            guidance: "1. State the main claim clearly\n2. Identify key assumptions\n3. Check for logical consistency\n4. Consider alternative interpretations".to_string(),
            verification: "- Does the conclusion follow from premises?\n- Are all assumptions stated?\n- Could the opposite be true?".to_string(),
        }
    }

    /// Medical domain: Emphasize mechanisms and plausibility.
    pub fn medical() -> Self {
        Self {
            domain: Domain::Medical,
            system: "You are a medical reasoning system. Evaluate clinical claims by checking mechanisms, drug interactions, contraindications, and evidence standards.".to_string(),
            guidance: "1. Identify the medical claim (diagnosis, treatment, mechanism)\n2. Check the mechanism: Is it biologically plausible?\n3. Assess dose/timing: Are numeric values reasonable for the context?\n4. Verify contraindications: Could this cause harm to specific populations?\n5. Check precedent: Does medical literature support this?".to_string(),
            verification: "- Is the mechanism biologically sound?\n- Are doses within normal ranges (e.g., 0.1-1000mg)?\n- Are side effects consistent with known pharmacology?\n- Are contraindications properly noted?".to_string(),
        }
    }

    /// Legal domain: Emphasize arguments and precedent.
    pub fn legal() -> Self {
        Self {
            domain: Domain::Legal,
            system: "You are a legal reasoning system. Evaluate legal claims by assessing arguments, precedents, statutory language, and logical consistency.".to_string(),
            guidance: "1. State the legal question clearly\n2. Identify both sides of the argument\n3. Check precedent: What do similar cases establish?\n4. Examine statutory language: Is interpretation literal or contextual?\n5. Assess logical strength: Are inferences valid?".to_string(),
            verification: "- Are both arguments presented fairly?\n- Is precedent correctly applied?\n- Are statutory terms clearly defined?\n- Is the logical chain sound?".to_string(),
        }
    }

    /// Finance domain: Emphasize data and risk.
    pub fn finance() -> Self {
        Self {
            domain: Domain::Finance,
            system: "You are a financial reasoning system. Evaluate claims by checking data accuracy, calculations, risk factors, and financial plausibility.".to_string(),
            guidance: "1. State the financial claim (return, risk, valuation)\n2. Verify data: Are numbers accurate and from credible sources?\n3. Check calculations: Do financial formulas apply correctly?\n4. Assess risk: What could make this claim wrong?\n5. Verify assumptions: Are market conditions realistic?".to_string(),
            verification: "- Are financial figures in realistic ranges?\n- Do calculations follow standard formulas?\n- Are risk factors acknowledged?\n- Are assumptions about market conditions stated?".to_string(),
        }
    }

    /// Science domain: Emphasize mechanisms and evidence.
    pub fn science() -> Self {
        Self {
            domain: Domain::Science,
            system: "You are a scientific reasoning system. Evaluate claims using the scientific method: testability, evidence, mechanism, and consistency with known laws.".to_string(),
            guidance: "1. State the scientific claim clearly\n2. Is it testable? Can it be verified or falsified?\n3. Check mechanism: Does it follow known physical/chemical laws?\n4. Assess evidence: Is there experimental support?\n5. Check literature: Do peer-reviewed sources support this?".to_string(),
            verification: "- Is the claim testable?\n- Does it violate known laws (e.g., thermodynamics)?\n- Is the mechanism understood?\n- Is experimental evidence cited?".to_string(),
        }
    }

    /// Code domain: Emphasize logic and execution.
    pub fn code() -> Self {
        Self {
            domain: Domain::Code,
            system: "You are a code reasoning system. Evaluate claims about code by checking logic, syntax, execution flow, and correctness.".to_string(),
            guidance: "1. State the code claim (behavior, output, logic)\n2. Trace execution: What is the actual flow?\n3. Check logic: Are conditionals and loops correct?\n4. Verify types: Are data types compatible?\n5. Check boundary conditions: What about edge cases?".to_string(),
            verification: "- Does the logic flow make sense?\n- Are variable types compatible?\n- Are edge cases handled?\n- Does the output match expected behavior?".to_string(),
        }
    }

    /// Prepend this template to a claim for better reasoning.
    pub fn prepend_to_claim(&self, claim: &str) -> String {
        format!(
            "{}\n\nGuidance:\n{}\n\nClaim to evaluate:\n{}",
            self.system, self.guidance, claim
        )
    }

    /// Get the verification checklist for this domain.
    pub fn get_verification_checklist(&self) -> &str {
        &self.verification
    }
}

/// Create a structured reasoning prompt for a domain.
pub fn create_reasoning_prompt(domain: Domain, claim: &str) -> String {
    let template = PromptTemplate::for_domain(domain);
    template.prepend_to_claim(claim)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_domains_have_templates() {
        for domain in &[
            Domain::Medical,
            Domain::Legal,
            Domain::Finance,
            Domain::Science,
            Domain::Code,
            Domain::General,
        ] {
            let template = PromptTemplate::for_domain(*domain);
            assert!(
                !template.system.is_empty(),
                "System prompt missing for {:?}",
                domain
            );
            assert!(
                !template.guidance.is_empty(),
                "Guidance missing for {:?}",
                domain
            );
            assert!(
                !template.verification.is_empty(),
                "Verification checklist missing for {:?}",
                domain
            );
        }
    }

    #[test]
    fn test_medical_template_contains_key_terms() {
        let template = PromptTemplate::medical();
        assert!(template.system.to_lowercase().contains("medical"));
        assert!(template.guidance.to_lowercase().contains("mechanism"));
        assert!(template.guidance.to_lowercase().contains("dose"));
    }

    #[test]
    fn test_legal_template_contains_key_terms() {
        let template = PromptTemplate::legal();
        assert!(template.system.to_lowercase().contains("legal"));
        assert!(template.guidance.to_lowercase().contains("precedent"));
        assert!(template.guidance.to_lowercase().contains("argument"));
    }

    #[test]
    fn test_prepend_to_claim() {
        let claim = "Aspirin prevents heart attacks.";
        let prompt = create_reasoning_prompt(Domain::Medical, claim);
        assert!(prompt.contains("medical"));
        assert!(prompt.contains(claim));
        assert!(prompt.contains("Guidance"));
    }

    #[test]
    fn test_verification_checklist_contains_items() {
        let medical = PromptTemplate::medical();
        let checklist = medical.get_verification_checklist();
        assert!(checklist.contains("-"));
        assert!(checklist.contains("?"));
    }

    #[test]
    fn test_domains_have_different_templates() {
        let medical = PromptTemplate::medical();
        let legal = PromptTemplate::legal();
        assert_ne!(medical.system, legal.system);
        assert_ne!(medical.guidance, legal.guidance);
    }

    #[test]
    fn test_code_template_emphasizes_logic() {
        let template = PromptTemplate::code();
        assert!(template.guidance.to_lowercase().contains("logic"));
        assert!(template.guidance.to_lowercase().contains("execution"));
    }

    #[test]
    fn test_science_template_emphasizes_testability() {
        let template = PromptTemplate::science();
        assert!(template.guidance.to_lowercase().contains("testable"));
        assert!(template.guidance.to_lowercase().contains("evidence"));
    }

    #[test]
    fn test_finance_template_emphasizes_risk() {
        let template = PromptTemplate::finance();
        assert!(template.guidance.to_lowercase().contains("risk"));
        assert!(template.guidance.to_lowercase().contains("data"));
    }
}
