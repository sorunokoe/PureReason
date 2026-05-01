"""NLP utilities for the Z3 syllogism verifier — spaCy-powered.

Requires the ``[nlp]`` optional extra::

    pip install pureason[nlp]
    python -m spacy download en_core_web_sm

This module replaces the original frozenset-based vocabulary
(``_NON_ENTITIES``, ``_PROP_STOP``, ``_AUX_VERBS``) and manual stem
functions (``_verb_stem``, ``_noun_stem``) with spaCy's pre-trained
linguistic model.  Zero words are hardcoded: spaCy encodes them.
"""

from __future__ import annotations

import re
from functools import lru_cache

_NLP = None


def _get_nlp():
    """Return a cached spaCy ``Language`` object.

    Raises ``ImportError`` if spaCy is not installed, or ``OSError`` if the
    ``en_core_web_sm`` model has not been downloaded.
    """
    global _NLP
    if _NLP is None:
        try:
            import spacy
        except ImportError as exc:
            raise ImportError(
                "PureReason reasoning requires spaCy. Install it with: pip install pureason[nlp]"
            ) from exc
        try:
            _NLP = spacy.load("en_core_web_sm", disable=["ner"])
        except OSError as exc:
            raise OSError(
                "spaCy model 'en_core_web_sm' not found. "
                "Install it with: python -m spacy download en_core_web_sm"
            ) from exc
    return _NLP


# ---------------------------------------------------------------------------
# Identifier normalization (no linguistic knowledge — pure string manipulation)
# ---------------------------------------------------------------------------


def _norm_id(text: str) -> str:
    """Normalize arbitrary text to a valid Z3 identifier.

    Replaces any run of non-alphanumeric characters with ``_`` and strips
    leading/trailing underscores.  Returns ``"x"`` for the empty result.
    """
    return re.sub(r"[^a-z0-9]+", "_", text.lower().strip()).strip("_") or "x"


# ---------------------------------------------------------------------------
# Lemmatization (single-word helper, cached)
# ---------------------------------------------------------------------------


@lru_cache(maxsize=512)
def _lemma(word: str) -> str:
    """Return the lowercase spaCy lemma of a single word.

    Uses ``lru_cache`` so repeated words (common in predicate normalization)
    pay the spaCy overhead only once.
    """
    nlp = _get_nlp()
    doc = nlp(word.lower())
    return doc[0].lemma_.lower() if doc else word.lower()


# ---------------------------------------------------------------------------
# Predicate and propositional key builders
# ---------------------------------------------------------------------------


@lru_cache(maxsize=512)
def _pred_key(pred: str) -> str:
    """Canonical predicate key using spaCy lemmatization.

    Strips stop words (determiners, auxiliaries, prepositions, punctuation)
    and joins the remaining lemmas with ``_``.  Always returns a non-empty
    string — falls back to ``_norm_id`` when all tokens are stop words.

    Examples:
        ``"is a mammal"``  →  ``"mammal"``
        ``"warm-blooded"``  →  ``"warm_blooded"``
    """
    nlp = _get_nlp()
    doc = nlp(pred.lower())
    content = [
        t.lemma_ for t in doc if not t.is_stop and not t.is_punct and not t.is_space and t.lemma_
    ]
    return "_".join(content) or _norm_id(pred)


@lru_cache(maxsize=512)
def _prop_key(clause: str) -> str:
    """Canonical content-word key for propositional atom matching.

    Excludes negation tokens (``dep_=="neg"``), stop words, and grammatical
    function words; lemmatizes, sorts, and deduplicates.  Always returns a
    non-empty string.

    Note: order-insensitive deduplication is intentional — ``"Alice admires Bob"``
    and ``"Bob admires Alice"`` map to the same propositional atom.
    """
    nlp = _get_nlp()
    doc = nlp(clause.lower().strip().rstrip(".,!?"))
    content = [
        t.lemma_
        for t in doc
        if not t.is_stop
        and not t.is_punct
        and not t.is_space
        and t.dep_ != "neg"
        and t.pos_ not in {"DET", "ADP", "AUX", "CCONJ", "SCONJ", "PART", "PRON"}
        and t.lemma_
    ]
    return "_".join(sorted(set(content))) or _norm_id(clause)


# ---------------------------------------------------------------------------
# Entity extraction
# ---------------------------------------------------------------------------


def _extract_entities(texts: list[str]) -> list[str]:
    """Extract named entities from sentences using spaCy POS and NER.

    Identifies:

    - Proper nouns (``PROPN``) that are not stop words.
    - Common nouns (``NOUN``) that start with an uppercase letter — captures
      entity-like capitalized words ("Whales", "Copper") that spaCy tags as
      common nouns rather than proper nouns.
    - Numeric tokens (``NUM``).
    - Common nouns preceded by the definite article *the*.

    Returns a sorted, deduplicated list of lowercase strings.
    Falls back to ``["obj1", "obj2"]`` when no entities are found.
    """
    nlp = _get_nlp()
    entities: set[str] = set()
    for text in texts:
        doc = nlp(text)
        for i, token in enumerate(doc):
            if token.is_space or token.is_punct:
                continue
            if (token.pos_ == "PROPN" and not token.is_stop) or (
                token.pos_ == "NOUN" and token.text[0].isupper() and not token.is_stop
            ):
                entities.add(token.lemma_.lower())
            elif token.pos_ == "NUM":
                entities.add(token.text)
            elif token.pos_ == "NOUN" and i > 0 and doc[i - 1].text.lower() == "the":
                entities.add(token.lemma_.lower())
    return sorted(entities) or ["obj1", "obj2"]
