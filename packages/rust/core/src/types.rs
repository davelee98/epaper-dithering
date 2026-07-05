/// A flat RGB image buffer. Height is derived from data length and width.
pub struct ImageBuffer<'a> {
    pub data: &'a [u8],
    pub width: usize,
    pub height: usize,
}

impl<'a> ImageBuffer<'a> {
    /// Create from flat RGB bytes (len = width × height × 3).
    ///
    /// # Panics
    /// Panics if `width == 0` or `data.len()` is not exactly `width × height × 3`. FFI
    /// boundaries validate and return errors before reaching this contract backstop.
    pub fn new(data: &'a [u8], width: usize) -> Self {
        assert!(width > 0, "image width must be non-zero");
        let height = data.len() / 3 / width;
        assert_eq!(data.len(), width * height * 3, "pixel buffer size mismatch (len={}, width={width})", data.len());
        Self { data, width, height }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_accepts_well_formed_buffer() {
        let data = [0u8; 2 * 3 * 3]; // 3×2 RGB
        let img = ImageBuffer::new(&data, 3);
        assert_eq!(img.height, 2);
    }

    #[test]
    #[should_panic(expected = "width must be non-zero")]
    fn new_rejects_zero_width() {
        ImageBuffer::new(&[0u8; 3], 0);
    }

    #[test]
    #[should_panic(expected = "pixel buffer size mismatch")]
    fn new_rejects_ragged_buffer() {
        // 7 bytes is neither a whole number of pixels nor rows for width 2.
        ImageBuffer::new(&[0u8; 7], 2);
    }
}
