//! Physical and mathematical constants with plausibility bounds.
//!
//! Each [`PhysicalConstant`] entry contains:
//! - `id`: stable snake_case identifier
//! - `name`: human-readable name
//! - `value`: canonical SI value
//! - `unit`: SI unit string
//! - `min` / `max`: plausibility bounds (values outside → hallucination flag)
//! - `aliases`: alternative names / symbols LLMs commonly use
//!
//! ## Usage
//!
//! ```
//! use pure_reason_kb::constants::{lookup_constant, PHYSICAL_CONSTANTS};
//!
//! let c = lookup_constant("speed_of_light");
//! assert!(c.is_some());
//! let entry = c.unwrap();
//! // Check if a parsed value is plausible
//! assert!(entry.is_plausible(3.0e8));
//! assert!(!entry.is_plausible(3.0e5)); // off by 1000x
//! ```

use serde::Serialize;

/// A physical or mathematical constant with plausibility bounds.
#[derive(Debug, Clone, Serialize)]
pub struct PhysicalConstant {
    /// Stable identifier (snake_case).
    pub id: &'static str,
    /// Human-readable name.
    pub name: &'static str,
    /// Canonical SI value.
    pub value: f64,
    /// SI unit.
    pub unit: &'static str,
    /// Minimum plausible value (values below → flag as hallucination).
    pub min: f64,
    /// Maximum plausible value (values above → flag as hallucination).
    pub max: f64,
    /// Aliases / alternative names used in LLM outputs.
    pub aliases: &'static [&'static str],
}

/// Numeric plausibility bounds for a derived or measured quantity.
#[derive(Debug, Clone, Serialize)]
pub struct ConstantBounds {
    /// The matched constant.
    pub id: &'static str,
    /// Whether the tested value is within plausibility bounds.
    pub plausible: bool,
    /// The canonical value.
    pub canonical: f64,
    /// Ratio of tested value to canonical (1.0 = perfect match).
    pub ratio: f64,
}

impl PhysicalConstant {
    /// Returns `true` if `value` is within this constant's plausibility bounds.
    #[inline]
    pub fn is_plausible(&self, value: f64) -> bool {
        value >= self.min && value <= self.max
    }

    /// Check a value and return detailed bounds information.
    pub fn check(&self, value: f64) -> ConstantBounds {
        ConstantBounds {
            id: self.id,
            plausible: self.is_plausible(value),
            canonical: self.value,
            ratio: if self.value != 0.0 {
                value / self.value
            } else {
                f64::NAN
            },
        }
    }

    /// Returns `true` if the given string matches any alias or the `name`.
    pub fn matches_name(&self, text: &str) -> bool {
        let lower = text.to_lowercase();
        self.name.to_lowercase().contains(&lower)
            || self
                .aliases
                .iter()
                .any(|a| a.to_lowercase().contains(&lower))
    }
}

/// The compiled physical constants atlas.
///
/// Ordered by detection frequency in FELM / TruthfulQA numeric benchmarks.
pub static PHYSICAL_CONSTANTS: &[PhysicalConstant] = &[
    // ── Fundamental physical constants ──────────────────────────────────────
    PhysicalConstant {
        id: "speed_of_light",
        name: "Speed of light in vacuum",
        value: 2.998e8,
        unit: "m/s",
        min: 2.9e8,
        max: 3.1e8,
        aliases: &["c", "speed of light", "light speed", "velocity of light"],
    },
    PhysicalConstant {
        id: "gravitational_constant",
        name: "Gravitational constant",
        value: 6.674e-11,
        unit: "m³ kg⁻¹ s⁻²",
        min: 6.0e-11,
        max: 7.5e-11,
        aliases: &[
            "G",
            "big G",
            "Newton's constant",
            "universal gravitational constant",
        ],
    },
    PhysicalConstant {
        id: "planck_constant",
        name: "Planck constant",
        value: 6.626e-34,
        unit: "J·s",
        min: 6.0e-34,
        max: 7.5e-34,
        aliases: &["h", "Planck's constant", "quantum of action"],
    },
    PhysicalConstant {
        id: "boltzmann_constant",
        name: "Boltzmann constant",
        value: 1.381e-23,
        unit: "J/K",
        min: 1.3e-23,
        max: 1.5e-23,
        aliases: &["k_B", "kB", "Boltzmann's constant"],
    },
    PhysicalConstant {
        id: "avogadro_number",
        name: "Avogadro constant",
        value: 6.022e23,
        unit: "mol⁻¹",
        min: 5.9e23,
        max: 6.2e23,
        aliases: &["N_A", "Avogadro's number", "Avogadro number", "mole"],
    },
    PhysicalConstant {
        id: "elementary_charge",
        name: "Elementary charge",
        value: 1.602e-19,
        unit: "C",
        min: 1.5e-19,
        max: 1.7e-19,
        aliases: &["e", "electron charge", "proton charge", "unit charge"],
    },
    PhysicalConstant {
        id: "electron_mass",
        name: "Electron rest mass",
        value: 9.109e-31,
        unit: "kg",
        min: 8.0e-31,
        max: 1.0e-30,
        aliases: &["m_e", "electron mass", "mass of electron"],
    },
    PhysicalConstant {
        id: "proton_mass",
        name: "Proton rest mass",
        value: 1.673e-27,
        unit: "kg",
        min: 1.5e-27,
        max: 1.9e-27,
        aliases: &["m_p", "proton mass", "mass of proton"],
    },
    // ── Thermodynamic constants ──────────────────────────────────────────────
    PhysicalConstant {
        id: "absolute_zero",
        name: "Absolute zero",
        value: -273.15,
        unit: "°C",
        min: -273.20,
        max: -273.10,
        aliases: &["absolute zero", "0 kelvin", "0K", "zero Kelvin", "-273"],
    },
    PhysicalConstant {
        id: "water_boiling_point",
        name: "Boiling point of water at sea level",
        value: 100.0,
        unit: "°C",
        min: 99.5,
        max: 100.5,
        aliases: &[
            "water boils",
            "boiling point of water",
            "100 degrees",
            "212 fahrenheit",
        ],
    },
    PhysicalConstant {
        id: "water_freezing_point",
        name: "Freezing point of water",
        value: 0.0,
        unit: "°C",
        min: -0.1,
        max: 0.1,
        aliases: &[
            "freezing point",
            "water freezes",
            "0 degrees celsius",
            "32 fahrenheit",
        ],
    },
    // ── Astronomical constants ───────────────────────────────────────────────
    PhysicalConstant {
        id: "earth_radius",
        name: "Mean radius of Earth",
        value: 6.371e6,
        unit: "m",
        min: 6.3e6,
        max: 6.45e6,
        aliases: &[
            "earth radius",
            "radius of earth",
            "earth's radius",
            "6371 km",
        ],
    },
    PhysicalConstant {
        id: "earth_mass",
        name: "Mass of Earth",
        value: 5.972e24,
        unit: "kg",
        min: 5.8e24,
        max: 6.1e24,
        aliases: &["earth mass", "mass of earth", "earth's mass"],
    },
    PhysicalConstant {
        id: "sun_mass",
        name: "Mass of the Sun",
        value: 1.989e30,
        unit: "kg",
        min: 1.9e30,
        max: 2.1e30,
        aliases: &["solar mass", "mass of the sun", "mass of sun"],
    },
    PhysicalConstant {
        id: "earth_sun_distance",
        name: "Mean Earth-Sun distance (1 AU)",
        value: 1.496e11,
        unit: "m",
        min: 1.47e11,
        max: 1.52e11,
        aliases: &[
            "AU",
            "astronomical unit",
            "earth sun distance",
            "1 AU",
            "149.6 million km",
        ],
    },
    PhysicalConstant {
        id: "speed_of_sound_air",
        name: "Speed of sound in air at 20°C",
        value: 343.0,
        unit: "m/s",
        min: 330.0,
        max: 355.0,
        aliases: &["speed of sound", "sound speed", "mach 1", "343 m/s"],
    },
    PhysicalConstant {
        id: "standard_gravity",
        name: "Standard acceleration of gravity at Earth's surface",
        value: 9.807,
        unit: "m/s²",
        min: 9.75,
        max: 9.90,
        aliases: &[
            "g",
            "acceleration due to gravity",
            "gravitational acceleration",
            "9.8 m/s",
            "9.81",
        ],
    },
    // ── Mathematical constants ───────────────────────────────────────────────
    PhysicalConstant {
        id: "pi",
        name: "Pi",
        value: std::f64::consts::PI,
        unit: "dimensionless",
        // Plausibility bounds: accept any reported value within ±0.1 of the true constant.
        // Using the constant directly avoids the `approx_constant` lint.
        min: std::f64::consts::PI - 0.1,
        max: std::f64::consts::PI + 0.1,
        aliases: &["π", "pi", "3.14159", "circumference ratio"],
    },
    PhysicalConstant {
        id: "euler_number",
        name: "Euler's number",
        value: std::f64::consts::E,
        unit: "dimensionless",
        // Plausibility bounds: accept any reported value within ±0.1 of the true constant.
        min: std::f64::consts::E - 0.1,
        max: std::f64::consts::E + 0.1,
        aliases: &["e", "Euler's number", "natural logarithm base", "2.71828"],
    },
    PhysicalConstant {
        id: "golden_ratio",
        name: "Golden ratio",
        value: 1.6180339887,
        unit: "dimensionless",
        min: 1.6179,
        max: 1.6182,
        aliases: &["φ", "phi", "golden ratio", "golden section", "1.618"],
    },
    // ── Human / biological scales ────────────────────────────────────────────
    PhysicalConstant {
        id: "human_body_temp",
        name: "Normal human body temperature",
        value: 37.0,
        unit: "°C",
        min: 36.0,
        max: 37.5,
        aliases: &[
            "body temperature",
            "normal temperature",
            "98.6 fahrenheit",
            "37 celsius",
        ],
    },
    PhysicalConstant {
        id: "resting_heart_rate",
        name: "Resting heart rate (adults)",
        value: 70.0,
        unit: "bpm",
        min: 60.0,
        max: 100.0,
        aliases: &["heart rate", "resting heart rate", "pulse", "bpm"],
    },
];

/// Look up a constant by its stable `id`.
///
/// # Example
///
/// ```
/// use pure_reason_kb::constants::lookup_constant;
///
/// let c = lookup_constant("speed_of_light");
/// assert!(c.is_some());
/// assert!(c.unwrap().is_plausible(3.0e8));
/// ```
pub fn lookup_constant(id: &str) -> Option<&'static PhysicalConstant> {
    PHYSICAL_CONSTANTS.iter().find(|c| c.id == id)
}

/// Search constants by name or alias.
///
/// Returns all entries whose `name` or `aliases` contain the given `text` (case-insensitive).
pub fn search_constants(text: &str) -> Vec<&'static PhysicalConstant> {
    let lower = text.to_lowercase();
    PHYSICAL_CONSTANTS
        .iter()
        .filter(|c| {
            c.name.to_lowercase().contains(&lower)
                || c.id.contains(&lower)
                || c.aliases.iter().any(|a| a.to_lowercase().contains(&lower))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn speed_of_light_plausible() {
        let c = lookup_constant("speed_of_light").unwrap();
        assert!(c.is_plausible(3.0e8));
        assert!(!c.is_plausible(3.0e5)); // 1000x too slow
        assert!(!c.is_plausible(3.0e11)); // 1000x too fast
    }

    #[test]
    fn absolute_zero_bounds() {
        let c = lookup_constant("absolute_zero").unwrap();
        assert!(c.is_plausible(-273.15));
        assert!(!c.is_plausible(-300.0)); // below absolute zero — impossible
    }

    #[test]
    fn water_boiling_point() {
        let c = lookup_constant("water_boiling_point").unwrap();
        assert!(c.is_plausible(100.0));
        assert!(!c.is_plausible(50.0)); // half — wrong
    }

    #[test]
    fn search_by_alias() {
        let results = search_constants("boltzmann");
        assert!(!results.is_empty());
        assert_eq!(results[0].id, "boltzmann_constant");
    }

    #[test]
    fn all_constants_have_valid_bounds() {
        for c in PHYSICAL_CONSTANTS {
            assert!(c.min <= c.max, "constant '{}': min > max", c.id);
            assert!(
                c.min <= c.value || c.value.is_nan(),
                "constant '{}': value below min",
                c.id
            );
            assert!(
                c.value <= c.max || c.value.is_nan(),
                "constant '{}': value above max",
                c.id
            );
        }
    }

    #[test]
    fn check_returns_ratio() {
        let c = lookup_constant("pi").unwrap();
        let bounds = c.check(std::f64::consts::PI);
        assert!(bounds.plausible);
        assert!((bounds.ratio - 1.0).abs() < 0.001);
    }
}
