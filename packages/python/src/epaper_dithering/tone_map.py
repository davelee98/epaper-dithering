"""Dynamic range compression for measured e-paper palettes.

Maps image luminance from full [0, 1] to the display's actual [black_Y, white_Y]
range before dithering. This prevents large quantization errors at highlights and
shadows, producing smoother dithered output.

Based on fast_compress_dynamic_range() from esp32-photoframe by aitjcize.
"""

from __future__ import annotations

import numpy as np

from .color_space_lab import rgb_to_lab

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

    When compression is needed, strength is derived from the Reinhard 2004
    log-histogram skewness — the position of the geometric mean luminance
    within the log luminance range. A balanced image (log-average near center)
    gets partial compression, preserving perceived contrast. A heavily skewed
    image (dark scene with bright highlights) gets full compression.

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

    # Derive compression strength from Reinhard 2004 log-histogram skewness.
    #
    # Skewness = where the geometric mean (log-average) luminance sits within
    # the log luminance range [log(p_low), log(p_high)]:
    #   0 = log-average at the bright end → balanced/bright image → less compression
    #   1 = log-average at the dark end  → dark scene, bright highlights → full compression
    #
    # strength = clip(skew ^ 1.4, 0, 1)  — the 1.4 exponent from Reinhard 2004
    # makes the response non-linear, more sensitive at extremes.
    Y_nonzero = Y.ravel()
    Y_nonzero = Y_nonzero[Y_nonzero > 1e-6]
    if len(Y_nonzero) > 0:
        L_lav = float(np.exp(np.mean(np.log(Y_nonzero + 1e-5))))
        log_min = float(np.log(max(p_low, 1e-5)))
        log_max = float(np.log(max(p_high, 1e-5)))
        log_range = log_max - log_min
        if log_range > 1e-6:
            skew = (log_max - float(np.log(L_lav + 1e-5))) / log_range
            strength = float(np.clip(skew**1.4, 0.0, 1.0))
        else:
            strength = 1.0
    else:
        strength = 1.0

    # Remap: [p_low, p_high] → [black_Y, white_Y], blended at computed strength.
    # At strength=0: original luminance preserved.
    # At strength=1: full content-adaptive remap (original behaviour).
    normalized_Y = (Y - p_low) / image_range
    target_Y_full = black_Y + normalized_Y * display_range
    target_Y = Y + strength * (target_Y_full - Y)

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


def gamut_compress(
    pixels_linear: np.ndarray,
    palette_linear: np.ndarray,
    strength: float = 1.0,
) -> np.ndarray:
    """Blend out-of-gamut pixels toward their nearest palette color.

    Pre-dithering step that reduces colors lying far outside the display's
    reproducible gamut. Colors near any palette color are left unchanged;
    colors far outside are blended toward the nearest palette color.

    Useful for images with highly saturated colors the palette cannot reproduce
    (e.g. vivid purple on a BWGBRY display where the measured red is very dark).
    Without compression, error diffusion produces a muddy mix of red/blue dots;
    with compression the result is a controlled, intentional blend.

    Distance is measured in OKLab with the same LCH weighting used for palette
    matching, so compression is applied where it matters perceptually. Based on
    the gamut mapping approach of Stone, Cowan & Beatty (ACM TOG 1988).

    Args:
        pixels_linear: Image in linear RGB, shape (H, W, 3), values in [0, 1].
        palette_linear: Palette in linear RGB, shape (N, 3).
        strength: 0.0 = no compression, 1.0 = full blend to nearest palette
            color for pixels far outside the gamut. Values of 0.7–0.9 are
            recommended for typical use.

    Returns:
        Modified pixels_linear array with out-of-gamut colors compressed.
    """
    if strength <= 0.0:
        return pixels_linear

    lab_pixels = rgb_to_lab(pixels_linear)  # (H, W, 3)
    lab_palette = rgb_to_lab(palette_linear)  # (N, 3)

    # Euclidean OKLab distance to every palette color: (H, W, N)
    # NOTE: LCH-weighted distance is NOT used here. The LCH hue weight causes
    # near-achromatic palette colors (black, white) to appear "nearest" to any
    # saturated pixel because their low chroma produces near-zero hue mismatch.
    # Plain Euclidean OKLab finds the genuinely closest color by all three
    # dimensions, so purple correctly maps to blue/red, not to black.
    diff = lab_pixels[..., np.newaxis, :] - lab_palette[np.newaxis, :, :]  # (H, W, N, 3)
    dist_sq = np.sum(diff**2, axis=-1)  # (H, W, N)

    # Nearest palette color distance per pixel
    nearest_idx = np.argmin(dist_sq, axis=-1)  # (H, W)
    nearest_dist = np.sqrt(np.take_along_axis(dist_sq, nearest_idx[..., np.newaxis], axis=-1).squeeze(-1))  # (H, W)

    # Smoothstep blend factor: 0 inside gamut, ramps to 1 far outside
    # Threshold chosen in OKLab LCH-weighted space: 0.05 ≈ just-noticeable difference
    _THRESHOLD = 0.05
    _THRESHOLD_MAX = 0.20
    t = np.clip((nearest_dist - _THRESHOLD) / (_THRESHOLD_MAX - _THRESHOLD), 0.0, 1.0)
    blend_factor = t * t * (3.0 - 2.0 * t) * strength  # smoothstep × strength

    # Blend toward nearest palette color in linear RGB
    nearest_rgb = palette_linear[nearest_idx]  # (H, W, 3)
    result = pixels_linear + blend_factor[..., np.newaxis] * (nearest_rgb - pixels_linear)

    clipped: np.ndarray = np.clip(result, 0.0, 1.0)
    return clipped


def auto_gamut_compress(
    pixels_linear: np.ndarray,
    palette_linear: np.ndarray,
) -> np.ndarray:
    """Conditionally apply gamut compression based on image content.

    Analyzes the image's 95th-percentile nearest-palette-color distance and
    only compresses when a meaningful fraction of pixels genuinely lie outside
    the display's reproducible gamut. Images whose colors already fall within
    the palette's gamut are returned unchanged.

    Args:
        pixels_linear: Image in linear RGB, shape (H, W, 3), values in [0, 1].
        palette_linear: Palette in linear RGB, shape (N, 3).

    Returns:
        Modified pixels_linear array, or the original if already in gamut.
    """
    lab_pixels = rgb_to_lab(pixels_linear)
    lab_palette = rgb_to_lab(palette_linear)

    diff = lab_pixels[..., np.newaxis, :] - lab_palette[np.newaxis, :, :]
    dist_sq = np.sum(diff**2, axis=-1)
    nearest_dist = np.sqrt(np.min(dist_sq, axis=-1))  # (H, W)

    p50 = float(np.percentile(nearest_dist, 50))
    p95 = float(np.percentile(nearest_dist, 95))

    # Only compress if a significant portion of the image is out of gamut.
    # 0.25 sits between natural photos (p95 ≈ 0.20) and synthetic/vivid images
    # (p95 ≈ 0.30+). Raised above _THRESHOLD_MAX (0.20) to avoid false triggers.
    if p95 <= 0.25:
        return pixels_linear

    # Derive strength from two independent signals (calibrated OKLab distances):
    #   p95 / 0.35: images with extreme out-of-gamut outliers → full strength
    #   p50 / 0.12: images where the median pixel is moderately out-of-gamut
    #               (widespread issue, not just outliers) → scale up
    # Take the maximum so either signal can drive full compression independently.
    s_p95 = float(np.clip(p95 / 0.35, 0.0, 1.0))
    s_p50 = float(np.clip(p50 / 0.12, 0.0, 1.0))
    strength = max(s_p95, s_p50)

    return gamut_compress(pixels_linear, palette_linear, strength=strength)
