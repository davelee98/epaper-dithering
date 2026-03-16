use epaper_dithering_core::enums::{DitherMode, GamutCompression, ToneCompression};
use epaper_dithering_core::palettes::ColorScheme;
use epaper_dithering_core::types::ImageBuffer;
use epaper_dithering_core::dither;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

fn parse_scheme(v: u8) -> PyResult<ColorScheme> {
    ColorScheme::try_from(v).map_err(|e| PyValueError::new_err(e.to_string()))
}

fn parse_mode(v: u8) -> PyResult<DitherMode> {
    match v {
        0 => Ok(DitherMode::None),
        1 => Ok(DitherMode::Ordered),
        2 => Ok(DitherMode::FloydSteinberg),
        3 => Ok(DitherMode::Burkes),
        4 => Ok(DitherMode::Atkinson),
        5 => Ok(DitherMode::Stucki),
        6 => Ok(DitherMode::Sierra),
        7 => Ok(DitherMode::SierraLite),
        8 => Ok(DitherMode::JarvisJudiceNinke),
        _ => Err(PyValueError::new_err(format!("unknown dither mode: {v}"))),
    }
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
/// Args:
///     pixels:           flat RGB bytes, row-major (len = width * height * 3)
///     width:            image width
///     height:           image height
///     scheme_id:        firmware color scheme int (0=mono … 7=grayscale16)
///     mode_id:          dither mode int (0=none … 8=jjn)
///     serpentine:       alternate row direction (error diffusion only)
///     tone_compression: None = auto, 0.0 = off, 0.0–1.0 = fixed strength
///     gamut_compression: None = off, 0.0–1.0 = fixed strength
///
/// Returns:
///     bytes of palette indices, one per pixel
#[pyfunction]
#[pyo3(signature = (pixels, width, height, scheme_id, mode_id=3, serpentine=true, tone_compression=None, gamut_compression=None))]
fn dither_image(
    pixels: &[u8],
    width: usize,
    height: usize,
    scheme_id: u8,
    mode_id: u8,
    serpentine: bool,
    tone_compression: Option<f64>,
    gamut_compression: Option<f64>,
) -> PyResult<Vec<u8>> {
    let palette = parse_scheme(scheme_id)?.palette();
    let img = ImageBuffer::new(pixels, width);
    let _ = height; // ImageBuffer derives height from buffer length
    Ok(dither(
        &img,
        palette,
        parse_mode(mode_id)?,
        serpentine,
        parse_tone(tone_compression),
        parse_gamut(gamut_compression),
    ))
}

#[pymodule]
fn epaper_dithering_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(dither_image, m)?)?;
    Ok(())
}
