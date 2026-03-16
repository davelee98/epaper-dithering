"""Main dithering interface."""

from __future__ import annotations

import logging

from PIL import Image

import epaper_dithering._rs as _rs

from .enums import DitherMode
from .palettes import ColorPalette, ColorScheme

_LOGGER = logging.getLogger(__name__)


def _to_rgb_bytes(image: Image.Image) -> tuple[bytes, int, int]:
    if image.mode == "RGBA":
        bg = Image.new("RGB", image.size, (255, 255, 255))
        bg.paste(image, mask=image.split()[3])
        img_rgb = bg
    else:
        img_rgb = image.convert("RGB")
    width, height = img_rgb.size
    return img_rgb.tobytes(), width, height


def _compression(v: float | str) -> float | None:
    """Map Python compression param to Rust Option<f64>: None=auto, float=fixed."""
    return None if v == "auto" else float(v)


def dither_image(
    image: Image.Image,
    color_scheme: ColorScheme | ColorPalette,
    mode: DitherMode = DitherMode.BURKES,
    serpentine: bool = True,
    tone_compression: float | str = "auto",
    gamut_compression: float | str = "auto",
) -> Image.Image:
    """Apply dithering to image for e-paper display.

    Args:
        image: Input image (RGB or RGBA)
        color_scheme: Target display color scheme OR measured ColorPalette
        mode: Dithering algorithm (default: BURKES)
        serpentine: Use serpentine scanning for error diffusion (default: True).
            Alternates scan direction each row to reduce directional artifacts.
            Only applies to error diffusion algorithms, ignored for NONE and ORDERED.
        tone_compression: Dynamic range compression (default: "auto").
            "auto" = analyze image histogram and fit to display range.
            0.0 = disabled, 0.0-1.0 = fixed linear compression strength.
            Only applies to measured ColorPalette.
        gamut_compression: Pre-dithering gamut compression (default: "auto").
            Blends out-of-gamut pixels toward their nearest palette color before
            dithering. Useful for images with highly saturated colors the palette
            cannot reproduce (e.g. vivid purple on a BWGBRY display).
            "auto" = only compress when image content genuinely exceeds the
            palette gamut; auto mode only activates for measured ColorPalette.
            0.0 = disabled, 0.7-0.9 = fixed strength (works on all palette types).

    Returns:
        Dithered palette image matching color scheme
    """
    if not isinstance(tone_compression, (float, str)):
        raise TypeError(f"tone_compression must be float or 'auto', got {type(tone_compression).__name__}")
    if not isinstance(gamut_compression, (float, str)):
        raise TypeError(f"gamut_compression must be float or 'auto', got {type(gamut_compression).__name__}")

    scheme_name = color_scheme.name if isinstance(color_scheme, ColorScheme) else "custom"
    _LOGGER.debug("Applying %s dithering for %s palette", mode.name, scheme_name)

    pixels, width, height = _to_rgb_bytes(image)

    if isinstance(color_scheme, ColorScheme):
        indices = _rs.dither_image(
            pixels,
            width,
            height,
            color_scheme.value,  # type: ignore[arg-type]  # non-standard enum pattern: _value_ is int
            int(mode),
            serpentine,
            0.0,
            0.0,  # tone/gamut off — idealized palette has no measured display range
        )
        palette_colors = list(color_scheme.palette.colors.values())
    else:
        palette_colors = list(color_scheme.colors.values())
        palette_bytes = bytes(c for rgb in palette_colors for c in rgb)
        accent_idx = list(color_scheme.colors.keys()).index(color_scheme.accent)
        indices = _rs.dither_image_palette(
            pixels,
            width,
            height,
            palette_bytes,
            accent_idx,
            int(mode),
            serpentine,
            _compression(tone_compression),
            _compression(gamut_compression),
        )

    out = Image.new("P", (width, height))
    out.putdata(indices)
    out.putpalette([c for rgb in palette_colors for c in rgb])
    return out
