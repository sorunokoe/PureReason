//! # Numeric Plausibility Detector (TRIZ Report VIII — S3)
//!
//! Validates numeric claims against a static table of physics and biology
//! constants with order-of-magnitude envelopes (log₁₀ ± tolerance).
//!
//! ## Design principles (TRIZ IFR)
//!
//! The ideal function: detect implausible numeric assertions at zero marginal cost.
//! Implementation: one-pass regex-free scan using ASCII digit detection and a
//! 50-entry static constant table. No heap allocation on the hot path.
//!
//! ## Scope
//!
//! Only checks claims whose NanoType is `ClaimType::Numeric`.
//! Called by `diagnose_claim()` for numeric sentences; wired into `ClaimAnnotation::build()`.
//! Atlas expanded to ~100 entries in TRIZ Report XII (S26 Track 2):
//! universal physics, astronomy, chemistry, biology, geophysics, engineering, medicine, environment.
//!
//! ## False-positive control
//!
//! - Only fires when the parsed order-of-magnitude differs from the expected
//!   range by more than `tolerance` (log₁₀ scale, default 1.0).
//! - Approximate parsing: extracts the first numeric literal in the sentence;
//!   if no number is parseable, no flag is raised (fail-safe: no false positives
//!   over false negatives).

use serde::{Deserialize, Serialize};

// ─── Constant atlas entry ─────────────────────────────────────────────────────

/// A single physical/biological constant with an order-of-magnitude envelope.
///
/// `expected_log10` is the base-10 logarithm of the expected value.
/// `tolerance` defines the allowed deviation (log₁₀ units, default 1.5 = 1.5 orders of magnitude).
struct ConstantEntry {
    /// Keywords that trigger this check (ANY → try to validate).
    topic_signals: &'static [&'static str],
    /// Description used in issue messages.
    description: &'static str,
    /// log₁₀(expected value) — the central value.
    expected_log10: f64,
    /// Allowed deviation in log₁₀ units.
    /// 1.0 → value must be within 10× of expected.
    /// 1.5 → within ~32× of expected.
    /// 2.0 → within 100× of expected.
    tolerance: f64,
}

// ─── Static constant atlas (~100 entries) ──────────────────────────────────

static CONSTANT_ATLAS: &[ConstantEntry] = &[
    // ── Universal physical constants ─────────────────────────────────────────
    ConstantEntry {
        topic_signals: &["speed of light", "c =", "c=", "light speed", "299"],
        description: "Speed of light (~3×10⁸ m/s)",
        expected_log10: 8.477, // log10(3e8)
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &["planck constant", "planck's constant", "6.626", "6.63e"],
        description: "Planck constant (~6.626×10⁻³⁴ J·s)",
        expected_log10: -33.178,
        tolerance: 1.0,
    },
    ConstantEntry {
        topic_signals: &["gravitational constant", "big g", "newton's g", "6.674"],
        description: "Gravitational constant G (~6.674×10⁻¹¹ N·m²/kg²)",
        expected_log10: -10.176,
        tolerance: 1.0,
    },
    ConstantEntry {
        topic_signals: &["avogadro", "avogadro's number", "mole", "6.022"],
        description: "Avogadro's number (~6.022×10²³)",
        expected_log10: 23.779,
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &[
            "boltzmann constant",
            "boltzmann's constant",
            "1.38e",
            "1.381",
        ],
        description: "Boltzmann constant (~1.381×10⁻²³ J/K)",
        expected_log10: -22.860,
        tolerance: 1.0,
    },
    ConstantEntry {
        topic_signals: &[
            "electron charge",
            "elementary charge",
            "charge of electron",
            "1.602e",
        ],
        description: "Elementary charge (~1.602×10⁻¹⁹ C)",
        expected_log10: -18.795,
        tolerance: 1.0,
    },
    ConstantEntry {
        topic_signals: &["electron mass", "mass of electron", "9.109e"],
        description: "Electron mass (~9.109×10⁻³¹ kg)",
        expected_log10: -30.040,
        tolerance: 1.0,
    },
    ConstantEntry {
        topic_signals: &["proton mass", "mass of proton", "1.673e"],
        description: "Proton mass (~1.673×10⁻²⁷ kg)",
        expected_log10: -26.777,
        tolerance: 1.0,
    },
    // ── Astronomical constants ────────────────────────────────────────────────
    ConstantEntry {
        topic_signals: &[
            "astronomical unit",
            "au ",
            "earth-sun distance",
            "sun distance",
        ],
        description: "Astronomical unit (~1.496×10¹¹ m)",
        expected_log10: 11.175,
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &["light year", "light-year", "lightyear"],
        description: "Light-year (~9.461×10¹⁵ m)",
        expected_log10: 15.976,
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &["parsec", " pc ", "kiloparsec"],
        description: "Parsec (~3.086×10¹⁶ m)",
        expected_log10: 16.489,
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &["radius of earth", "earth's radius", "earth radius"],
        description: "Radius of Earth (~6.371×10⁶ m)",
        expected_log10: 6.804,
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &["mass of earth", "earth's mass", "earth mass"],
        description: "Mass of Earth (~5.972×10²⁴ kg)",
        expected_log10: 24.776,
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &["radius of sun", "sun's radius", "solar radius"],
        description: "Radius of the Sun (~6.957×10⁸ m)",
        expected_log10: 8.842,
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &["mass of sun", "sun's mass", "solar mass"],
        description: "Mass of the Sun (~1.989×10³⁰ kg)",
        expected_log10: 30.299,
        tolerance: 0.5,
    },
    // ── Chemistry ─────────────────────────────────────────────────────────────
    ConstantEntry {
        topic_signals: &[
            "boiling point of water",
            "water boils",
            "100 degrees",
            "100°c",
        ],
        description: "Boiling point of water (100 °C at 1 atm)",
        expected_log10: 2.0, // log10(100)
        tolerance: 0.3,
    },
    ConstantEntry {
        topic_signals: &[
            "melting point of iron",
            "iron melts",
            "iron melting",
            "iron's melting",
        ],
        description: "Melting point of iron (~1538 °C)",
        expected_log10: 3.187,
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &["melting point of gold", "gold melts", "gold melting"],
        description: "Melting point of gold (~1064 °C)",
        expected_log10: 3.027,
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &[
            "atomic number of carbon",
            "carbon atomic",
            "atomic weight of carbon",
        ],
        description: "Atomic number of carbon (6) / atomic weight (~12.011 u)",
        expected_log10: 1.079, // log10(12)
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &["atomic number of uranium", "uranium atomic number"],
        description: "Atomic number of uranium (92)",
        expected_log10: 1.964,
        tolerance: 0.3,
    },
    // ── Biology ───────────────────────────────────────────────────────────────
    ConstantEntry {
        topic_signals: &[
            "human body temperature",
            "normal body temperature",
            "body temperature",
            "98.6",
        ],
        description: "Human body temperature (~37 °C / 98.6 °F)",
        expected_log10: 1.568, // log10(37)
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &[
            "human heart rate",
            "resting heart rate",
            "beats per minute",
            "bpm",
        ],
        description: "Human resting heart rate (60–100 bpm)",
        expected_log10: 1.875, // log10(75)
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &["human lifespan", "average lifespan", "life expectancy"],
        description: "Human average lifespan (~73 years globally)",
        expected_log10: 1.863,
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &["dna length", "length of human dna", "human genome"],
        description: "Human genome size (~3.2×10⁹ base pairs)",
        expected_log10: 9.505,
        tolerance: 0.7,
    },
    ConstantEntry {
        topic_signals: &["number of cells", "human body cells", "cells in human body"],
        description: "Number of cells in the human body (~3.7×10¹³)",
        expected_log10: 13.568,
        tolerance: 1.0,
    },
    ConstantEntry {
        topic_signals: &["neurons in brain", "brain cells", "number of neurons"],
        description: "Number of neurons in the human brain (~86×10⁹)",
        expected_log10: 10.934,
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &[
            "height of average human",
            "average human height",
            "average height",
        ],
        description: "Average adult height (~1.7 m)",
        expected_log10: 0.230, // log10(1.7)
        tolerance: 0.5,
    },
    // ── Geophysics ────────────────────────────────────────────────────────────
    ConstantEntry {
        topic_signals: &[
            "age of earth",
            "earth is",
            "earth formed",
            "billion years old",
            "earth's age",
        ],
        description: "Age of Earth (~4.54×10⁹ years)",
        expected_log10: 9.657,
        tolerance: 0.3,
    },
    ConstantEntry {
        topic_signals: &[
            "age of universe",
            "universe is",
            "universe formed",
            "big bang",
        ],
        description: "Age of the universe (~1.38×10¹⁰ years)",
        expected_log10: 10.140,
        tolerance: 0.3,
    },
    ConstantEntry {
        topic_signals: &[
            "depth of mariana trench",
            "deepest ocean",
            "mariana trench depth",
        ],
        description: "Depth of Mariana Trench (~11,000 m)",
        expected_log10: 4.041,
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &[
            "height of mount everest",
            "everest height",
            "tallest mountain",
        ],
        description: "Height of Mt Everest (~8,849 m)",
        expected_log10: 3.947,
        tolerance: 0.3,
    },
    ConstantEntry {
        topic_signals: &["circumference of earth", "earth circumference"],
        description: "Circumference of Earth (~40,075 km)",
        expected_log10: 7.603, // log10(4.0075e7 m)
        tolerance: 0.3,
    },
    // ── Computing ─────────────────────────────────────────────────────────────
    ConstantEntry {
        topic_signals: &[
            "transistors on chip",
            "transistor count",
            "nm process",
            "nanometer chip",
        ],
        description: "Transistor counts on modern chips (10⁹–10¹²)",
        expected_log10: 10.5,
        tolerance: 2.0,
    },
    ConstantEntry {
        topic_signals: &[
            "internet speed",
            "broadband speed",
            "5g speed",
            "fiber speed",
        ],
        description: "Typical broadband/5G speeds (10–1000 Mbps)",
        expected_log10: 8.0, // log10(100e6 bps)
        tolerance: 2.0,
    },
    // ── Economics ─────────────────────────────────────────────────────────────
    ConstantEntry {
        topic_signals: &["world gdp", "global gdp", "world economy size"],
        description: "World GDP (~$100 trillion USD)",
        expected_log10: 14.0, // log10(1e14)
        tolerance: 1.0,
    },
    ConstantEntry {
        topic_signals: &["us gdp", "united states gdp", "american gdp", "gdp of usa"],
        description: "US GDP (~$25–28 trillion USD)",
        expected_log10: 13.41, // log10(2.5e13)
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &["world population", "global population", "people on earth"],
        description: "World population (~8×10⁹)",
        expected_log10: 9.903,
        tolerance: 0.3,
    },
    ConstantEntry {
        topic_signals: &[
            "us population",
            "population of united states",
            "american population",
        ],
        description: "US population (~330 million)",
        expected_log10: 8.519,
        tolerance: 0.3,
    },
    ConstantEntry {
        topic_signals: &[
            "china population",
            "population of china",
            "chinese population",
        ],
        description: "China population (~1.4 billion)",
        expected_log10: 9.146,
        tolerance: 0.3,
    },
    ConstantEntry {
        topic_signals: &["india population", "population of india"],
        description: "India population (~1.4 billion)",
        expected_log10: 9.146,
        tolerance: 0.3,
    },
    // ── Energy ───────────────────────────────────────────────────────────────
    ConstantEntry {
        topic_signals: &["hiroshima bomb", "atomic bomb energy", "little boy energy"],
        description: "Hiroshima bomb yield (~15 kilotons TNT ≈ 6.3×10¹³ J)",
        expected_log10: 13.799,
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &[
            "solar luminosity",
            "sun's power output",
            "energy output of sun",
        ],
        description: "Solar luminosity (~3.828×10²⁶ W)",
        expected_log10: 26.583,
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &[
            "human daily calorie",
            "caloric needs",
            "calories per day",
            "kcal per day",
        ],
        description: "Human daily calorie need (~2000–2500 kcal)",
        expected_log10: 3.362, // log10(2300)
        tolerance: 0.5,
    },
    // ── Speed / Distance ─────────────────────────────────────────────────────
    ConstantEntry {
        topic_signals: &["sound speed", "speed of sound", "mach 1", "1 mach"],
        description: "Speed of sound in air (~343 m/s at 20°C)",
        expected_log10: 2.535,
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &["terminal velocity", "free fall speed", "skydiver speed"],
        description: "Human terminal velocity (~55–60 m/s / 200 km/h)",
        expected_log10: 1.763,
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &[
            "escape velocity earth",
            "escape velocity",
            "to escape earth",
        ],
        description: "Earth escape velocity (~11.2 km/s)",
        expected_log10: 4.049, // log10(11200 m/s)
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &["distance to moon", "moon distance", "earth to moon"],
        description: "Distance to the Moon (~3.844×10⁸ m)",
        expected_log10: 8.585,
        tolerance: 0.3,
    },
    ConstantEntry {
        topic_signals: &["distance to sun", "sun distance", "earth to sun"],
        description: "Distance to the Sun (~1.496×10¹¹ m)",
        expected_log10: 11.175,
        tolerance: 0.3,
    },
    // ── Temperature extremes ─────────────────────────────────────────────────
    ConstantEntry {
        topic_signals: &[
            "absolute zero",
            "0 kelvin",
            "0k",
            "-273",
            "lowest temperature possible",
        ],
        description: "Absolute zero (0 K = −273.15 °C)",
        expected_log10: 2.436, // log10(273.15) for the magnitude
        tolerance: 0.3,
    },
    ConstantEntry {
        topic_signals: &[
            "surface temperature of sun",
            "sun surface temp",
            "photosphere temperature",
        ],
        description: "Surface temperature of the Sun (~5778 K)",
        expected_log10: 3.762,
        tolerance: 0.5,
    },
    // ── Chemistry — additional ────────────────────────────────────────────────
    ConstantEntry {
        topic_signals: &[
            "molar mass of water",
            "molecular weight of water",
            "water molar mass",
        ],
        description: "Molar mass of water (18.015 g/mol)",
        expected_log10: 1.256,
        tolerance: 0.3,
    },
    ConstantEntry {
        topic_signals: &[
            "molar mass of carbon dioxide",
            "co2 molar mass",
            "molecular weight co2",
        ],
        description: "Molar mass of CO₂ (44.01 g/mol)",
        expected_log10: 1.644,
        tolerance: 0.3,
    },
    ConstantEntry {
        topic_signals: &["ph of pure water", "neutral ph", "water ph"],
        description: "pH of pure water at 25 °C (7.0)",
        expected_log10: 0.845, // log10(7)
        tolerance: 0.3,
    },
    ConstantEntry {
        topic_signals: &["density of water", "water density"],
        description: "Density of liquid water at 4 °C (~1000 kg/m³)",
        expected_log10: 3.0,
        tolerance: 0.3,
    },
    ConstantEntry {
        topic_signals: &["density of gold", "gold density"],
        description: "Density of gold (~19,300 kg/m³)",
        expected_log10: 4.286,
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &["density of iron", "iron density", "density of steel"],
        description: "Density of iron/steel (~7,874 kg/m³)",
        expected_log10: 3.896,
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &[
            "gas constant",
            "universal gas constant",
            "ideal gas constant",
            "8.314",
        ],
        description: "Universal gas constant R (~8.314 J/(mol·K))",
        expected_log10: 0.920,
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &["faraday constant", "96485", "faraday's constant"],
        description: "Faraday constant (~96,485 C/mol)",
        expected_log10: 4.985,
        tolerance: 0.5,
    },
    // ── Biology — additional ──────────────────────────────────────────────────
    ConstantEntry {
        topic_signals: &["human blood pressure", "normal blood pressure", "systolic"],
        description: "Normal systolic blood pressure (~120 mmHg)",
        expected_log10: 2.079,
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &[
            "gestation period human",
            "human pregnancy",
            "pregnancy duration",
        ],
        description: "Human gestation period (~266–280 days ≈ 9 months)",
        expected_log10: 2.449, // log10(280)
        tolerance: 0.3,
    },
    ConstantEntry {
        topic_signals: &["number of bones", "bones in human body", "human skeleton"],
        description: "Bones in the adult human body (206)",
        expected_log10: 2.314,
        tolerance: 0.3,
    },
    ConstantEntry {
        topic_signals: &[
            "number of chromosomes",
            "human chromosomes",
            "chromosomes in human",
        ],
        description: "Chromosomes in human diploid cells (46)",
        expected_log10: 1.663,
        tolerance: 0.2,
    },
    ConstantEntry {
        topic_signals: &[
            "number of muscles",
            "muscles in human body",
            "human muscle count",
        ],
        description: "Number of muscles in the human body (~640)",
        expected_log10: 2.806,
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &[
            "speed of nerve impulse",
            "nerve conduction velocity",
            "nerve impulse",
        ],
        description: "Speed of nerve impulse (1–100 m/s, fast myelinated ~70–100 m/s)",
        expected_log10: 1.845, // log10(70)
        tolerance: 1.0,
    },
    ConstantEntry {
        topic_signals: &[
            "human brain weight",
            "weight of brain",
            "mass of human brain",
        ],
        description: "Human brain mass (~1.4 kg)",
        expected_log10: 0.146,
        tolerance: 0.5,
    },
    // ── Geophysics — additional ───────────────────────────────────────────────
    ConstantEntry {
        topic_signals: &[
            "pressure at sea level",
            "atmospheric pressure",
            "standard atmosphere",
            "101325",
        ],
        description: "Standard atmospheric pressure at sea level (~101,325 Pa)",
        expected_log10: 5.006,
        tolerance: 0.3,
    },
    ConstantEntry {
        topic_signals: &[
            "acceleration due to gravity",
            "gravitational acceleration",
            "g =",
            "9.8 m",
        ],
        description: "Surface gravitational acceleration (g ≈ 9.8 m/s²)",
        expected_log10: 0.991,
        tolerance: 0.3,
    },
    ConstantEntry {
        topic_signals: &[
            "magnetic field earth",
            "earth magnetic field",
            "geomagnetic field strength",
        ],
        description: "Earth's surface magnetic field strength (~25–65 μT)",
        expected_log10: -4.7, // log10(45e-6) ≈ -4.35
        tolerance: 1.0,
    },
    ConstantEntry {
        topic_signals: &["temperature at earth's core", "earth core temperature"],
        description: "Temperature at Earth's core (~5,100–6,000 K)",
        expected_log10: 3.73,
        tolerance: 0.3,
    },
    // ── Astronomy — additional ────────────────────────────────────────────────
    ConstantEntry {
        topic_signals: &[
            "hubble constant",
            "hubble's constant",
            "expansion rate universe",
        ],
        description: "Hubble constant (~67–73 km/s/Mpc)",
        expected_log10: 1.85, // log10(70 km/s/Mpc)
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &[
            "temperature of cosmic microwave background",
            "cmb temperature",
            "cosmic background radiation",
        ],
        description: "CMB temperature (~2.725 K)",
        expected_log10: 0.435,
        tolerance: 0.3,
    },
    ConstantEntry {
        topic_signals: &[
            "number of stars in milky way",
            "milky way stars",
            "stars in our galaxy",
        ],
        description: "Number of stars in the Milky Way (~2×10¹¹)",
        expected_log10: 11.301,
        tolerance: 1.0,
    },
    ConstantEntry {
        topic_signals: &[
            "distance to nearest star",
            "alpha centauri distance",
            "proxima centauri distance",
        ],
        description: "Distance to Proxima Centauri (~4.24 light-years ≈ 4.0×10¹⁶ m)",
        expected_log10: 16.602,
        tolerance: 0.5,
    },
    // ── Materials / Engineering ───────────────────────────────────────────────
    ConstantEntry {
        topic_signals: &[
            "young's modulus of steel",
            "elastic modulus steel",
            "steel stiffness",
        ],
        description: "Young's modulus of steel (~200 GPa = 2×10¹¹ Pa)",
        expected_log10: 11.301,
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &[
            "thermal conductivity of copper",
            "copper thermal conductivity",
        ],
        description: "Thermal conductivity of copper (~385 W/(m·K))",
        expected_log10: 2.586,
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &[
            "refractive index of glass",
            "glass refractive index",
            "glass refraction",
        ],
        description: "Refractive index of typical glass (~1.5)",
        expected_log10: 0.176,
        tolerance: 0.3,
    },
    // ── Medicine / Pharmacology ───────────────────────────────────────────────
    ConstantEntry {
        topic_signals: &["lethal dose aspirin", "aspirin ld50", "aspirin toxicity"],
        description: "Aspirin LD₅₀ in rats (~200 mg/kg)",
        expected_log10: 2.301,
        tolerance: 1.0,
    },
    ConstantEntry {
        topic_signals: &[
            "blood volume human",
            "human blood volume",
            "total blood volume",
        ],
        description: "Human total blood volume (~5 liters)",
        expected_log10: 0.699,
        tolerance: 0.5,
    },
    // ── Environment ───────────────────────────────────────────────────────────
    ConstantEntry {
        topic_signals: &[
            "co2 concentration atmosphere",
            "atmospheric co2",
            "ppm co2",
            "carbon dioxide concentration",
        ],
        description: "Atmospheric CO₂ concentration (~420 ppm as of 2024)",
        expected_log10: 2.623,
        tolerance: 0.5,
    },
    ConstantEntry {
        topic_signals: &["annual global carbon emissions", "global co2 emissions"],
        description: "Annual global CO₂ emissions (~37 billion tonnes)",
        expected_log10: 10.568, // log10(3.7e10 tonnes)
        tolerance: 0.5,
    },
];

// ─── Issue type ───────────────────────────────────────────────────────────────

/// Detected numeric plausibility issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumericIssue {
    /// The constant description (e.g., "Speed of light (~3×10⁸ m/s)").
    pub constant_description: String,
    /// The order of magnitude the claim implies.
    pub claimed_log10: f64,
    /// The expected order of magnitude.
    pub expected_log10: f64,
    /// Deviation = |claimed − expected| in log₁₀ units.
    pub deviation: f64,
}

impl std::fmt::Display for NumericIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Numeric plausibility issue — {}: claimed order-of-magnitude {:.1} vs expected {:.1} (deviation {:.1} log₁₀ units)",
            self.constant_description, self.claimed_log10, self.expected_log10, self.deviation
        )
    }
}

// ─── Scanner ─────────────────────────────────────────────────────────────────

/// Numeric plausibility scanner.
///
/// Call `scan(text)` on `ClaimType::Numeric` sentences only.
/// Returns `None` on clean or unparseable claims (fail-safe).
pub struct NumericPlausibilityScanner;

impl NumericPlausibilityScanner {
    /// Scan a single claim text for numeric plausibility violations.
    ///
    /// Returns `None` if:
    /// - No constant entry matches the text (no topic signal found).
    /// - A number is found but parses to zero or negative (skip safely).
    /// - The parsed value is within the tolerance envelope.
    ///
    /// Returns `Some(NumericIssue)` only when a constant is recognised AND
    /// the extracted numeric value deviates beyond tolerance.
    pub fn scan(text: &str) -> Option<NumericIssue> {
        let lower = text.to_lowercase();

        for entry in CONSTANT_ATLAS {
            if !entry.topic_signals.iter().any(|s| lower.contains(s)) {
                continue;
            }
            // Try to extract the first numeric literal from the text.
            let value = extract_first_number(text)?;
            if value <= 0.0 {
                return None; // Can't take log of zero/negative — skip
            }
            let claimed_log10 = value.log10();
            let deviation = (claimed_log10 - entry.expected_log10).abs();
            if deviation > entry.tolerance {
                return Some(NumericIssue {
                    constant_description: entry.description.to_string(),
                    claimed_log10,
                    expected_log10: entry.expected_log10,
                    deviation,
                });
            }
            // First matching constant found and value is in-range: clean, stop scan.
            return None;
        }
        None // No matching constant found — no opinion
    }
}

// ─── Numeric extraction ───────────────────────────────────────────────────────

/// Extract the first numeric literal (integer or decimal, possibly with commas,
/// word-scale suffixes (billion, million, trillion), or scientific notation
/// suffix like "e8", "×10^8", "x10^8") from text.
///
/// Deliberately approximate: returns `None` rather than crash on malformed input.
fn extract_first_number(text: &str) -> Option<f64> {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < bytes.len()
                && (bytes[i].is_ascii_digit() || bytes[i] == b'.' || bytes[i] == b',')
            {
                i += 1;
            }
            let raw = &text[start..i];
            let cleaned = raw.replace(',', "");
            let mantissa: f64 = cleaned.parse().ok()?;

            // Check for word-scale suffixes (billion, million, trillion)
            let rest = text[i..].trim_start();
            let lower_rest = rest.to_lowercase();
            let (word_multiplier, rest_after_word) = if lower_rest.starts_with("billion") {
                (1_000_000_000.0_f64, &rest[7..])
            } else if lower_rest.starts_with("trillion") {
                (1_000_000_000_000.0_f64, &rest[8..])
            } else if lower_rest.starts_with("million") {
                (1_000_000.0_f64, &rest[7..])
            } else if lower_rest.starts_with("thousand") {
                (1_000.0_f64, &rest[8..])
            } else {
                (1.0_f64, rest)
            };

            // Check for scientific notation suffix: e8, e-34, ×10^8, x10^8, ×10⁸
            let exp = parse_exponent_suffix(rest_after_word.trim_start());
            let value = mantissa * word_multiplier * 10f64.powi(exp);
            return Some(value);
        }
        i += 1;
    }
    None
}

/// Parse an optional exponent suffix from text immediately after a mantissa.
///
/// Handles: "e8", "e-34", "e+23", "×10^8", "x10^8", "×10⁻³⁴", "× 10^8".
/// Returns 0 (i.e., 10⁰ = 1) if no exponent suffix is found.
fn parse_exponent_suffix(rest: &str) -> i32 {
    let lower = rest.to_lowercase();

    // "e" notation: e8, e-34, e+23
    if let Some(after) = lower.strip_prefix('e') {
        // May have sign
        let after = after.trim_start_matches(' ');
        let end = after
            .find(|c: char| !c.is_ascii_digit() && c != '-' && c != '+')
            .unwrap_or(after.len());
        if end > 0 {
            return after[..end].parse().unwrap_or(0);
        }
    }

    // "× 10^8" or "x10^8" or "×10⁸" patterns
    if lower.starts_with('×') || lower.starts_with('x') {
        // Find "10"
        if let Some(idx) = lower.find("10") {
            let after = &lower[idx + 2..];
            // Skip ^ or ⁻ etc.
            let after = after.trim_start_matches('^').trim_start_matches(' ');
            // Convert unicode superscripts to ASCII
            let ascii_exp = superscript_to_ascii(after);
            return ascii_exp.parse().unwrap_or(0);
        }
    }

    0
}

/// Convert leading unicode superscript digits/signs to an ASCII string.
fn superscript_to_ascii(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        match ch {
            '⁰' => out.push('0'),
            '¹' => out.push('1'),
            '²' => out.push('2'),
            '³' => out.push('3'),
            '⁴' => out.push('4'),
            '⁵' => out.push('5'),
            '⁶' => out.push('6'),
            '⁷' => out.push('7'),
            '⁸' => out.push('8'),
            '⁹' => out.push('9'),
            '⁻' => out.push('-'),
            '+' | '-' => out.push(ch),
            c if c.is_ascii_digit() => out.push(c),
            _ => break,
        }
    }
    out
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn speed_of_light_correct_value_passes() {
        // 3×10^8 is correct — should NOT fire
        let result =
            NumericPlausibilityScanner::scan("The speed of light is approximately 3×10^8 m/s.");
        assert!(
            result.is_none(),
            "Correct value should not flag: {:?}",
            result
        );
    }

    #[test]
    fn speed_of_light_wrong_order_fires() {
        // Claiming 3×10^4 is off by 4 orders of magnitude — should fire
        let result =
            NumericPlausibilityScanner::scan("The speed of light is approximately 3×10^4 m/s.");
        assert!(result.is_some(), "Wrong order of magnitude should flag");
        let issue = result.unwrap();
        assert!(issue.deviation > 3.0);
    }

    #[test]
    fn world_population_correct_passes() {
        let result = NumericPlausibilityScanner::scan(
            "The world population is approximately 8 billion people on earth.",
        );
        assert!(result.is_none(), "8 billion is correct: {:?}", result);
    }

    #[test]
    fn unknown_constant_no_false_positive() {
        // Should not fire on arbitrary text with numbers
        let result = NumericPlausibilityScanner::scan("She bought 42 apples at the market.");
        assert!(result.is_none(), "No constant match → no flag");
    }

    #[test]
    fn body_temperature_correct_passes() {
        let result = NumericPlausibilityScanner::scan(
            "Normal human body temperature is 37 degrees Celsius.",
        );
        assert!(result.is_none(), "37 °C is correct: {:?}", result);
    }

    #[test]
    fn body_temperature_absurd_fires() {
        let result = NumericPlausibilityScanner::scan(
            "Normal human body temperature is 3700 degrees Celsius.",
        );
        assert!(result.is_some(), "3700 °C is implausible");
    }

    #[test]
    fn extract_first_number_plain() {
        assert!((extract_first_number("42 apples").unwrap() - 42.0).abs() < 0.001);
    }

    #[test]
    fn extract_first_number_decimal() {
        assert!((extract_first_number("6.626e-34 J·s").unwrap() - 6.626e-34).abs() < 1e-37);
    }

    #[test]
    fn extract_first_number_comma() {
        assert!((extract_first_number("40,075 km").unwrap() - 40075.0).abs() < 0.1);
    }
}
