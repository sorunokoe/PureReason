//! Symbolic arithmetic rules — unit conversions and inequality checks.
//!
//! Provides deterministic verification for simple numeric claims:
//! - Unit conversions (miles → km, °F → °C, etc.)
//! - Order-of-magnitude sanity checks
//!
//! No floating-point arithmetic happens at pipeline time; all values are
//! pre-computed `f64` constants.

use serde::Serialize;

/// A unit conversion rule.
#[derive(Debug, Clone, Serialize)]
pub struct UnitConversion {
    /// Stable identifier.
    pub id: &'static str,
    /// Human-readable description.
    pub description: &'static str,
    /// Source unit name.
    pub from_unit: &'static str,
    /// Target unit name.
    pub to_unit: &'static str,
    /// Conversion factor: `to_value = from_value * factor + offset`.
    pub factor: f64,
    /// Additive offset (used for temperature conversions).
    pub offset: f64,
    /// Signals that indicate this conversion is relevant.
    pub signals: &'static [&'static str],
}

impl UnitConversion {
    /// Convert a value from the source unit to the target unit.
    #[inline]
    pub fn convert(&self, value: f64) -> f64 {
        value * self.factor + self.offset
    }

    /// Check if a claimed `to_value` is approximately equal to converting `from_value`.
    ///
    /// `tolerance` is fractional (e.g., 0.02 = 2% tolerance).
    pub fn is_correct(&self, from_value: f64, claimed_to_value: f64, tolerance: f64) -> bool {
        let expected = self.convert(from_value);
        if expected == 0.0 {
            return claimed_to_value.abs() < tolerance;
        }
        ((claimed_to_value - expected) / expected).abs() <= tolerance
    }
}

/// The compiled unit conversion atlas.
pub static UNIT_CONVERSIONS: &[UnitConversion] = &[
    // ── Length ────────────────────────────────────────────────────────────────
    UnitConversion {
        id: "miles_to_km",
        description: "Statute miles to kilometres",
        from_unit: "miles",
        to_unit: "km",
        factor: 1.60934,
        offset: 0.0,
        signals: &[
            "miles to km",
            "miles to kilometres",
            "miles in km",
            "convert miles",
        ],
    },
    UnitConversion {
        id: "km_to_miles",
        description: "Kilometres to statute miles",
        from_unit: "km",
        to_unit: "miles",
        factor: 0.621371,
        offset: 0.0,
        signals: &["km to miles", "kilometres to miles", "km in miles"],
    },
    UnitConversion {
        id: "feet_to_meters",
        description: "Feet to metres",
        from_unit: "feet",
        to_unit: "m",
        factor: 0.3048,
        offset: 0.0,
        signals: &[
            "feet to meters",
            "feet to metres",
            "feet in meters",
            "ft to m",
        ],
    },
    UnitConversion {
        id: "inches_to_cm",
        description: "Inches to centimetres",
        from_unit: "inches",
        to_unit: "cm",
        factor: 2.54,
        offset: 0.0,
        signals: &["inches to cm", "inch to centimeter", "inch in cm"],
    },
    // ── Temperature ───────────────────────────────────────────────────────────
    UnitConversion {
        id: "fahrenheit_to_celsius",
        description: "Fahrenheit to Celsius",
        from_unit: "°F",
        to_unit: "°C",
        factor: 5.0 / 9.0,
        offset: -32.0 * (5.0 / 9.0),
        signals: &[
            "fahrenheit to celsius",
            "f to c",
            "degrees f to c",
            "convert fahrenheit",
        ],
    },
    UnitConversion {
        id: "celsius_to_fahrenheit",
        description: "Celsius to Fahrenheit",
        from_unit: "°C",
        to_unit: "°F",
        factor: 9.0 / 5.0,
        offset: 32.0,
        signals: &[
            "celsius to fahrenheit",
            "c to f",
            "degrees c to f",
            "convert celsius",
        ],
    },
    UnitConversion {
        id: "celsius_to_kelvin",
        description: "Celsius to Kelvin",
        from_unit: "°C",
        to_unit: "K",
        factor: 1.0,
        offset: 273.15,
        signals: &["celsius to kelvin", "c to k", "degrees celsius to kelvin"],
    },
    // ── Weight / mass ─────────────────────────────────────────────────────────
    UnitConversion {
        id: "pounds_to_kg",
        description: "Pounds (avoirdupois) to kilograms",
        from_unit: "lb",
        to_unit: "kg",
        factor: 0.453592,
        offset: 0.0,
        signals: &[
            "pounds to kg",
            "lbs to kg",
            "pounds to kilograms",
            "lb to kg",
        ],
    },
    UnitConversion {
        id: "kg_to_pounds",
        description: "Kilograms to pounds",
        from_unit: "kg",
        to_unit: "lb",
        factor: 2.20462,
        offset: 0.0,
        signals: &["kg to pounds", "kg to lbs", "kilograms to pounds"],
    },
    UnitConversion {
        id: "ounces_to_grams",
        description: "Ounces (avoirdupois) to grams",
        from_unit: "oz",
        to_unit: "g",
        factor: 28.3495,
        offset: 0.0,
        signals: &["ounces to grams", "oz to g", "oz to grams"],
    },
    // ── Volume ────────────────────────────────────────────────────────────────
    UnitConversion {
        id: "gallons_us_to_liters",
        description: "US gallons to litres",
        from_unit: "US gal",
        to_unit: "L",
        factor: 3.78541,
        offset: 0.0,
        signals: &[
            "gallons to liters",
            "gallons to litres",
            "us gallon to liter",
        ],
    },
    UnitConversion {
        id: "liters_to_gallons_us",
        description: "Litres to US gallons",
        from_unit: "L",
        to_unit: "US gal",
        factor: 0.264172,
        offset: 0.0,
        signals: &["liters to gallons", "litres to gallons", "litre to gallon"],
    },
    // ── Energy / Power ────────────────────────────────────────────────────────
    UnitConversion {
        id: "calories_to_joules",
        description: "Thermochemical calories to joules",
        from_unit: "cal",
        to_unit: "J",
        factor: 4.184,
        offset: 0.0,
        signals: &["calories to joules", "cal to j", "calorie to joule"],
    },
    UnitConversion {
        id: "kwh_to_joules",
        description: "Kilowatt-hours to joules",
        from_unit: "kWh",
        to_unit: "J",
        factor: 3_600_000.0,
        offset: 0.0,
        signals: &["kwh to joules", "kilowatt hour to joule", "kWh to J"],
    },
];

/// Look up a unit conversion rule by its stable `id`.
pub fn check_unit_conversion(id: &str) -> Option<&'static UnitConversion> {
    UNIT_CONVERSIONS.iter().find(|c| c.id == id)
}

/// Find relevant conversions for a given text.
pub fn find_relevant_conversions(text: &str) -> Vec<&'static UnitConversion> {
    let lower = text.to_lowercase();
    UNIT_CONVERSIONS
        .iter()
        .filter(|c| c.signals.iter().any(|s| lower.contains(*s)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn miles_to_km_conversion() {
        let conv = check_unit_conversion("miles_to_km").unwrap();
        let km = conv.convert(1.0);
        assert!((km - 1.60934).abs() < 0.001);
    }

    #[test]
    fn fahrenheit_to_celsius_boiling() {
        let conv = check_unit_conversion("fahrenheit_to_celsius").unwrap();
        let c = conv.convert(212.0);
        assert!((c - 100.0).abs() < 0.01, "212°F should be 100°C, got {}", c);
    }

    #[test]
    fn celsius_to_kelvin_absolute_zero() {
        let conv = check_unit_conversion("celsius_to_kelvin").unwrap();
        let k = conv.convert(-273.15);
        assert!(k.abs() < 0.01, "-273.15°C should be ~0K, got {}", k);
    }

    #[test]
    fn is_correct_within_tolerance() {
        let conv = check_unit_conversion("miles_to_km").unwrap();
        assert!(conv.is_correct(1.0, 1.609, 0.01));
        assert!(!conv.is_correct(1.0, 1.0, 0.01)); // forgot to convert
    }

    #[test]
    fn find_relevant_returns_matches() {
        let convs = find_relevant_conversions("convert miles to km");
        assert!(!convs.is_empty());
        assert!(convs.iter().any(|c| c.id == "miles_to_km"));
    }

    #[test]
    fn all_conversions_have_signals() {
        for c in UNIT_CONVERSIONS {
            assert!(
                !c.signals.is_empty(),
                "conversion '{}' has no signals",
                c.id
            );
        }
    }
}
