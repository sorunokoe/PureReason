"""Tests for pureason.reasoning._z3utils utility functions (spaCy-based).

Covers normalization helpers, key generation, and entity extraction.
Also includes _Z3Context integration tests for the parsing paths.
"""

from __future__ import annotations

import unittest

from pureason.reasoning._z3utils import (
    _extract_entities,
    _lemma,
    _norm_id,
    _pred_key,
    _prop_key,
)

# ---------------------------------------------------------------------------
# _norm_id
# ---------------------------------------------------------------------------


class TestNormId(unittest.TestCase):
    def test_lowercases_and_replaces_spaces(self) -> None:
        self.assertEqual(_norm_id("Hello World"), "hello_world")

    def test_replaces_special_chars(self) -> None:
        self.assertEqual(_norm_id("foo-bar!baz"), "foo_bar_baz")

    def test_strips_leading_trailing_underscores(self) -> None:
        self.assertEqual(_norm_id("  hello  "), "hello")

    def test_empty_string_returns_x(self) -> None:
        self.assertEqual(_norm_id(""), "x")

    def test_only_special_chars_returns_x(self) -> None:
        self.assertEqual(_norm_id("!!!"), "x")

    def test_preserves_digits(self) -> None:
        self.assertEqual(_norm_id("item42"), "item42")

    def test_unicode_removed(self) -> None:
        result = _norm_id("café")
        self.assertRegex(result, r"^[a-z0-9_]+$")


# ---------------------------------------------------------------------------
# _lemma
# ---------------------------------------------------------------------------


class TestLemma(unittest.TestCase):
    def test_verb_inflection(self) -> None:
        self.assertEqual(_lemma("conducts"), "conduct")

    def test_plural_noun(self) -> None:
        self.assertEqual(_lemma("mammals"), "mammal")

    def test_lowercase_output(self) -> None:
        self.assertEqual(_lemma("ALICE"), "alice")

    def test_already_base_form(self) -> None:
        self.assertEqual(_lemma("run"), "run")


# ---------------------------------------------------------------------------
# _pred_key
# ---------------------------------------------------------------------------


class TestPredKey(unittest.TestCase):
    def test_strips_stop_words(self) -> None:
        key = _pred_key("is a mammal")
        self.assertIn("mammal", key)
        self.assertNotIn("is", key)

    def test_all_stop_words_returns_nonempty(self) -> None:
        key = _pred_key("is are was")
        self.assertGreater(len(key), 0)

    def test_noun_plural_normalized(self) -> None:
        key_plural = _pred_key("animals")
        key_singular = _pred_key("animal")
        self.assertEqual(key_plural, key_singular)

    def test_empty_string_returns_nonempty(self) -> None:
        key = _pred_key("")
        self.assertGreater(len(key), 0)

    def test_hyphenated_predicate(self) -> None:
        key = _pred_key("warm-blooded")
        self.assertIn("warm", key)
        self.assertIn("blood", key)


# ---------------------------------------------------------------------------
# _prop_key
# ---------------------------------------------------------------------------


class TestPropKey(unittest.TestCase):
    def test_negation_stripped(self) -> None:
        key_pos = _prop_key("Alice is happy")
        key_neg = _prop_key("Alice is not happy")
        self.assertEqual(key_pos, key_neg)

    def test_order_insensitive(self) -> None:
        key1 = _prop_key("cats chase dogs")
        key2 = _prop_key("dogs chase cats")
        self.assertEqual(key1, key2)

    def test_stop_words_removed(self) -> None:
        key = _prop_key("the cat is on the mat")
        self.assertNotIn("the", key)
        self.assertNotIn("is", key)

    def test_all_stop_words_returns_nonempty(self) -> None:
        key = _prop_key("is are was the a")
        self.assertGreater(len(key), 0)

    def test_empty_string_returns_nonempty(self) -> None:
        key = _prop_key("")
        self.assertGreater(len(key), 0)

    def test_punctuation_stripped(self) -> None:
        key1 = _prop_key("Alice wins.")
        key2 = _prop_key("Alice wins")
        self.assertEqual(key1, key2)


# ---------------------------------------------------------------------------
# _extract_entities
# ---------------------------------------------------------------------------


class TestExtractEntities(unittest.TestCase):
    def test_extracts_proper_nouns(self) -> None:
        entities = _extract_entities(["Alice is a doctor."])
        self.assertIn("alice", entities)

    def test_ignores_common_nouns_without_the(self) -> None:
        entities = _extract_entities(["All dogs are animals."])
        self.assertIsInstance(entities, list)

    def test_extracts_numbers_as_bare_digits(self) -> None:
        entities = _extract_entities(["There are 42 people."])
        self.assertIn("42", entities)
        self.assertNotIn("n42", entities)

    def test_extracts_the_plus_noun(self) -> None:
        entities = _extract_entities(["The cat sat on the mat."])
        self.assertIn("cat", entities)
        self.assertIn("mat", entities)

    def test_falls_back_to_obj1_obj2(self) -> None:
        entities = _extract_entities(["all are warm-blooded."])
        self.assertEqual(entities, ["obj1", "obj2"])

    def test_multiple_texts(self) -> None:
        entities = _extract_entities(["Alice wins.", "Bob loses."])
        self.assertIn("alice", entities)
        self.assertIn("bob", entities)

    def test_sorted_and_unique(self) -> None:
        entities = _extract_entities(["Alice and Alice run.", "Bob runs."])
        self.assertEqual(entities, sorted(set(entities)))


# ---------------------------------------------------------------------------
# _Z3Context integration tests (require z3-solver)
# ---------------------------------------------------------------------------


def _z3_available() -> bool:
    try:
        import z3  # noqa: F401

        return True
    except ImportError:
        return False


def _spacy_available() -> bool:
    try:
        import spacy  # noqa: F401

        return True
    except ImportError:
        return False


# ---------------------------------------------------------------------------
# _heuristic_fallacy_check — direct unit tests (requires spaCy stop words)
# ---------------------------------------------------------------------------


@unittest.skipUnless(_spacy_available(), "spaCy not installed")
class TestHeuristicFallacyCheck(unittest.TestCase):
    """Unit tests for _heuristic_fallacy_check logic paths."""

    def _check(self, premises: list[str], conclusion: str):
        from pureason.reasoning.syllogism import _heuristic_fallacy_check

        return _heuristic_fallacy_check(premises, conclusion)

    def test_hasty_generalisation_detected(self) -> None:
        """Specific-instance premises + universal conclusion → False (fallacy)."""
        result = self._check(
            ["Alice is tall.", "Bob is tall."],
            "All people are tall.",
        )
        self.assertFalse(result)

    def test_universal_premise_prevents_hasty_gen_flag(self) -> None:
        """Universal premise present → not flagged as hasty generalisation."""
        result = self._check(
            ["All mammals are warm-blooded."],
            "All whales are warm-blooded.",
        )
        # Has universal premise; not flagged as hasty gen.
        # Conclusion words "whale", "warm-blooded" — "whale" absent from premise → not circular.
        self.assertIsNone(result)

    def test_conditional_premise_prevents_hasty_gen_flag(self) -> None:
        """If-premise counts as universal → universal conclusion not flagged."""
        result = self._check(
            ["If it rains, the road gets wet."],
            "All wet roads become slippery.",
        )
        # "if" → has_universal; conclusion words include "slippery" absent from premise.
        self.assertIsNone(result)

    def test_circular_reasoning_detected(self) -> None:
        """Conclusion content words are a strict subset of a premise's words → False."""
        result = self._check(
            ["The sky appears beautifully blue every morning."],
            "The sky is beautifully blue every morning.",
        )
        self.assertFalse(result)

    def test_not_circular_when_novel_word_in_conclusion(self) -> None:
        """A new content word in the conclusion breaks the circular check."""
        result = self._check(
            ["It is raining outside today."],
            "The ground is wet and muddy outside.",
        )
        # "muddy" not in premise → not circular; conclusion not universal → not hasty gen.
        self.assertIsNone(result)

    def test_returns_none_for_valid_argument(self) -> None:
        """Valid deductive chain with distinct premise/conclusion → None."""
        result = self._check(
            ["All metals conduct electricity.", "Copper is a metal."],
            "Copper conducts electricity.",
        )
        self.assertIsNone(result)

    def test_every_premise_prevents_hasty_gen_flag(self) -> None:
        """'Every' prefix counts as a universal premise."""
        result = self._check(
            ["Every student who studies passes."],
            "All students pass.",
        )
        # "every" matches universal pattern → not flagged as hasty gen.
        self.assertIsNone(result)


@unittest.skipUnless(_z3_available(), "z3-solver not installed")
class TestZ3ContextIntegration(unittest.TestCase):
    """Integration tests for _Z3Context parsing through verify_syllogism."""

    def _run(self, premises: list[str], conclusion: str):
        from pureason.reasoning.syllogism import verify_syllogism

        return verify_syllogism(premises, conclusion)

    def test_universal_rule_modus_ponens(self) -> None:
        report = self._run(
            ["All mammals are warm-blooded.", "Whales are mammals."],
            "Whales are warm-blooded.",
        )
        self.assertTrue(report.is_valid)

    def test_conditional_entailment(self) -> None:
        report = self._run(
            ["If it rains, the ground gets wet.", "It rains."],
            "The ground gets wet.",
        )
        self.assertIsNotNone(report)

    def test_entity_predicate_pattern(self) -> None:
        report = self._run(
            ["All metals conduct electricity.", "Copper is a metal."],
            "Copper conducts electricity.",
        )
        self.assertIsNotNone(report)

    def test_invalid_conclusion_detected(self) -> None:
        report = self._run(
            ["Alice passed the test.", "Bob passed the test."],
            "All students passed the test.",
        )
        self.assertFalse(report.is_valid)


# ---------------------------------------------------------------------------
# _Z3Context.parse_sentence — dep-tree pattern coverage
# ---------------------------------------------------------------------------


@unittest.skipUnless(_z3_available(), "z3-solver not installed")
class TestZ3ContextParseSentence(unittest.TestCase):
    """Unit tests for each grammatical pattern in _Z3Context.parse_sentence."""

    def _ctx(self, entities):
        from pureason.reasoning._z3ctx import _Z3Context

        return _Z3Context(entities)

    def _is_z3_expr(self, obj) -> bool:
        from z3 import is_expr

        return is_expr(obj)

    def test_empty_text_returns_none(self) -> None:
        ctx = self._ctx(["alice", "bob"])
        self.assertIsNone(ctx.parse_sentence(""))
        self.assertIsNone(ctx.parse_sentence("   "))

    def test_universal_all_are_count_per_entity(self) -> None:
        """'All X are Y' produces one Implies constraint per entity."""
        ctx = self._ctx(["alice", "bob"])
        result = ctx.parse_sentence("All students are hard-working.")
        self.assertIsNotNone(result)
        self.assertIsInstance(result, list)
        self.assertEqual(len(result), 2)
        self.assertTrue(all(self._is_z3_expr(c) for c in result))

    def test_universal_count_scales_with_entity_count(self) -> None:
        """Constraint count matches entity count."""
        ctx = self._ctx(["alice", "bob", "carol"])
        result = ctx.parse_sentence("All students are diligent.")
        self.assertIsNotNone(result)
        self.assertEqual(len(result), 3)

    def test_no_x_are_y_negated_universal(self) -> None:
        """'No X are Y' produces negated universal constraints."""
        ctx = self._ctx(["alice", "bob"])
        result = ctx.parse_sentence("No reptiles are mammals.")
        self.assertIsNotNone(result)
        self.assertIsInstance(result, list)
        self.assertEqual(len(result), 2)
        self.assertTrue(all(self._is_z3_expr(c) for c in result))

    def test_all_universal_vs_no_universal_differ(self) -> None:
        """'All X are Y' and 'No X are Y' produce structurally different Z3 expressions."""
        ctx = self._ctx(["alice", "bob"])
        pos = ctx.parse_sentence("All birds can fly.")
        neg = ctx.parse_sentence("No birds can fly.")
        self.assertIsNotNone(pos)
        self.assertIsNotNone(neg)
        self.assertNotEqual(str(pos[0]), str(neg[0]))

    def test_existential_some_creates_witness(self) -> None:
        """'Some X are Y' creates two entity-predicate atoms (witness pair)."""
        ctx = self._ctx(["alice", "bob"])
        result = ctx.parse_sentence("Some animals are dangerous.")
        self.assertIsNotNone(result)
        self.assertIsInstance(result, list)
        self.assertEqual(len(result), 2)
        self.assertTrue(all(self._is_z3_expr(c) for c in result))

    def test_conditional_if_then(self) -> None:
        """'If P, then Q' produces exactly one Implies constraint."""
        ctx = self._ctx(["alice", "bob"])
        result = ctx.parse_sentence("If it rains, the ground gets wet.")
        self.assertIsNotNone(result)
        self.assertIsInstance(result, list)
        self.assertEqual(len(result), 1)
        self.assertTrue(self._is_z3_expr(result[0]))

    def test_entity_specific_is_predicate(self) -> None:
        """'Alice is happy' produces one entity-predicate Z3 Bool."""
        ctx = self._ctx(["alice", "bob"])
        result = ctx.parse_sentence("Alice is happy.")
        self.assertIsNotNone(result)
        self.assertIsInstance(result, list)
        self.assertEqual(len(result), 1)
        self.assertTrue(self._is_z3_expr(result[0]))

    def test_entity_specific_negated_differs_from_positive(self) -> None:
        """Negation wraps the Z3 atom in Not()."""
        ctx = self._ctx(["alice", "bob"])
        pos_result = ctx.parse_sentence("Alice is happy.")
        neg_result = ctx.parse_sentence("Alice is not happy.")
        self.assertIsNotNone(pos_result)
        self.assertIsNotNone(neg_result)
        self.assertNotEqual(str(pos_result[0]), str(neg_result[0]))

    def test_entity_content_verb_intransitive(self) -> None:
        """'Alice wins' uses a content verb with no object."""
        ctx = self._ctx(["alice", "bob"])
        result = ctx.parse_sentence("Alice wins.")
        self.assertIsNotNone(result)
        self.assertEqual(len(result), 1)
        self.assertTrue(self._is_z3_expr(result[0]))

    def test_entity_content_verb_with_dobj(self) -> None:
        """'Alice likes Bob' uses content verb + direct object."""
        ctx = self._ctx(["alice", "bob"])
        result = ctx.parse_sentence("Alice likes Bob.")
        self.assertIsNotNone(result)
        self.assertEqual(len(result), 1)
        self.assertTrue(self._is_z3_expr(result[0]))

    def test_all_have_dobj(self) -> None:
        """'All X have Y' encodes possession as a universal constraint."""
        ctx = self._ctx(["alice", "bob"])
        result = ctx.parse_sentence("All students have a diploma.")
        self.assertIsNotNone(result)
        self.assertEqual(len(result), 2)

    def test_propositional_fallback_for_unknown_structure(self) -> None:
        """Sentences with no entity or quantifier fall back to a propositional atom."""
        ctx = self._ctx(["alice", "bob"])
        result = ctx.parse_sentence("The weather is pleasant today.")
        self.assertIsNotNone(result)
        self.assertIsInstance(result, list)
        self.assertEqual(len(result), 1)
        self.assertTrue(self._is_z3_expr(result[0]))

    def test_same_prop_key_for_logically_equivalent_atoms(self) -> None:
        """Two different phrasings of the same proposition share the same prop key."""
        ctx = self._ctx(["alice"])
        r1 = ctx.parse_sentence("The sky is blue.")
        r2 = ctx.parse_sentence("The sky is blue.")
        self.assertIsNotNone(r1)
        self.assertIsNotNone(r2)
        self.assertEqual(str(r1[0]), str(r2[0]))


if __name__ == "__main__":
    unittest.main()
