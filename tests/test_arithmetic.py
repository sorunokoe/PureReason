"""Tests for arithmetic helper functions in pureason.reasoning."""

import os
import sys
import unittest

# Allow import even if reasoning.py is still monolithic or already a package
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))


def _import_arithmetic():
    """Import arithmetic helpers from wherever they live (file or subpackage)."""
    try:
        from pureason.reasoning.arithmetic import _detect_operation, _extract_numbers, _safe_eval
    except ImportError:
        from pureason.reasoning import (  # type: ignore[no-redef]
            _detect_operation,
            _extract_numbers,
            _safe_eval,
        )
    return _safe_eval, _extract_numbers, _detect_operation


def _import_repair():
    try:
        from pureason.reasoning.repair import (
            _extract_letter_answer,
            _extract_numeric_answer,
            _majority_vote,
            _majority_vote_letters,
            _repair_arithmetic_in_step,
        )
    except ImportError:
        from pureason.reasoning import (  # type: ignore[no-redef]
            _extract_letter_answer,
            _extract_numeric_answer,
            _majority_vote,
            _majority_vote_letters,
            _repair_arithmetic_in_step,
        )
    return (
        _repair_arithmetic_in_step,
        _extract_numeric_answer,
        _extract_letter_answer,
        _majority_vote,
        _majority_vote_letters,
    )


class TestSafeEval(unittest.TestCase):
    def setUp(self) -> None:
        self._safe_eval, _, _ = _import_arithmetic()

    def test_addition(self) -> None:
        self.assertAlmostEqual(self._safe_eval("2 + 3"), 5.0)

    def test_subtraction(self) -> None:
        self.assertAlmostEqual(self._safe_eval("10 - 4"), 6.0)

    def test_multiplication(self) -> None:
        self.assertAlmostEqual(self._safe_eval("6 * 7"), 42.0)

    def test_division(self) -> None:
        self.assertAlmostEqual(self._safe_eval("10 / 4"), 2.5)

    def test_division_by_zero(self) -> None:
        self.assertIsNone(self._safe_eval("5 / 0"))

    def test_power(self) -> None:
        self.assertAlmostEqual(self._safe_eval("2 ** 10"), 1024.0)

    def test_nested(self) -> None:
        self.assertAlmostEqual(self._safe_eval("(3 + 4) * 2"), 14.0)

    def test_invalid_expression(self) -> None:
        self.assertIsNone(self._safe_eval("import os"))

    def test_empty_string(self) -> None:
        self.assertIsNone(self._safe_eval(""))

    def test_negative_number(self) -> None:
        self.assertAlmostEqual(self._safe_eval("-5 + 3"), -2.0)


class TestExtractNumbers(unittest.TestCase):
    def setUp(self) -> None:
        _, self._extract_numbers, _ = _import_arithmetic()

    def test_extracts_integers(self) -> None:
        nums = self._extract_numbers("There are 3 apples and 10 bananas.")
        self.assertIn(3.0, nums)
        self.assertIn(10.0, nums)

    def test_extracts_decimals(self) -> None:
        nums = self._extract_numbers("The price is 3.14 dollars.")
        self.assertIn(3.14, nums)

    def test_extracts_word_numbers(self) -> None:
        nums = self._extract_numbers("There are two cats and three dogs.")
        self.assertIn(2.0, nums)
        self.assertIn(3.0, nums)

    def test_negative_numbers(self) -> None:
        nums = self._extract_numbers("Temperature is -5 degrees.")
        self.assertIn(-5.0, nums)

    def test_no_numbers(self) -> None:
        nums = self._extract_numbers("No numeric content here.")
        self.assertEqual(nums, [])


class TestDetectOperation(unittest.TestCase):
    def setUp(self) -> None:
        _, _, self._detect_operation = _import_arithmetic()

    def test_addition_keywords(self) -> None:
        op = self._detect_operation("How many total items if we add 3 more?")
        self.assertEqual(op, "+")

    def test_subtraction_remaining(self) -> None:
        op = self._detect_operation("How many are left after removing 5?")
        self.assertEqual(op, "-")

    def test_division_speed(self) -> None:
        op = self._detect_operation("What is the average speed?")
        self.assertEqual(op, "/")

    def test_multiplication_rate_scaling(self) -> None:
        op = self._detect_operation(
            "A car travels 60 miles per hour for 4 hours. How far does it travel?"
        )
        self.assertEqual(op, "*")


class TestRepairArithmetic(unittest.TestCase):
    def setUp(self) -> None:
        (
            self._repair,
            self._extract_num,
            self._extract_letter,
            self._majority_vote,
            self._majority_vote_letters,
        ) = _import_repair()

    def test_correct_expression_unchanged(self) -> None:
        step = "We have 3 + 4 = 7 apples."
        result = self._repair(step)
        self.assertNotIn("[repaired]", result)

    def test_wrong_addition_repaired(self) -> None:
        step = "We have 3 + 4 = 8 apples."
        result = self._repair(step)
        self.assertIn("[repaired]", result)
        self.assertIn("7", result)

    def test_wrong_multiplication_repaired(self) -> None:
        step = "6 * 7 = 41"
        result = self._repair(step)
        self.assertIn("[repaired]", result)
        self.assertIn("42", result)

    def test_extract_numeric_answer_at_end(self) -> None:
        text = "The answer is 42."
        val = self._extract_num(text)
        self.assertEqual(val, 42.0)

    def test_extract_numeric_answer_none(self) -> None:
        val = self._extract_num("No number here at all.")
        self.assertIsNone(val)

    def test_extract_letter_answer_a(self) -> None:
        text = "Therefore the answer is A."
        letter = self._extract_letter(text)
        self.assertEqual(letter, "A")

    def test_extract_letter_answer_bold(self) -> None:
        text = "After analysis, the best answer is **B**."
        letter = self._extract_letter(text)
        self.assertEqual(letter, "B")

    def test_extract_letter_answer_none(self) -> None:
        text = "No clear MCQ answer here."
        letter = self._extract_letter(text)
        self.assertIsNone(letter)

    def test_majority_vote_letters(self) -> None:
        answers = ["A", "B", "A", "A", "C"]
        self.assertEqual(self._majority_vote_letters(answers), "A")

    def test_majority_vote_letters_none_ignored(self) -> None:
        answers = [None, "B", None, "B"]
        self.assertEqual(self._majority_vote_letters(answers), "B")

    def test_majority_vote_numeric(self) -> None:
        answers = [42.0, 42.0, 41.0, 42.0]
        self.assertEqual(self._majority_vote(answers), 42.0)

    def test_majority_vote_empty(self) -> None:
        self.assertIsNone(self._majority_vote([]))
        self.assertIsNone(self._majority_vote_letters([]))


class TestDetectOperationExtended(unittest.TestCase):
    """Additional verb-lemma and structural coverage for _detect_operation."""

    def setUp(self) -> None:
        _, _, self._detect_operation = _import_arithmetic()

    def test_subtraction_sell_verb(self) -> None:
        op = self._detect_operation("Maria sold 5 apples.")
        self.assertEqual(op, "-")

    def test_subtraction_spend_verb(self) -> None:
        op = self._detect_operation("They spent 30 dollars.")
        self.assertEqual(op, "-")

    def test_subtraction_remove_verb(self) -> None:
        op = self._detect_operation("He removed 7 items from the shelf.")
        self.assertEqual(op, "-")

    def test_subtraction_use_verb(self) -> None:
        op = self._detect_operation("She used 3 liters of water.")
        self.assertEqual(op, "-")

    def test_addition_earn_verb(self) -> None:
        op = self._detect_operation("She earned 50 dollars this month.")
        self.assertEqual(op, "+")

    def test_addition_receive_verb(self) -> None:
        op = self._detect_operation("He received 3 packages today.")
        self.assertEqual(op, "+")

    def test_addition_altogether_signal(self) -> None:
        op = self._detect_operation("How many items are there altogether?")
        self.assertEqual(op, "+")

    def test_division_split_verb(self) -> None:
        op = self._detect_operation("They split the 100 dollars equally.")
        self.assertEqual(op, "/")

    def test_division_per_token(self) -> None:
        # Sentence with no NUM token so structural path doesn't fire before "per"
        op = self._detect_operation("Costs are calculated per kilometer.")
        self.assertEqual(op, "/")

    def test_multiplication_produce_verb(self) -> None:
        op = self._detect_operation("The factory produces 50 units.")
        self.assertEqual(op, "*")

    def test_returns_string_not_none_for_known_patterns(self) -> None:
        op = self._detect_operation("How many total apples after adding 5 more?")
        self.assertIsNotNone(op)


class TestExtractNumbersExtended(unittest.TestCase):
    """Edge-case coverage for _extract_numbers."""

    def setUp(self) -> None:
        _, self._extract_numbers, _ = _import_arithmetic()

    def test_hyphenated_word_number(self) -> None:
        nums = self._extract_numbers("There are twenty-three cats.")
        self.assertIn(23.0, nums)

    def test_comma_separated_number(self) -> None:
        nums = self._extract_numbers("The factory produced 1,000 units.")
        self.assertIn(1000.0, nums)

    def test_multiple_word_numbers(self) -> None:
        nums = self._extract_numbers("three dogs and two cats")
        self.assertIn(3.0, nums)
        self.assertIn(2.0, nums)

    def test_word_number_forty_five(self) -> None:
        nums = self._extract_numbers("forty-five participants joined.")
        self.assertIn(45.0, nums)

    def test_no_double_counting_integer(self) -> None:
        # "3" already extracted by digit regex; word path should not add extra
        nums = self._extract_numbers("There are 3 apples.")
        digit_occurrences = sum(1 for n in nums if n == 3.0)
        self.assertEqual(digit_occurrences, 1)


if __name__ == "__main__":
    unittest.main()
