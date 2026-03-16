use epaper_dithering_core::enums::{DitherMode, GamutCompression, ToneCompression};
use epaper_dithering_core::palettes::ColorScheme;
use epaper_dithering_core::types::ImageBuffer;
use epaper_dithering_core::dither;
use wasm_bindgen::prelude::*;

fn parse_scheme(v: u8) -> Result<ColorScheme, JsValue> {
    ColorScheme::try_from(v).map_err(|e| JsValue::from_str(&e.to_string()))
}

fn parse_mode(v: u8) -> Result<DitherMode, JsValue> {
    DitherMode::try_from(v).map_err(|e| JsValue::from_str(&e.to_string()))
}

fn parse_tone(v: Option<f64>) -> ToneCompression {
    match v {
        None => ToneCompression::Auto,
        Some(s) => ToneCompression::Fixed(s),
    }
}

fn parse_gamut(v: Option<f64>) -> GamutCompression {
    match v {
        Some(s) if s > 0.0 => GamutCompression::Fixed(s),
        _ => GamutCompression::None,
    }
}

/// Dither a flat RGB image for e-paper display.
///
/// - `pixels`: flat RGB bytes, row-major (len = width × height × 3)
/// - `scheme_id`: firmware color scheme (0=mono … 7=grayscale16)
/// - `mode_id`: dither algorithm (0=none … 8=jjn), default 3 (Burkes)
/// - `serpentine`: alternate row direction, default true
/// - `tone_compression`: null = auto, 0.0 = off, 0.0–1.0 = fixed
/// - `gamut_compression`: null/0.0 = off, 0.0–1.0 = fixed
///
/// Returns a Uint8Array of palette indices (one per pixel).
#[wasm_bindgen]
pub fn dither_image(
    pixels: &[u8],
    width: usize,
    scheme_id: u8,
    mode_id: u8,
    serpentine: bool,
    tone_compression: Option<f64>,
    gamut_compression: Option<f64>,
) -> Result<Vec<u8>, JsValue> {
    let palette = parse_scheme(scheme_id)?.palette();
    let img = ImageBuffer::new(pixels, width);
    Ok(dither(
        &img,
        palette,
        parse_mode(mode_id)?,
        serpentine,
        parse_tone(tone_compression),
        parse_gamut(gamut_compression),
    ))
}
