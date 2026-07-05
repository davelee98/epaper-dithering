use epaper_dithering_core::{
    dither, dither_with_canonical, DitherConfig,
    enums::{DitherMode, GamutCompression, ToneCompression},
    measured_palettes::CATALOG,
    palettes::{ColorScheme, Palette},
    types::ImageBuffer,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

fn parse_mode(v: u8) -> PyResult<DitherMode> {
    DitherMode::try_from(v).map_err(|e| PyValueError::new_err(e.to_string()))
}

fn parse_tone(v: Option<f64>) -> ToneCompression {
    match v {
        None    => ToneCompression::Auto,
        Some(s) => ToneCompression::Fixed(s),
    }
}

fn parse_gamut(v: Option<f64>) -> GamutCompression {
    match v {
        None              => GamutCompression::Auto,
        Some(s) if s > 0.0 => GamutCompression::Fixed(s),
        _                 => GamutCompression::None,
    }
}

/// Dither a flat RGB image.
///
/// Pass either `scheme_id` (idealized color scheme) or `palette_bytes` + `accent_idx`
/// (measured palette). If both are given, `palette_bytes` wins.
#[pyfunction]
#[pyo3(signature = (
    pixels, width, height, *,
    scheme_id=None, palette_bytes=None, accent_idx=0,
    mode_id=1, serpentine=true,
    exposure=1.0, saturation=1.0, shadows=0.0, highlights=0.0,
    tone=0.0, gamut=0.0,
))]
#[allow(clippy::too_many_arguments)]
fn dither_image(
    pixels: &[u8],
    width: usize,
    height: usize,
    scheme_id: Option<u8>,
    palette_bytes: Option<&[u8]>,
    accent_idx: usize,
    mode_id: u8,
    serpentine: bool,
    exposure: f64,
    saturation: f64,
    shadows: f64,
    highlights: f64,
    tone: Option<f64>,
    gamut: Option<f64>,
) -> PyResult<Vec<u8>> {
    // Validate the buffer against the caller's dimensions instead of silently
    // deriving height and truncating trailing pixels on a width/length mismatch.
    // Layout is flat RGB: len = width × height × 3.
    let expected = width
        .checked_mul(height)
        .and_then(|n| n.checked_mul(3))
        .ok_or_else(|| PyValueError::new_err("width × height × 3 overflows usize"))?;
    if pixels.len() != expected {
        return Err(PyValueError::new_err(format!(
            "pixel buffer length ({}) does not match width × height × 3 ({} × {} × 3 = {})",
            pixels.len(),
            width,
            height,
            expected,
        )));
    }
    let img = ImageBuffer::new(pixels, width);
    let config = DitherConfig {
        mode: parse_mode(mode_id)?,
        serpentine,
        exposure,
        saturation,
        shadows,
        highlights,
        tone:  parse_tone(tone),
        gamut: parse_gamut(gamut),
    };

    match (palette_bytes, scheme_id) {
        (Some(bytes), scheme_id) => {
            if !bytes.len().is_multiple_of(3) {
                return Err(PyValueError::new_err("palette_bytes length must be a multiple of 3"));
            }
            let colors: Vec<[u8; 3]> = bytes.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect();
            let palette = Palette::new(colors, accent_idx);
            if let Some(id) = scheme_id {
                let scheme = ColorScheme::try_from(id)
                    .map_err(|e| PyValueError::new_err(e.to_string()))?;
                Ok(dither_with_canonical(&img, &palette, scheme.palette(), config))
            } else {
                Ok(dither(&img, palette, config))
            }
        }
        (None, Some(id)) => {
            let scheme = ColorScheme::try_from(id)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(dither(&img, scheme.palette(), config))
        }
        (None, None) => Err(PyValueError::new_err("must provide either scheme_id or palette_bytes")),
    }
}

/// Returns all measured palettes from the Rust catalog.
///
/// Each entry is `(id, rgb_bytes, color_names, accent_idx, scheme_id)`.
#[pyfunction]
fn measured_palettes() -> Vec<(String, Vec<u8>, Vec<String>, usize, u8)> {
    CATALOG
        .iter()
        .map(|e| {
            (
                e.id.to_string(),
                e.palette.colors.iter().flatten().copied().collect(),
                e.color_names.iter().map(|&s| s.to_string()).collect(),
                e.palette.accent_idx,
                u8::from(e.scheme),
            )
        })
        .collect()
}

#[pymodule]
fn _rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(dither_image, m)?)?;
    m.add_function(wrap_pyfunction!(measured_palettes, m)?)?;
    Ok(())
}
