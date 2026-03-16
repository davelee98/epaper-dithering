use crate::palettes::Palette;

/// A flat RGB image buffer. Height is derived from data length and width.
pub struct ImageBuffer<'a> {
    pub data: &'a [u8],
    pub width: usize,
    pub height: usize,
}

impl<'a> ImageBuffer<'a> {
    /// Create from flat RGB bytes (len = width × height × 3).
    pub fn new(data: &'a [u8], width: usize) -> Self {
        let height = data.len() / 3 / width;
        debug_assert_eq!(data.len(), width * height * 3, "pixel buffer size mismatch");
        Self { data, width, height }
    }
}

/// Anything that can provide a palette reference.
/// Implemented by both `ColorScheme` and `Palette` so `dither()` accepts either.
pub trait AsPalette {
    fn as_palette(&self) -> &Palette;
}

impl AsPalette for Palette {
    fn as_palette(&self) -> &Palette {
        self
    }
}

impl AsPalette for &Palette {
    fn as_palette(&self) -> &Palette {
        self
    }
}
