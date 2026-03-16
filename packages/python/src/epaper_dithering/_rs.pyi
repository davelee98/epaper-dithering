def dither_image(
    pixels: bytes,
    width: int,
    height: int,
    scheme_id: int,
    mode_id: int = ...,
    serpentine: bool = ...,
    tone_compression: float | None = ...,
    gamut_compression: float | None = ...,
) -> bytes: ...
def dither_image_palette(
    pixels: bytes,
    width: int,
    height: int,
    palette_bytes: bytes,
    accent_idx: int = ...,
    mode_id: int = ...,
    serpentine: bool = ...,
    tone_compression: float | None = ...,
    gamut_compression: float | None = ...,
) -> bytes: ...
