def dither_image(
    pixels: bytes,
    width: int,
    height: int,
    *,
    scheme_id: int | None = ...,
    palette_bytes: bytes | None = ...,
    accent_idx: int = ...,
    mode_id: int = ...,
    serpentine: bool = ...,
    exposure: float = ...,
    saturation: float = ...,
    shadows: float = ...,
    highlights: float = ...,
    tone: float | None = ...,
    gamut: float | None = ...,
) -> bytes: ...
def measured_palettes() -> list[tuple[str, list[int], list[str], int]]: ...
