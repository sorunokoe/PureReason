//! # Structured Intuition (TRIZ S-8)
//!
//! Extends the Aesthetic layer to handle structured inputs (JSON, TOML, CSV)
//! in addition to free text. Maps structured fields to Kantian intuition forms:
//!
//! - **Space** = structural/relational form (object graph, field relationships)
//! - **Time** = sequential/temporal form (arrays, ordered data, event logs)
//!
//! This enables PureReason to validate tool-using agents, not just text generators.
//! Tool call outputs (JSON) are treated as structured intuitions and merged into
//! the WorldModel as structured object attribute updates.
//!
//! ## TRIZ Rationale
//! **S-8 (TC-1):**  
//! Generality: the system now handles any data type, not just text.
//! The Transition to Another Dimension principle (P17) — adding structure to the
//! input space (from 1D text to multi-dimensional data).

use crate::analytic::Category;
use serde::{Deserialize, Serialize};

// ─── StructuredField ─────────────────────────────────────────────────────────

/// A single field from a structured input, annotated with Kantian category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredField {
    pub key: String,
    pub value: String,
    pub inferred_category: Category,
    pub is_temporal: bool,
    pub is_causal: bool,
}

// ─── StructuredIntuition ─────────────────────────────────────────────────────

/// The result of parsing a structured input through the Aesthetic layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredIntuition {
    /// Source format detected.
    pub format: StructuredFormat,
    /// All fields parsed from the input.
    pub fields: Vec<StructuredField>,
    /// A flat text representation for use with the standard pipeline.
    pub text_projection: String,
    /// Estimated spatial complexity (depth of nesting).
    pub spatial_depth: usize,
    /// Whether any temporal ordering was detected.
    pub has_temporal_sequence: bool,
}

/// The detected format of the structured input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StructuredFormat {
    Json,
    Csv,
    KeyValue,
    PlainText,
}

// ─── StructuredIntuitionParser ────────────────────────────────────────────────

/// Parses structured inputs and maps them to Kantian intuition forms.
pub struct StructuredIntuitionParser;

impl StructuredIntuitionParser {
    /// Parse a string input — auto-detecting JSON, CSV, or falling back to plain text.
    pub fn parse(input: &str) -> StructuredIntuition {
        let trimmed = input.trim();
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            Self::parse_json(trimmed)
        } else if trimmed.contains('\n') && trimmed.contains(',') {
            Self::parse_csv(trimmed)
        } else if trimmed.contains(':') && !trimmed.contains(". ") {
            Self::parse_key_value(trimmed)
        } else {
            Self::parse_plain(trimmed)
        }
    }

    fn parse_json(input: &str) -> StructuredIntuition {
        match serde_json::from_str::<serde_json::Value>(input) {
            Ok(val) => {
                let mut fields = Vec::new();
                let mut spatial_depth = 0usize;
                flatten_json(&val, "", &mut fields, &mut spatial_depth);
                let has_temporal = fields.iter().any(|f| f.is_temporal);
                let text = fields
                    .iter()
                    .map(|f| format!("{} is {}", f.key, f.value))
                    .collect::<Vec<_>>()
                    .join(". ");
                StructuredIntuition {
                    format: StructuredFormat::Json,
                    fields,
                    text_projection: text,
                    spatial_depth,
                    has_temporal_sequence: has_temporal,
                }
            }
            Err(_) => Self::parse_plain(input),
        }
    }

    fn parse_csv(input: &str) -> StructuredIntuition {
        let mut lines = input.lines();
        let headers: Vec<&str> = lines
            .next()
            .unwrap_or("")
            .split(',')
            .map(|h| h.trim())
            .collect();
        let mut fields = Vec::new();
        for line in lines {
            let values: Vec<&str> = line.split(',').map(|v| v.trim()).collect();
            for (i, val) in values.iter().enumerate() {
                let key = headers.get(i).copied().unwrap_or("field");
                fields.push(annotate_field(key, val));
            }
        }
        let has_temporal = fields.iter().any(|f| f.is_temporal);
        let text = fields
            .iter()
            .map(|f| format!("{} is {}", f.key, f.value))
            .collect::<Vec<_>>()
            .join(". ");
        StructuredIntuition {
            format: StructuredFormat::Csv,
            fields,
            text_projection: text,
            spatial_depth: 1,
            has_temporal_sequence: has_temporal,
        }
    }

    fn parse_key_value(input: &str) -> StructuredIntuition {
        let mut fields = Vec::new();
        for line in input.lines() {
            if let Some(pos) = line.find(':') {
                let key = line[..pos].trim();
                let val = line[pos + 1..].trim();
                if !key.is_empty() && !val.is_empty() {
                    fields.push(annotate_field(key, val));
                }
            }
        }
        let has_temporal = fields.iter().any(|f| f.is_temporal);
        let text = fields
            .iter()
            .map(|f| format!("{} is {}", f.key, f.value))
            .collect::<Vec<_>>()
            .join(". ");
        StructuredIntuition {
            format: StructuredFormat::KeyValue,
            fields,
            text_projection: text,
            spatial_depth: 1,
            has_temporal_sequence: has_temporal,
        }
    }

    fn parse_plain(input: &str) -> StructuredIntuition {
        // Fall back to treating the whole string as one field
        let field = annotate_field("text", input);
        let text = input.to_string();
        StructuredIntuition {
            format: StructuredFormat::PlainText,
            fields: vec![field],
            text_projection: text,
            spatial_depth: 0,
            has_temporal_sequence: false,
        }
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn flatten_json(
    val: &serde_json::Value,
    prefix: &str,
    fields: &mut Vec<StructuredField>,
    depth: &mut usize,
) {
    match val {
        serde_json::Value::Object(map) => {
            *depth = (*depth).max(1 + prefix.chars().filter(|c| *c == '.').count());
            for (k, v) in map {
                let key = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{}.{}", prefix, k)
                };
                flatten_json(v, &key, fields, depth);
            }
        }
        serde_json::Value::Array(arr) => {
            *depth = (*depth).max(1 + prefix.chars().filter(|c| *c == '.').count());
            for (i, v) in arr.iter().enumerate() {
                let key = format!("{}[{}]", prefix, i);
                flatten_json(v, &key, fields, depth);
            }
        }
        other => {
            let val_str = match other {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Null => "null".to_string(),
                v => v.to_string(),
            };
            fields.push(annotate_field(prefix, &val_str));
        }
    }
}

fn annotate_field(key: &str, value: &str) -> StructuredField {
    let key_lower = key.to_lowercase();
    let val_lower = value.to_lowercase();

    // Temporal detection
    let is_temporal = key_lower.contains("time")
        || key_lower.contains("date")
        || key_lower.contains("when")
        || key_lower.contains("at")
        || key_lower.contains("start")
        || key_lower.contains("end")
        || val_lower.contains("ago")
        || val_lower.contains("yesterday")
        || val_lower.contains("tomorrow")
        || val_lower.contains("2024")
        || val_lower.contains("2025")
        || val_lower.contains("2026");

    // Causal detection
    let is_causal = key_lower.contains("cause")
        || key_lower.contains("effect")
        || key_lower.contains("result")
        || key_lower.contains("reason")
        || val_lower.contains("causes")
        || val_lower.contains("leads to")
        || val_lower.contains("because")
        || val_lower.contains("therefore");

    // Category inference from field name
    let inferred_category = if is_causal {
        Category::Causality
    } else if is_temporal {
        Category::Existence
    } else if key_lower.contains("count") || key_lower.contains("num") || key_lower.contains("qty")
    {
        Category::Plurality
    } else if key_lower.contains("id") || key_lower.contains("name") || key_lower.contains("type") {
        Category::Substance
    } else if key_lower.contains("possible") || key_lower.contains("optional") {
        Category::Possibility
    } else if key_lower.contains("required") || key_lower.contains("mandatory") {
        Category::Necessity
    } else {
        Category::Reality
    };

    StructuredField {
        key: key.to_string(),
        value: value.to_string(),
        inferred_category,
        is_temporal,
        is_causal,
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_json_extracts_fields() {
        let input = r#"{"name": "Alice", "age": 30, "active": true}"#;
        let intuition = StructuredIntuitionParser::parse(input);
        assert_eq!(intuition.format, StructuredFormat::Json);
        assert_eq!(intuition.fields.len(), 3);
        assert!(!intuition.text_projection.is_empty());
    }

    #[test]
    fn parse_json_with_timestamp_detects_temporal() {
        let input = r#"{"timestamp": "2025-01-01", "event": "start"}"#;
        let intuition = StructuredIntuitionParser::parse(input);
        assert!(intuition.has_temporal_sequence);
    }

    #[test]
    fn parse_csv_extracts_rows() {
        let input = "name,age,city\nAlice,30,Paris\nBob,25,London";
        let intuition = StructuredIntuitionParser::parse(input);
        assert_eq!(intuition.format, StructuredFormat::Csv);
        assert!(!intuition.fields.is_empty());
    }

    #[test]
    fn parse_key_value_extracts_pairs() {
        let input = "name: Alice\nage: 30\ncity: Paris";
        let intuition = StructuredIntuitionParser::parse(input);
        assert_eq!(intuition.format, StructuredFormat::KeyValue);
        assert_eq!(intuition.fields.len(), 3);
    }

    #[test]
    fn parse_plain_text_falls_back() {
        let input = "This is plain text without structure.";
        let intuition = StructuredIntuitionParser::parse(input);
        assert_eq!(intuition.format, StructuredFormat::PlainText);
    }

    #[test]
    fn causal_field_infers_causality_category() {
        let field = annotate_field("cause", "fire");
        assert_eq!(field.inferred_category, Category::Causality);
    }

    #[test]
    fn text_projection_is_usable_as_pipeline_input() {
        let input = r#"{"temperature": 100, "state": "boiling"}"#;
        let intuition = StructuredIntuitionParser::parse(input);
        assert!(intuition.text_projection.contains("temperature"));
        assert!(intuition.text_projection.contains("boiling"));
    }
}
