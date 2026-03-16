pub mod algorithms;
pub mod color_space;
pub mod color_space_lab;
pub mod enums;
pub mod error;
pub mod measured_palettes;
pub mod palettes;
pub mod types;

use crate::enums::DitherMode;
use crate::types::{AsPalette, ImageBuffer};

// ColorScheme needs both palettes and types modules, so the impl lives here.
impl AsPalette for palettes::ColorScheme {
    fn as_palette(&self) -> &palettes::Palette {
        self.palette()
    }
}

/// Dither an image for e-paper display.
pub fn dither(
    img: &ImageBuffer,
    palette: impl AsPalette,
    mode: DitherMode,
    serpentine: bool,
) -> Vec<u8> {
    let p = palette.as_palette();
    match mode.kernel() {
        None => algorithms::ordered_dither(img.data, img.width, p),
        Some(k) => algorithms::error_diffusion_dither(img.data, img.width, img.height, p, k, serpentine),
    }
}
