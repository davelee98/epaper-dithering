use epaper_dithering_core::algorithms;
use epaper_dithering_core::palettes::ColorScheme;
use wasm_bindgen::prelude::*;

/// Resolve a firmware scheme int, or return a JS error string.
fn scheme(v: u8) -> Result<ColorScheme, JsValue> {
    ColorScheme::try_from(v)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Apply Floyd-Steinberg error diffusion dithering.
///
/// Returns a Uint8Array of palette indices (one per pixel).
/// Throws a string error if `scheme_id` is invalid.
#[wasm_bindgen]
pub fn floyd_steinberg(
    pixels: &[u8],
    width: usize,
    height: usize,
    scheme_id: u8,
    serpentine: bool,
) -> Result<Vec<u8>, JsValue> {
    Ok(algorithms::floyd_steinberg(pixels, width, height, scheme(scheme_id)?.palette(), serpentine))
}

#[wasm_bindgen]
pub fn atkinson(pixels: &[u8], width: usize, height: usize, scheme_id: u8, serpentine: bool) -> Result<Vec<u8>, JsValue> {
    Ok(algorithms::atkinson(pixels, width, height, scheme(scheme_id)?.palette(), serpentine))
}
#[wasm_bindgen]
pub fn burkes(pixels: &[u8], width: usize, height: usize, scheme_id: u8, serpentine: bool) -> Result<Vec<u8>, JsValue> {
    Ok(algorithms::burkes(pixels, width, height, scheme(scheme_id)?.palette(), serpentine))
}
#[wasm_bindgen]
pub fn stucki(pixels: &[u8], width: usize, height: usize, scheme_id: u8, serpentine: bool) -> Result<Vec<u8>, JsValue> {
    Ok(algorithms::stucki(pixels, width, height, scheme(scheme_id)?.palette(), serpentine))
}
#[wasm_bindgen]
pub fn sierra(pixels: &[u8], width: usize, height: usize, scheme_id: u8, serpentine: bool) -> Result<Vec<u8>, JsValue> {
    Ok(algorithms::sierra(pixels, width, height, scheme(scheme_id)?.palette(), serpentine))
}
#[wasm_bindgen]
pub fn sierra_lite(pixels: &[u8], width: usize, height: usize, scheme_id: u8, serpentine: bool) -> Result<Vec<u8>, JsValue> {
    Ok(algorithms::sierra_lite(pixels, width, height, scheme(scheme_id)?.palette(), serpentine))
}
#[wasm_bindgen]
pub fn jarvis_judice_ninke(pixels: &[u8], width: usize, height: usize, scheme_id: u8, serpentine: bool) -> Result<Vec<u8>, JsValue> {
    Ok(algorithms::jarvis_judice_ninke(pixels, width, height, scheme(scheme_id)?.palette(), serpentine))
}
