use epaper_dithering_core::algorithms;
use epaper_dithering_core::palettes::ColorScheme;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// Resolve a firmware scheme int to a ColorScheme, or raise ValueError.
fn scheme(v: u8) -> PyResult<ColorScheme> {
    ColorScheme::try_from(v)
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Apply Floyd-Steinberg error diffusion dithering.
///
/// Args:
///     pixels:     flat RGB bytes, row-major (len = width * height * 3)
///     width:      image width in pixels
///     height:     image height in pixels
///     scheme:     firmware color scheme int (0=mono, 1=bwr, …, 7=grayscale16)
///     serpentine: alternate row direction to reduce artifacts
///
/// Returns:
///     bytes of palette indices, one per pixel (len = width * height)
#[pyfunction]
fn floyd_steinberg(
    pixels: &[u8],
    width: usize,
    height: usize,
    scheme_id: u8,
    serpentine: bool,
) -> PyResult<Vec<u8>> {
    Ok(algorithms::floyd_steinberg(pixels, width, height, scheme(scheme_id)?.palette(), serpentine))
}

// The remaining kernels follow the exact same pattern — I've implemented them for you.

#[pyfunction]
fn atkinson(pixels: &[u8], width: usize, height: usize, scheme_id: u8, serpentine: bool) -> PyResult<Vec<u8>> {
    Ok(algorithms::atkinson(pixels, width, height, scheme(scheme_id)?.palette(), serpentine))
}
#[pyfunction]
fn burkes(pixels: &[u8], width: usize, height: usize, scheme_id: u8, serpentine: bool) -> PyResult<Vec<u8>> {
    Ok(algorithms::burkes(pixels, width, height, scheme(scheme_id)?.palette(), serpentine))
}
#[pyfunction]
fn stucki(pixels: &[u8], width: usize, height: usize, scheme_id: u8, serpentine: bool) -> PyResult<Vec<u8>> {
    Ok(algorithms::stucki(pixels, width, height, scheme(scheme_id)?.palette(), serpentine))
}
#[pyfunction]
fn sierra(pixels: &[u8], width: usize, height: usize, scheme_id: u8, serpentine: bool) -> PyResult<Vec<u8>> {
    Ok(algorithms::sierra(pixels, width, height, scheme(scheme_id)?.palette(), serpentine))
}
#[pyfunction]
fn sierra_lite(pixels: &[u8], width: usize, height: usize, scheme_id: u8, serpentine: bool) -> PyResult<Vec<u8>> {
    Ok(algorithms::sierra_lite(pixels, width, height, scheme(scheme_id)?.palette(), serpentine))
}
#[pyfunction]
fn jarvis_judice_ninke(pixels: &[u8], width: usize, height: usize, scheme_id: u8, serpentine: bool) -> PyResult<Vec<u8>> {
    Ok(algorithms::jarvis_judice_ninke(pixels, width, height, scheme(scheme_id)?.palette(), serpentine))
}

/// The Python module. Each function registered here becomes callable from Python.
#[pymodule]
fn epaper_dithering_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(floyd_steinberg, m)?)?;
    m.add_function(wrap_pyfunction!(atkinson, m)?)?;
    m.add_function(wrap_pyfunction!(burkes, m)?)?;
    m.add_function(wrap_pyfunction!(stucki, m)?)?;
    m.add_function(wrap_pyfunction!(sierra, m)?)?;
    m.add_function(wrap_pyfunction!(sierra_lite, m)?)?;
    m.add_function(wrap_pyfunction!(jarvis_judice_ninke, m)?)?;
    Ok(())
}
