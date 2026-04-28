pub mod algorithms;
pub mod color_space;
pub mod color_space_lab;
pub mod enums;
pub mod error;
pub mod measured_palettes;
pub mod palettes;
pub mod tone_map;
pub mod types;

use crate::color_space::{linear_channel_to_srgb, srgb_channel_to_linear};
use crate::enums::{DitherMode, GamutCompression, ToneCompression};
use crate::palettes::Palette;
use crate::types::ImageBuffer;

/// Configuration for `dither()`. All fields have sensible defaults.
///
/// Construct with struct-update syntax:
/// ```ignore
/// dither(&img, palette, DitherConfig { mode: DitherMode::Burkes, ..Default::default() });
/// ```
///
/// Pre-processing pipeline (applied in order, each step is a no-op at its identity value):
/// `exposure → saturation → shadows/highlights → tone → gamut → dither`.
#[derive(Debug, Clone, PartialEq)]
pub struct DitherConfig {
    /// Dithering algorithm.
    pub mode: DitherMode,
    /// Use serpentine scanning for error diffusion (alternates row direction). Ignored for
    /// `None` and `Ordered`.
    pub serpentine: bool,
    /// Linear-RGB exposure multiplier. 1.0 = no change, 2.0 = +1 stop, 0.5 = -1 stop.
    pub exposure: f64,
    /// OKLab saturation multiplier. 1.0 = no change, 0.0 = grayscale, >1.0 = boost.
    /// Hue-preserving.
    pub saturation: f64,
    /// Shadow lift strength (lower-half S-curve). 0.0 = identity, 1.0 = strong lift.
    pub shadows: f64,
    /// Highlight compression strength (upper-half S-curve). 0.0 = identity, 1.0 = strong.
    pub highlights: f64,
    /// Dynamic-range compression. Auto = histogram-based; Fixed(s) = manual blend strength.
    pub tone: ToneCompression,
    /// Gamut compression. Auto = full strength on out-of-gamut; Fixed(s) = manual.
    pub gamut: GamutCompression,
}

impl Default for DitherConfig {
    fn default() -> Self {
        Self {
            mode:       DitherMode::Burkes,
            serpentine: true,
            exposure:   1.0,
            saturation: 1.0,
            shadows:    0.0,
            highlights: 0.0,
            tone:       ToneCompression::Auto,
            gamut:      GamutCompression::Auto,
        }
    }
}

fn dispatch(img: &ImageBuffer, p: &Palette, mode: DitherMode, serpentine: bool) -> Vec<u8> {
    match mode {
        DitherMode::None    => algorithms::direct_map(img.data, p),
        DitherMode::Ordered => algorithms::ordered_dither(img.data, img.width, p),
        DitherMode::FloydSteinberg    => algorithms::error_diffusion_dither(img.data, img.width, img.height, p, &algorithms::FLOYD_STEINBERG,      serpentine),
        DitherMode::Burkes            => algorithms::error_diffusion_dither(img.data, img.width, img.height, p, &algorithms::BURKES,              serpentine),
        DitherMode::Atkinson          => algorithms::error_diffusion_dither(img.data, img.width, img.height, p, &algorithms::ATKINSON,            serpentine),
        DitherMode::Stucki            => algorithms::error_diffusion_dither(img.data, img.width, img.height, p, &algorithms::STUCKI,              serpentine),
        DitherMode::Sierra            => algorithms::error_diffusion_dither(img.data, img.width, img.height, p, &algorithms::SIERRA,              serpentine),
        DitherMode::SierraLite        => algorithms::error_diffusion_dither(img.data, img.width, img.height, p, &algorithms::SIERRA_LITE,         serpentine),
        DitherMode::JarvisJudiceNinke => algorithms::error_diffusion_dither(img.data, img.width, img.height, p, &algorithms::JARVIS_JUDICE_NINKE, serpentine),
    }
}

fn needs_preprocess(c: &DitherConfig) -> bool {
    (c.exposure - 1.0).abs() > 1e-9
        || (c.saturation - 1.0).abs() > 1e-9
        || c.shadows > 0.0
        || c.highlights > 0.0
        || !matches!(c.tone, ToneCompression::Fixed(s) if s <= 0.0)
        || !matches!(c.gamut, GamutCompression::None)
}

/// Dither an image for e-paper display.
///
/// Returns palette indices (one `u8` per pixel, length = `width × height`).
pub fn dither(img: &ImageBuffer, palette: impl AsRef<Palette>, config: DitherConfig) -> Vec<u8> {
    let p = palette.as_ref();

    if !needs_preprocess(&config) {
        return dispatch(img, p, config.mode, config.serpentine);
    }

    // Convert sRGB bytes → linear, apply pre-processing pipeline, convert back.
    let mut linear: Vec<[f64; 3]> = img
        .data
        .chunks_exact(3)
        .map(|c| [
            srgb_channel_to_linear(c[0]),
            srgb_channel_to_linear(c[1]),
            srgb_channel_to_linear(c[2]),
        ])
        .collect();

    tone_map::apply_exposure(&mut linear, config.exposure);
    tone_map::adjust_saturation(&mut linear, config.saturation);
    tone_map::apply_shadows_highlights(&mut linear, config.shadows, config.highlights);
    config.tone.apply(&mut linear, p);
    config.gamut.apply(&mut linear, p);

    let processed: Vec<u8> = linear
        .iter()
        .flat_map(|&[r, g, b]| [
            linear_channel_to_srgb(r),
            linear_channel_to_srgb(g),
            linear_channel_to_srgb(b),
        ])
        .collect();

    let processed_img = ImageBuffer::new(&processed, img.width);
    dispatch(&processed_img, p, config.mode, config.serpentine)
}
