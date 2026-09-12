//! PyO3 binding layer: result types, finance/signal wrappers, and the grouped
//! FeatureEngine. Python is the control/API layer; computation lives in Rust.

pub mod engine;
pub mod experimental;
pub mod finance;
pub mod finance_experimental;
pub mod result_types;
pub mod signal;
pub mod spectral_missing;

use pyo3::exceptions::PyValueError;
use pyo3::PyResult;

/// Validate a scalar input is finite (rejects NaN / +-Inf).
#[inline]
pub fn chk(v: f64) -> PyResult<()> {
    if v.is_finite() {
        Ok(())
    } else {
        Err(PyValueError::new_err("input must be finite (got NaN or Inf)"))
    }
}

/// Validate two scalar inputs.
#[inline]
pub fn chk2(a: f64, b: f64) -> PyResult<()> {
    if a.is_finite() && b.is_finite() {
        Ok(())
    } else {
        Err(PyValueError::new_err("inputs must be finite (got NaN or Inf)"))
    }
}

/// Validate a slice of inputs.
#[inline]
pub fn chk_slice(s: &[f64]) -> PyResult<()> {
    for &v in s {
        if !v.is_finite() {
            return Err(PyValueError::new_err("all inputs must be finite (got NaN or Inf)"));
        }
    }
    Ok(())
}

/// Map a positive-integer parameter error.
#[inline]
pub fn chk_pos(v: usize, name: &str) -> PyResult<()> {
    if v > 0 {
        Ok(())
    } else {
        Err(PyValueError::new_err(format!("{name} must be > 0")))
    }
}
