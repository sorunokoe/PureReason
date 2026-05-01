//! # Specialized Mathematical Solver
//!
//! Medium Win #6: Accurate Arithmetic & Calculation Validation
//!
//! This module verifies mathematical claims by:
//! - Parsing and evaluating arithmetic expressions
//! - Checking order of operations (PEMDAS)
//! - Detecting calculation errors in claims
//! - Validating financial/percentage calculations
//! - Supporting multi-step derivations
//!
//! Key improvements:
//! - Finance domain: Catch miscalculated returns, ratios, NPV
//! - Science domain: Validate unit conversions, formula applications
//! - Code domain: Verify loop counts, array indexing math
//! - Medical domain: Dosage calculations (mg/kg, concentration)

use regex::Regex;
use serde::{Deserialize, Serialize};

/// A single mathematical operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MathOp {
    /// Addition
    Add,
    /// Subtraction
    Sub,
    /// Multiplication
    Mul,
    /// Division
    Div,
    /// Exponentiation
    Pow,
    /// Modulo (remainder)
    Mod,
}

impl MathOp {
    /// Apply operation to two numbers.
    pub fn apply(&self, a: f64, b: f64) -> Result<f64, String> {
        match self {
            MathOp::Add => Ok(a + b),
            MathOp::Sub => Ok(a - b),
            MathOp::Mul => Ok(a * b),
            MathOp::Div => {
                if b.abs() < 1e-10 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(a / b)
                }
            }
            MathOp::Pow => Ok(a.powf(b)),
            MathOp::Mod => {
                if b.abs() < 1e-10 {
                    Err("Modulo by zero".to_string())
                } else {
                    Ok(a % b)
                }
            }
        }
    }

    /// Get operator precedence (higher = earlier evaluation).
    pub fn precedence(&self) -> usize {
        match self {
            MathOp::Add | MathOp::Sub => 1,
            MathOp::Mul | MathOp::Div | MathOp::Mod => 2,
            MathOp::Pow => 3,
        }
    }
}

/// A mathematical claim to verify (e.g., "2 + 2 = 4").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MathClaim {
    /// Left side expression (e.g., "2 + 2")
    pub left_expr: String,
    /// Right side value or expression (e.g., "4")
    pub right_expr: String,
    /// Claimed result
    pub claimed_result: f64,
    /// Calculated correct result
    pub correct_result: f64,
    /// Whether claim is mathematically correct
    pub is_correct: bool,
    /// Absolute error magnitude
    pub error_magnitude: f64,
    /// Relative error (as percentage)
    pub relative_error: f64,
    /// Confidence in claim (0.0-1.0)
    pub confidence: f64,
    /// Human-readable explanation
    pub explanation: String,
}

/// Mathematical solver for validating claims.
pub struct MathSolver;

impl MathSolver {
    /// Evaluate a simple mathematical expression.
    pub fn evaluate(expr: &str) -> Result<f64, String> {
        let expr = expr.trim().replace(" ", "");
        Self::parse_expression(&expr)
    }

    /// Parse and evaluate expression with proper operator precedence.
    fn parse_expression(expr: &str) -> Result<f64, String> {
        if expr.is_empty() {
            return Err("Empty expression".to_string());
        }

        // Try to parse as number first
        if let Ok(num) = expr.parse::<f64>() {
            return Ok(num);
        }

        let chars: Vec<char> = expr.chars().collect();

        // Find operator with lowest precedence (evaluate last)
        let mut min_precedence = usize::MAX;
        let mut split_pos = None;

        let mut paren_depth = 0;
        for i in (0..chars.len()).rev() {
            match chars[i] {
                ')' => paren_depth += 1,
                '(' => paren_depth -= 1,
                '+' | '-' if paren_depth == 0 => {
                    // Minus is a sign if it follows an operator or paren, otherwise it's subtraction
                    let is_sign = i > 0 && "+-*/(^%".contains(chars[i - 1]);
                    if !is_sign && min_precedence > 1 {
                        min_precedence = 1;
                        split_pos = Some((i, chars[i]));
                    }
                }
                '*' | '/' | '%' if paren_depth == 0 => {
                    if min_precedence > 2 {
                        min_precedence = 2;
                        split_pos = Some((i, chars[i]));
                    }
                }
                '^' if paren_depth == 0 => {
                    if min_precedence > 3 {
                        min_precedence = 3;
                        split_pos = Some((i, chars[i]));
                    }
                }
                _ => {}
            }
        }

        if let Some((pos, op_char)) = split_pos {
            let left = Self::parse_expression(&expr[..pos])?;
            let right = Self::parse_expression(&expr[pos + 1..])?;

            let op = match op_char {
                '+' => MathOp::Add,
                '-' => MathOp::Sub,
                '*' => MathOp::Mul,
                '/' => MathOp::Div,
                '%' => MathOp::Mod,
                '^' => MathOp::Pow,
                _ => return Err("Unknown operator".to_string()),
            };

            return op.apply(left, right);
        }

        // Remove parentheses
        if expr.starts_with('(') && expr.ends_with(')') {
            return Self::parse_expression(&expr[1..expr.len() - 1]);
        }

        // Try parsing negative number
        if expr.starts_with('-') {
            let num: f64 = expr
                .parse()
                .map_err(|_| format!("Cannot parse as number: {}", expr))?;
            return Ok(num);
        }

        Err(format!("Cannot parse: {}", expr))
    }

    /// Check if a mathematical claim is correct.
    pub fn verify_claim(left: &str, right: &str, claimed_result: f64) -> MathClaim {
        let correct_result = match Self::evaluate(left) {
            Ok(val) => val,
            Err(_) => {
                return MathClaim {
                    left_expr: left.to_string(),
                    right_expr: right.to_string(),
                    claimed_result,
                    correct_result: f64::NAN,
                    is_correct: false,
                    error_magnitude: f64::NAN,
                    relative_error: f64::NAN,
                    confidence: 0.0,
                    explanation: "Failed to parse left expression".to_string(),
                }
            }
        };

        let error_magnitude = (correct_result - claimed_result).abs();
        let relative_error = if correct_result.abs() > 1e-10 {
            (error_magnitude / correct_result.abs()) * 100.0
        } else {
            0.0
        };

        let is_correct = error_magnitude < 1e-6;
        let confidence = if is_correct {
            0.99
        } else if relative_error < 1.0 {
            0.85
        } else if relative_error < 5.0 {
            0.60
        } else {
            0.20
        };

        let explanation = if is_correct {
            format!("Calculation correct: {} = {:.6}", left, correct_result)
        } else if relative_error < 1.0 {
            format!(
                "Minor arithmetic discrepancy: expected {:.6}, got {:.6} (error: {:.2}%)",
                correct_result, claimed_result, relative_error
            )
        } else {
            format!(
                "Significant calculation error: expected {:.6}, got {:.6} (error: {:.2}%)",
                correct_result, claimed_result, relative_error
            )
        };

        MathClaim {
            left_expr: left.to_string(),
            right_expr: right.to_string(),
            claimed_result,
            correct_result,
            is_correct,
            error_magnitude,
            relative_error,
            confidence,
            explanation,
        }
    }

    /// Extract numbers from a text claim.
    pub fn extract_numbers(text: &str) -> Vec<f64> {
        let re = Regex::new(r"-?\d+\.?\d*").unwrap();
        re.find_iter(text)
            .filter_map(|m| m.as_str().parse::<f64>().ok())
            .collect()
    }

    /// Check if a percentage claim is reasonable.
    pub fn verify_percentage(part: f64, whole: f64, claimed_percent: f64) -> (bool, f64, String) {
        if whole.abs() < 1e-10 {
            return (
                false,
                0.0,
                "Cannot calculate percentage with zero whole".to_string(),
            );
        }

        let correct_percent = (part / whole) * 100.0;
        let error = (correct_percent - claimed_percent).abs();
        let is_correct = error < 0.1;

        let explanation = if is_correct {
            format!("{} / {} = {:.2}%", part, whole, correct_percent)
        } else {
            format!(
                "Percentage error: claimed {:.2}%, actual {:.2}% (error: {:.2}%)",
                claimed_percent, correct_percent, error
            )
        };

        (is_correct, error, explanation)
    }

    /// Verify a financial calculation (e.g., compound interest).
    pub fn verify_compound_interest(
        principal: f64,
        rate: f64,
        years: f64,
        claimed_result: f64,
    ) -> (bool, f64, String) {
        let correct_result = principal * (1.0 + rate / 100.0).powf(years);
        let error = (correct_result - claimed_result).abs();
        let relative_error = if correct_result.abs() > 1e-10 {
            (error / correct_result.abs()) * 100.0
        } else {
            0.0
        };

        let is_correct = relative_error < 1.0;
        let explanation = if is_correct {
            format!(
                "Compound interest: ${:.2} @ {:.2}% for {:.1}y = ${:.2}",
                principal, rate, years, correct_result
            )
        } else {
            format!(
                "Interest calculation error: expected ${:.2}, got ${:.2} (error: {:.2}%)",
                correct_result, claimed_result, relative_error
            )
        };

        (is_correct, error, explanation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_addition() {
        let result = MathSolver::evaluate("2 + 3").unwrap();
        assert!((result - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_simple_subtraction() {
        let result = MathSolver::evaluate("10 - 3").unwrap();
        assert!((result - 7.0).abs() < 1e-6);
    }

    #[test]
    fn test_simple_multiplication() {
        let result = MathSolver::evaluate("4 * 5").unwrap();
        assert!((result - 20.0).abs() < 1e-6);
    }

    #[test]
    fn test_simple_division() {
        let result = MathSolver::evaluate("20 / 4").unwrap();
        assert!((result - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_order_of_operations() {
        let result = MathSolver::evaluate("2 + 3 * 4").unwrap();
        assert!((result - 14.0).abs() < 1e-6); // 3*4 first, then +2
    }

    #[test]
    fn test_parentheses() {
        let result = MathSolver::evaluate("(2 + 3) * 4").unwrap();
        assert!((result - 20.0).abs() < 1e-6);
    }

    #[test]
    fn test_exponentiation() {
        let result = MathSolver::evaluate("2 ^ 3").unwrap();
        assert!((result - 8.0).abs() < 1e-6);
    }

    #[test]
    fn test_division_by_zero() {
        let result = MathSolver::evaluate("5 / 0");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_correct_claim() {
        let claim = MathSolver::verify_claim("2 + 2", "4", 4.0);
        assert!(claim.is_correct);
        assert!(claim.confidence >= 0.90);
    }

    #[test]
    fn test_verify_incorrect_claim() {
        let claim = MathSolver::verify_claim("2 + 2", "5", 5.0);
        assert!(!claim.is_correct);
        assert!(claim.confidence < 0.5);
    }

    #[test]
    fn test_extract_numbers() {
        let numbers = MathSolver::extract_numbers("The price is $42.50 and quantity is 3");
        assert_eq!(numbers.len(), 2);
        assert!((numbers[0] - 42.50).abs() < 1e-6);
        assert!((numbers[1] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_percentage_calculation() {
        let (is_correct, error, _) = MathSolver::verify_percentage(25.0, 100.0, 25.0);
        assert!(is_correct);
        assert!(error < 0.1);
    }

    #[test]
    fn test_compound_interest() {
        // $1000 at 10% for 1 year = $1100
        let (is_correct, _, _) = MathSolver::verify_compound_interest(1000.0, 10.0, 1.0, 1100.0);
        assert!(is_correct);
    }

    #[test]
    fn test_negative_numbers() {
        let result = MathSolver::evaluate("-5 + 3").unwrap();
        assert!((result - (-2.0)).abs() < 1e-6);
    }

    #[test]
    fn test_complex_expression() {
        // (5 + 3) * 2 - 4 / 2 = 8*2 - 2 = 16 - 2 = 14
        let result = MathSolver::evaluate("(5 + 3) * 2 - 4 / 2").unwrap();
        assert!((result - 14.0).abs() < 1e-6);
    }
}
