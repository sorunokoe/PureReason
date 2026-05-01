//! # PureReason Python SDK (S-III-3)
//!
//! PyO3-based Python extension module for the PureReason Kantian reasoning system.
//!
// PyO3's PyResult<T> type alias expansion triggers clippy::useless_conversion in some
// toolchain versions — this is a known false positive from macro-generated code.
#![allow(clippy::useless_conversion)]
//!
//! ## Build
//! ```bash
//! pip install maturin
//! maturin develop   # install in the current virtualenv
//! maturin build     # build a .whl for distribution
//! ```
//!
//! ## Usage
//! ```python
//! import pure_reason
//!
//! pr = pure_reason.PureReason()
//!
//! # Epistemic Confidence Score — the primary calibration endpoint
//! result = pr.calibrate("The patient must have cancer.")
//! print(result["ecs"])          # e.g. 28
//! print(result["band"])         # "Low"
//! print(result["flags"])        # ["Epistemic overreach: ..."]
//! print(result["safe_version"]) # rewritten in regulative language
//!
//! # Full pipeline analysis → dict
//! result = pr.analyze("God exists necessarily.")
//! print(result["verdict"]["risk"])  # "HIGH"
//!
//! # Validation certificate → dict
//! cert = pr.certify("Water boils at 100 degrees.")
//! print(cert["content_hash"])       # BLAKE3 hex fingerprint
//!
//! # Regulative transformation → str
//! regulated = pr.regulate("The soul is an immortal substance.")
//! print(regulated)
//!
//! # Quick validation summary → dict
//! v = pr.validate("The universe had a beginning in time.")
//! print(v["risk_level"], v["has_illusions"])
//!
//! # Claim-first analysis → dict
//! claims = pr.claims("Knowledge: The capital of Australia is Canberra.\nAnswer: Sydney is the capital of Australia.")
//! print(claims["contradicted_count"])
//! ```

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use pure_reason_core::{
    calibration::PipelineCalibration, certificate::ValidationCertificate, claims::annotate_claims,
    pipeline::KantianPipeline,
};

// ─── PureReason Python class ──────────────────────────────────────────────────

/// The PureReason Kantian reasoning engine.
///
/// Wraps `KantianPipeline` and exposes it as a Python class.
/// All methods accept a `str` input and return `dict` or `str` results.
#[pyclass]
struct PureReason {
    _pipeline: (), // KantianPipeline is stateless; we create one per call to stay Send+Sync
}

#[pymethods]
impl PureReason {
    #[new]
    fn new() -> Self {
        Self { _pipeline: () }
    }

    /// Compute the Epistemic Confidence Score (ECS) for text.
    ///
    /// This is the primary calibration endpoint — the one most callers should use.
    /// Returns a dict with: ecs (int 0–100), band, calibrated (bool), flags, safe_version,
    /// epistemic_mode, and score_breakdown.
    ///
    /// Args:
    ///     text (str): The LLM output to calibrate.
    ///
    /// Returns:
    ///     dict: {ecs, band, calibrated, flags, safe_version, epistemic_mode, score_breakdown}
    ///
    /// Raises:
    ///     ValueError: If the pipeline fails to process the text.
    ///
    /// Example:
    ///     >>> pr = PureReason()
    ///     >>> r = pr.calibrate("The patient must have cancer.")
    ///     >>> r["ecs"]           # 28
    ///     >>> r["calibrated"]    # False
    ///     >>> r["safe_version"]  # rewritten in regulative language
    fn calibrate(&self, py: Python, text: &str) -> PyResult<PyObject> {
        let pipeline = KantianPipeline::new();
        let report = pipeline
            .process(text)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        let cal = report.calibration();
        let json = serde_json::to_string(&cal).map_err(|e| PyValueError::new_err(e.to_string()))?;
        json_str_to_pyobject(py, &json)
    }

    /// Run the full Kantian pipeline and return a dict with the complete PipelineReport.
    ///
    /// Args:
    ///     text (str): The text to analyze.
    ///
    /// Returns:
    ///     dict: Full analysis including category scores, dialectical report, and verdict.
    ///
    /// Raises:
    ///     ValueError: If the pipeline fails to process the text.
    fn analyze(&self, py: Python, text: &str) -> PyResult<PyObject> {
        let pipeline = KantianPipeline::new();
        let report = pipeline
            .process(text)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        let json =
            serde_json::to_string(&report).map_err(|e| PyValueError::new_err(e.to_string()))?;
        json_str_to_pyobject(py, &json)
    }

    /// Generate a content-addressed ValidationCertificate for the text.
    ///
    /// Args:
    ///     text (str): The text to certify.
    ///
    /// Returns:
    ///     dict: Certificate with content_hash, risk_level, issues, etc.
    fn certify(&self, py: Python, text: &str) -> PyResult<PyObject> {
        let pipeline = KantianPipeline::new();
        let report = pipeline
            .process(text)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        let cert = ValidationCertificate::from_report(&report);
        let json =
            serde_json::to_string(&cert).map_err(|e| PyValueError::new_err(e.to_string()))?;
        json_str_to_pyobject(py, &json)
    }

    /// Apply regulative transformation to epistemic overreach in the text.
    ///
    /// Args:
    ///     text (str): The text to transform.
    ///
    /// Returns:
    ///     str: The regulated text (identical to input if no issues found).
    fn regulate(&self, text: &str) -> PyResult<String> {
        let pipeline = KantianPipeline::new();
        let report = pipeline
            .process(text)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(report.regulated_text)
    }

    /// Quick validation: risk level, issue flags, and one-sentence summary.
    ///
    /// Args:
    ///     text (str): The text to validate.
    ///
    /// Returns:
    ///     dict: {risk_level, has_illusions, has_contradictions, has_paralogisms, summary}
    fn validate(&self, py: Python, text: &str) -> PyResult<PyObject> {
        let pipeline = KantianPipeline::new();
        let report = pipeline
            .process(text)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        let json = serde_json::json!({
            "risk_level": report.verdict.risk.to_string(),
            "has_illusions": report.verdict.has_illusions,
            "has_contradictions": report.verdict.has_contradictions,
            "has_paralogisms": report.verdict.has_paralogisms,
            "summary": report.summary,
            "regulated_text": report.regulated_text,
        })
        .to_string();
        json_str_to_pyobject(py, &json)
    }

    /// Claim-first analysis: per-claim risk, modality, and local evidence binding.
    ///
    /// Args:
    ///     text (str): The text to decompose into claims.
    ///
    /// Returns:
    ///     dict: ClaimAnnotatedReport with claim-level structure and evidence binding.
    fn claims(&self, py: Python, text: &str) -> PyResult<PyObject> {
        let report = annotate_claims(text).map_err(|e| PyValueError::new_err(e.to_string()))?;
        let json =
            serde_json::to_string(&report).map_err(|e| PyValueError::new_err(e.to_string()))?;
        json_str_to_pyobject(py, &json)
    }

    /// Compute the BLAKE3 content hash of a string (no pipeline run needed).
    ///
    /// Args:
    ///     text (str): The text to fingerprint.
    ///
    /// Returns:
    ///     str: 32-char hex BLAKE3 fingerprint (first 16 bytes of digest).
    fn hash_content(&self, text: &str) -> String {
        pure_reason_core::certificate::blake3_hex(text)
    }

    /// Verify that a hash matches the given text.
    ///
    /// Args:
    ///     text (str): The original text.
    ///     expected_hash (str): The expected BLAKE3 hex hash to verify against.
    ///
    /// Returns:
    ///     bool: True if the hash matches.
    fn verify_hash(&self, text: &str, expected_hash: &str) -> bool {
        ValidationCertificate::verify(text, expected_hash)
    }
}

// ─── Module-level functions ───────────────────────────────────────────────────

/// Wrap an LLM callable with automatic epistemic calibration (S14).
///
/// The wrapped function is called with `prompt`, its output is automatically
/// calibrated through the full Kantian pipeline, and a rich `CalibratedResult`
/// dict is returned containing the original output, regulated version, ECS,
/// flags, and score breakdown.
///
/// This is the zero-friction integration path: drop it around any LLM call.
///
/// Args:
///     llm_fn (callable): A Python callable that accepts a prompt (str) and
///                        returns the LLM output (str).
///     prompt (str): The prompt to pass to the LLM.
///
/// Returns:
///     dict: {
///         prompt (str): The original prompt.
///         original (str): The raw LLM output.
///         regulated (str): Output rewritten in regulative language (if issues found).
///         ecs (int): Epistemic Confidence Score 0–100.
///         band (str): "Critical"/"Low"/"Moderate"/"High"/"Full".
///         calibrated (bool): True when ecs >= 70.
///         flags (list[str]): Human-readable issue descriptions.
///         score_breakdown (dict): Per-dimension scores.
///         prior_matched (bool): True when a known misconception prior fired.
///     }
///
/// Raises:
///     ValueError: If the LLM callable raises or the pipeline fails.
///
/// Example:
///     >>> pr = pure_reason.PureReason()
///     >>> def my_llm(prompt):
///     ...     return "The earth is flat."
///     >>> result = pure_reason.wrap_llm(my_llm, "Is the earth round?")
///     >>> result["ecs"]        # e.g. 12
///     >>> result["calibrated"] # False
///     >>> result["flags"]      # ["Transcendental illusion: flat earth detected"]
#[pyfunction]
fn wrap_llm(py: Python, llm_fn: PyObject, prompt: &str) -> PyResult<PyObject> {
    // Call the user's LLM function
    let raw_output: String = llm_fn
        .call1(py, (prompt,))
        .map_err(|e| PyValueError::new_err(format!("LLM callable raised: {e}")))?
        .extract(py)
        .map_err(|e| PyValueError::new_err(format!("LLM callable must return str: {e}")))?;

    // Run the Kantian pipeline over the output
    let pipeline = pure_reason_core::pipeline::KantianPipeline::new();
    let report = pipeline
        .process(&raw_output)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let cal = report.calibration();

    let json = serde_json::json!({
        "prompt":          prompt,
        "original":        raw_output,
        "regulated":       report.regulated_text,
        "ecs":             cal.ecs,
        "band":            serde_json::to_value(cal.band).unwrap_or(serde_json::Value::Null),
        "calibrated":      cal.calibrated,
        "flags":           cal.flags,
        "score_breakdown": cal.score_breakdown,
        "prior_matched":   report.verdict.prior_matched,
        "risk":            report.verdict.risk.to_string(),
    })
    .to_string();

    json_str_to_pyobject(py, &json)
}

/// Calibrate a raw string without calling any LLM.
///
/// Convenience module-level function. Identical to `PureReason().calibrate(text)`.
///
/// Returns:
///     dict: {ecs, band, calibrated, flags, safe_version, epistemic_mode, score_breakdown}
#[pyfunction]
fn calibrate(py: Python, text: &str) -> PyResult<PyObject> {
    let pipeline = pure_reason_core::pipeline::KantianPipeline::new();
    let report = pipeline
        .process(text)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let cal = report.calibration();
    let json = serde_json::to_string(&cal).map_err(|e| PyValueError::new_err(e.to_string()))?;
    json_str_to_pyobject(py, &json)
}

// ─── Module registration ──────────────────────────────────────────────────────

#[pymodule]
fn pure_reason(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PureReason>()?;
    m.add_function(wrap_pyfunction!(wrap_llm, m)?)?;
    m.add_function(wrap_pyfunction!(calibrate, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add(
        "__doc__",
        "PureReason: Kant's Critique of Pure Reason as a Python reasoning library",
    )?;
    Ok(())
}

// ─── JSON string → Python object bridge ──────────────────────────────────────

/// Parse a JSON string into a Python object via `json.loads`.
///
/// This approach is stable across PyO3 versions since it delegates to
/// Python's own `json` module rather than manually constructing Python objects.
fn json_str_to_pyobject(py: Python, json: &str) -> PyResult<PyObject> {
    let json_mod = py.import_bound("json")?;
    let result = json_mod.call_method1("loads", (json,))?;
    Ok(result.into())
}
