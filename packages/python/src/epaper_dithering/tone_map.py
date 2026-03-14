"""Dynamic range compression for measured e-paper palettes.

Maps image luminance from full [0, 1] to the display's actual [black_Y, white_Y]
range before dithering. This prevents large quantization errors at highlights and
shadows, producing smoother dithered output.

Based on fast_compress_dynamic_range() from esp32-photoframe by aitjcize.
"""

from __future__ import annotations

import numpy as np

# ITU-R BT.709 luminance coefficients (same as sRGB)
_LUM_R = 0.2126729
_LUM_G = 0.7151522
_LUM_B = 0.0721750


def compress_dynamic_range(
    pixels_linear: np.ndarray,
    palette_linear: np.ndarray,
    strength: float = 1.0,
) -> np.ndarray:
    """Compress image dynamic range to match display capabilities.

    Remaps pixel luminance from [0, 1] to the display's actual [black_Y, white_Y]
    range, preserving hue. RGB channels are scaled proportionally by the luminance
    ratio so colors stay correct.

    Args:
        pixels_linear: Image in linear RGB, shape (H, W, 3), values in [0, 1].
        palette_linear: Palette in linear RGB, shape (N, 3). Row 0 = black, row 1 = white.
        strength: Blend factor. 0.0 = no compression, 1.0 = full compression.

    Returns:
        Modified pixels_linear array with compressed dynamic range.
    """
    if strength <= 0.0:
        return pixels_linear

    # Display black/white luminance from measured palette
    black_Y = _LUM_R * palette_linear[0, 0] + _LUM_G * palette_linear[0, 1] + _LUM_B * palette_linear[0, 2]
    white_Y = _LUM_R * palette_linear[1, 0] + _LUM_G * palette_linear[1, 1] + _LUM_B * palette_linear[1, 2]
    display_range = white_Y - black_Y

    if display_range <= 0:
        return pixels_linear

    # Per-pixel luminance
    Y = _LUM_R * pixels_linear[:, :, 0] + _LUM_G * pixels_linear[:, :, 1] + _LUM_B * pixels_linear[:, :, 2]

    # Compressed luminance mapped to display range
    compressed_Y = black_Y + Y * display_range

    # Blend between original and compressed based on strength
    if strength < 1.0:
        target_Y = Y + strength * (compressed_Y - Y)
    else:
        target_Y = compressed_Y

    # Scale RGB proportionally to preserve hue
    # For near-black pixels (Y < 1e-6), set to display black level
    safe_Y = np.where(Y > 1e-6, Y, 1.0)
    scale = np.where(Y > 1e-6, target_Y / safe_Y, 0.0)

    result = pixels_linear.copy()
    result[:, :, 0] *= scale
    result[:, :, 1] *= scale
    result[:, :, 2] *= scale

    # Near-black pixels: set to display black luminance
    near_black = Y <= 1e-6
    if np.any(near_black):
        black_level = black_Y * strength
        result[near_black, 0] = black_level
        result[near_black, 1] = black_level
        result[near_black, 2] = black_level

    clipped: np.ndarray = np.clip(result, 0.0, 1.0)
    return clipped


def auto_compress_dynamic_range(
    pixels_linear: np.ndarray,
    palette_linear: np.ndarray,
) -> np.ndarray:
    """Conditionally compress dynamic range to display capabilities.

    Analyzes the image's actual luminance distribution (2nd/98th percentiles)
    and only applies compression when the image content genuinely exceeds the
    display's reproducible range. Images that already fit within the display's
    [black_Y, white_Y] range are returned unchanged (ICC Black Point
    Compensation style).

    This avoids the over-compression that occurs when unconditionally stretching
    a well-exposed image to fill the display range — which washes out colors
    by pushing already-correct highlights above the display white point.

    Args:
        pixels_linear: Image in linear RGB, shape (H, W, 3), values in [0, 1].
        palette_linear: Palette in linear RGB, shape (N, 3). Row 0 = black, row 1 = white.

    Returns:
        Modified pixels_linear array with compressed dynamic range.
    """
    # Display black/white luminance from measured palette
    black_Y = _LUM_R * palette_linear[0, 0] + _LUM_G * palette_linear[0, 1] + _LUM_B * palette_linear[0, 2]
    white_Y = _LUM_R * palette_linear[1, 0] + _LUM_G * palette_linear[1, 1] + _LUM_B * palette_linear[1, 2]
    display_range = white_Y - black_Y

    if display_range <= 0:
        return pixels_linear

    # Per-pixel luminance
    Y = _LUM_R * pixels_linear[:, :, 0] + _LUM_G * pixels_linear[:, :, 1] + _LUM_B * pixels_linear[:, :, 2]

    # Image luminance percentiles (ignore 2% outliers at each end)
    p_low = float(np.percentile(Y, 2))
    p_high = float(np.percentile(Y, 98))
    image_range = p_high - p_low

    if image_range < 1e-6:
        # Uniform image: fall back to standard linear compression
        return compress_dynamic_range(pixels_linear, palette_linear, 1.0)

    # Only compress if the image content genuinely exceeds the display range.
    # Allow 10% of display_range as tolerance to avoid compressing images that
    # merely approach the display limits without meaningfully clipping.
    TOLERANCE = 0.10
    fits_shadows = p_low >= black_Y - TOLERANCE * display_range
    fits_highlights = p_high <= white_Y + TOLERANCE * display_range

    if fits_shadows and fits_highlights:
        # Image already fits within the display's reproducible range — no change.
        return pixels_linear

    # Remap: [p_low, p_high] → [black_Y, white_Y]
    normalized_Y = (Y - p_low) / image_range
    target_Y = black_Y + normalized_Y * display_range

    # Scale RGB proportionally to preserve hue
    safe_Y = np.where(Y > 1e-6, Y, 1.0)
    scale = np.where(Y > 1e-6, target_Y / safe_Y, 0.0)

    result = pixels_linear.copy()
    result[:, :, 0] *= scale
    result[:, :, 1] *= scale
    result[:, :, 2] *= scale

    # Near-black pixels: set to display black luminance
    near_black = Y <= 1e-6
    if np.any(near_black):
        result[near_black, 0] = black_Y
        result[near_black, 1] = black_Y
        result[near_black, 2] = black_Y

    clipped: np.ndarray = np.clip(result, 0.0, 1.0)
    return clipped
