//! # Certify Command (S-III-7)
//!
//! Generate or verify a content-addressed `ValidationCertificate` for any text.
//!
//! ```
//! pure-reason certify "God exists"
//! pure-reason certify --hash 9f8e3a2c1d4b5e6f --verify "God exists"
//! ```

use anyhow::Result;
use clap::Args;
use pure_reason_core::{
    certificate::{blake3_hex, ValidationCertificate},
    pipeline::KantianPipeline,
};

// ─── CertifyCmd ──────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct CertifyCmd {
    /// Text to certify (reads from stdin if omitted)
    pub text: Option<String>,

    /// Verify a given hash against the input text (prints VALID or INVALID)
    #[arg(long, value_name = "HASH")]
    pub verify: Option<String>,

    /// Only print the hash (suitable for scripting)
    #[arg(long)]
    pub hash_only: bool,
}

impl CertifyCmd {
    pub async fn run(&self, format: &str) -> Result<()> {
        let input = self.resolve_input()?;

        // Verify mode: check an existing hash without running the pipeline
        if let Some(expected_hash) = &self.verify {
            let matches = ValidationCertificate::verify(&input, expected_hash);
            if format == "json" {
                println!(
                    "{}",
                    serde_json::json!({
                        "valid": matches,
                        "expected_hash": expected_hash,
                        "computed_hash": blake3_hex(&input),
                    })
                );
            } else if matches {
                println!("✓ VALID — hash matches input");
            } else {
                println!("✗ INVALID — hash does not match input");
                println!("  Expected: {}", expected_hash);
                println!("  Computed: {}", blake3_hex(&input));
            }
            return Ok(());
        }

        // Hash-only mode: just print the BLAKE3 fingerprint
        if self.hash_only {
            println!("{}", blake3_hex(&input));
            return Ok(());
        }

        // Full certification: run the pipeline and build the certificate
        let pipeline = KantianPipeline::new();
        let report = pipeline.process(&input)?;
        let cert = ValidationCertificate::from_report(&report);

        match format {
            "json" => println!("{}", serde_json::to_string_pretty(&cert)?),
            "markdown" => {
                println!("# PureReason Validation Certificate\n");
                println!("| Field | Value |");
                println!("|-------|-------|");
                println!("| Hash | `{}` |", cert.content_hash);
                println!("| Issued | {} |", cert.issued_at);
                println!("| Version | {} |", cert.validator_version);
                println!("| Risk | **{}** |", cert.risk_level);
                println!("| Regulated | {} |", cert.regulated);
                if let Some(cat) = &cert.dominant_category {
                    println!("| Category | {} |", cat);
                }
                if cert.issues.is_empty() {
                    println!("| Issues | none |");
                } else {
                    println!("| Issues | {} |", cert.issues.join(", "));
                }
                println!("\n```\n{}\n```", cert.verify_hint);
            }
            _ => print!("{}", cert.display()),
        }

        Ok(())
    }

    fn resolve_input(&self) -> Result<String> {
        if let Some(text) = &self.text {
            return Ok(text.clone());
        }
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        Ok(buf.trim().to_string())
    }
}
