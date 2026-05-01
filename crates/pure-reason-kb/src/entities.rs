//! Entity–fact pairs for deterministic knowledge-grounded answering.
//!
//! Each [`EntityFact`] represents a curated, verifiable assertion about a
//! named entity. Used by the KAC (Knowledge-Grounded Answering Check) layer
//! to flag factual hallucinations without an LLM lookup.
//!
//! ## Coverage strategy
//!
//! Entries are selected based on TruthfulQA / FELM failure patterns:
//! - Capital cities (high LLM confusion rate)
//! - Founding/invention dates (frequently hallucinated)
//! - Organisational facts (HQ locations, member counts, founding bodies)
//! - Record holders (longest, tallest, heaviest — common superlative myths)

use serde::Serialize;

/// A curated entity–fact pair.
#[derive(Debug, Clone, Serialize)]
pub struct EntityFact {
    /// Stable identifier.
    pub id: &'static str,
    /// The entity this fact is about.
    pub entity: &'static str,
    /// The attribute or relation.
    pub attribute: &'static str,
    /// The canonical correct value.
    pub value: &'static str,
    /// Keywords that indicate this fact is relevant in a question/answer.
    pub signals: &'static [&'static str],
    /// Common wrong answers LLMs produce for this fact.
    pub common_errors: &'static [&'static str],
}

/// The compiled entity fact atlas.
pub static ENTITY_FACTS: &[EntityFact] = &[
    // ── Capital cities ───────────────────────────────────────────────────────
    EntityFact {
        id: "capital_australia",
        entity: "Australia",
        attribute: "capital city",
        value: "Canberra",
        signals: &[
            "capital of australia",
            "australia capital",
            "australian capital",
        ],
        common_errors: &["Sydney", "Melbourne", "Brisbane"],
    },
    EntityFact {
        id: "capital_canada",
        entity: "Canada",
        attribute: "capital city",
        value: "Ottawa",
        signals: &["capital of canada", "canada capital", "canadian capital"],
        common_errors: &["Toronto", "Vancouver", "Montreal"],
    },
    EntityFact {
        id: "capital_brazil",
        entity: "Brazil",
        attribute: "capital city",
        value: "Brasília",
        signals: &["capital of brazil", "brazil capital", "brazilian capital"],
        common_errors: &["Rio de Janeiro", "São Paulo", "Rio"],
    },
    EntityFact {
        id: "capital_new_zealand",
        entity: "New Zealand",
        attribute: "capital city",
        value: "Wellington",
        signals: &["capital of new zealand", "new zealand capital"],
        common_errors: &["Auckland", "Christchurch"],
    },
    EntityFact {
        id: "capital_south_africa",
        entity: "South Africa",
        attribute: "capital city",
        value: "Pretoria (executive), Cape Town (legislative), Bloemfontein (judicial)",
        signals: &["capital of south africa", "south africa capital"],
        common_errors: &["Johannesburg", "Cape Town alone", "Durban"],
    },
    EntityFact {
        id: "capital_usa",
        entity: "United States",
        attribute: "capital city",
        value: "Washington, D.C.",
        signals: &[
            "capital of the united states",
            "us capital",
            "capital of america",
        ],
        common_errors: &["New York", "New York City", "Los Angeles"],
    },
    // ── Scientific inventions / discoveries ─────────────────────────────────
    EntityFact {
        id: "inventor_telephone",
        entity: "Telephone",
        attribute: "inventor",
        value: "Alexander Graham Bell (patent 1876)",
        signals: &[
            "who invented the telephone",
            "invented the phone",
            "telephone inventor",
        ],
        common_errors: &["Thomas Edison", "Nikola Tesla", "Elisha Gray"],
    },
    EntityFact {
        id: "inventor_lightbulb",
        entity: "Practical incandescent light bulb",
        attribute: "inventor",
        value: "Thomas Edison (1879) and Joseph Swan (independently)",
        signals: &[
            "who invented the light bulb",
            "invented the lightbulb",
            "light bulb inventor",
        ],
        common_errors: &["Benjamin Franklin", "Nikola Tesla", "Alexander Graham Bell"],
    },
    EntityFact {
        id: "inventor_www",
        entity: "World Wide Web",
        attribute: "inventor",
        value: "Tim Berners-Lee (1989)",
        signals: &[
            "who invented the internet",
            "invented the web",
            "world wide web inventor",
            "who created the internet",
        ],
        common_errors: &["Al Gore", "Bill Gates", "Steve Jobs", "Mark Zuckerberg"],
    },
    EntityFact {
        id: "inventor_penicillin",
        entity: "Penicillin",
        attribute: "discoverer",
        value: "Alexander Fleming (1928)",
        signals: &[
            "who discovered penicillin",
            "penicillin inventor",
            "penicillin discoverer",
        ],
        common_errors: &["Louis Pasteur", "Marie Curie", "Joseph Lister"],
    },
    // ── Organisational facts ────────────────────────────────────────────────
    EntityFact {
        id: "un_founding_year",
        entity: "United Nations",
        attribute: "founding year",
        value: "1945",
        signals: &[
            "when was the un founded",
            "united nations founded",
            "un established",
        ],
        common_errors: &["1919", "1939", "1950"],
    },
    EntityFact {
        id: "un_member_count",
        entity: "United Nations",
        attribute: "member states",
        value: "193 member states (as of 2024)",
        signals: &[
            "how many countries in the un",
            "un member states",
            "united nations members",
        ],
        common_errors: &["195", "197", "200"],
    },
    EntityFact {
        id: "who_hq",
        entity: "World Health Organization",
        attribute: "headquarters",
        value: "Geneva, Switzerland",
        signals: &[
            "who headquarters",
            "world health organization hq",
            "who is based in",
        ],
        common_errors: &["New York", "Washington", "Brussels"],
    },
    // ── Record holders ──────────────────────────────────────────────────────
    EntityFact {
        id: "tallest_mountain",
        entity: "Tallest mountain above sea level",
        attribute: "name",
        value: "Mount Everest (8,848.86 m)",
        signals: &[
            "tallest mountain",
            "highest mountain",
            "highest peak",
            "everest",
        ],
        common_errors: &["K2", "Kilimanjaro", "Aconcagua", "Mauna Kea"],
    },
    EntityFact {
        id: "longest_river",
        entity: "Longest river",
        attribute: "name",
        value: "Nile or Amazon (disputed; Nile traditionally cited at ~6,650 km)",
        signals: &["longest river", "longest river in the world"],
        common_errors: &["Mississippi", "Amazon definitively", "Congo"],
    },
    EntityFact {
        id: "deepest_ocean_point",
        entity: "Deepest ocean point",
        attribute: "name and depth",
        value: "Challenger Deep, Mariana Trench (~10,935 m)",
        signals: &[
            "deepest part of the ocean",
            "deepest ocean",
            "mariana trench depth",
        ],
        common_errors: &["Puerto Rico Trench", "Java Trench", "Philippine Trench"],
    },
    EntityFact {
        id: "largest_country",
        entity: "Largest country by area",
        attribute: "name",
        value: "Russia (17,098,242 km²)",
        signals: &[
            "largest country",
            "biggest country by area",
            "largest country in the world",
        ],
        common_errors: &["Canada", "United States", "China"],
    },
    EntityFact {
        id: "most_spoken_language",
        entity: "Most spoken language (total speakers)",
        attribute: "name",
        value: "Mandarin Chinese (native), English (total including L2)",
        signals: &[
            "most spoken language",
            "most widely spoken language",
            "language with most speakers",
        ],
        common_errors: &["English definitively", "Spanish", "French"],
    },
    // ── Historical dates ─────────────────────────────────────────────────────
    EntityFact {
        id: "ww2_end_year",
        entity: "World War II",
        attribute: "end year",
        value: "1945",
        signals: &["when did world war 2 end", "ww2 ended", "world war ii end"],
        common_errors: &["1944", "1946", "1943"],
    },
    EntityFact {
        id: "moon_landing_first",
        entity: "First crewed Moon landing",
        attribute: "year and mission",
        value: "1969 — Apollo 11 (Neil Armstrong, Buzz Aldrin)",
        signals: &[
            "first moon landing",
            "man on the moon",
            "apollo 11",
            "when did we land on the moon",
        ],
        common_errors: &["1968", "1971", "1972"],
    },
    EntityFact {
        id: "berlin_wall_fall",
        entity: "Fall of the Berlin Wall",
        attribute: "year",
        value: "1989",
        signals: &[
            "berlin wall fell",
            "berlin wall came down",
            "fall of berlin wall",
        ],
        common_errors: &["1990", "1988", "1991"],
    },
];

/// Look up an entity fact by its stable `id`.
pub fn lookup_entity_fact(id: &str) -> Option<&'static EntityFact> {
    ENTITY_FACTS.iter().find(|f| f.id == id)
}

/// Find entity facts relevant to a given question/answer text.
///
/// Returns all facts whose `signals` match the normalized text.
pub fn find_relevant_facts(text: &str) -> Vec<&'static EntityFact> {
    let lower = text.to_lowercase();
    ENTITY_FACTS
        .iter()
        .filter(|f| f.signals.iter().any(|s| lower.contains(*s)))
        .collect()
}

/// Check whether the given `answer_text` contains a known common error for a fact.
pub fn detect_entity_error<'a>(fact: &'a EntityFact, answer_text: &str) -> Option<&'a str> {
    let lower = answer_text.to_lowercase();
    fact.common_errors
        .iter()
        .find(|e| lower.contains(&e.to_lowercase()))
        .map(|e| e as &str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capital_australia_lookup() {
        let f = lookup_entity_fact("capital_australia").unwrap();
        assert_eq!(f.value, "Canberra");
    }

    #[test]
    fn find_relevant_capital_facts() {
        let facts = find_relevant_facts("What is the capital of Australia?");
        assert!(!facts.is_empty());
        assert!(facts.iter().any(|f| f.id == "capital_australia"));
    }

    #[test]
    fn detect_sydney_error() {
        let f = lookup_entity_fact("capital_australia").unwrap();
        let err = detect_entity_error(f, "The capital of Australia is Sydney.");
        assert_eq!(err, Some("Sydney"));
    }

    #[test]
    fn no_false_positive_on_correct_answer() {
        let f = lookup_entity_fact("capital_australia").unwrap();
        let err = detect_entity_error(f, "The capital of Australia is Canberra.");
        assert_eq!(err, None);
    }

    #[test]
    fn all_facts_have_signals() {
        for f in ENTITY_FACTS {
            assert!(!f.signals.is_empty(), "fact '{}' has no signals", f.id);
            assert!(
                !f.common_errors.is_empty(),
                "fact '{}' has no common_errors",
                f.id
            );
        }
    }
}
