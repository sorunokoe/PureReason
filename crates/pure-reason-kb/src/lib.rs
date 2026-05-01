//! # PureReason Knowledge Base (TRIZ Report IX — S9)
//!
//! A compiled, zero-cost knowledge base for deterministic fact-checking.
//! All data is `&'static` — no heap allocation on the hot path.
//!
//! ## Segments
//!
//! - **[constants]**: Physical and mathematical constants with plausibility bounds.
//!   Used by the Numeric Plausibility Detector to flag impossible values.
//!
//! - **[entities]**: Curated entity–fact pairs (capitals, dates, counts).
//!   Used by the Knowledge-Grounded Answering Check (KAC).
//!
//! - **[symbolic]**: Symbolic arithmetic rules (unit conversions, inequalities).
//!   Used to verify simple calculations without an LLM.
//!
//! ## Design principles
//!
//! - All data is `&'static str` / `f64` — zero heap allocation at query time.
//! - Lookup is O(n) linear scan — acceptable for atlas sizes ≤ 10k entries.
//! - The crate is the single authoritative source for numeric bounds used across
//!   all pipeline layers; changes here propagate automatically.

pub mod constants;
pub mod entities;
pub mod symbolic;

// Re-export the most common query functions at crate root.
pub use constants::{lookup_constant, ConstantBounds, PhysicalConstant};
pub use entities::{lookup_entity_fact, EntityFact};
pub use symbolic::{check_unit_conversion, UnitConversion};
