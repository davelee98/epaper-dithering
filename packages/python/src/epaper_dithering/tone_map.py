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
    """Blend out-of-gamut pixels toward the hue-matching point on the palette hull.

    Pre-dithering step that reduces colors lying far outside the display's
    reproducible gamut. For each pixel, finds the point on the nearest palette
    edge whose hue matches the pixel's hue — so a vivid purple lands between
    blue and red rather than collapsing to the Euclidean nearest vertex. This
    preserves the pixel's hue character and gives error diffusion a better
    starting point for mixing.

    Args:
        pixels_linear: Image in linear RGB, shape (H, W, 3), values in [0, 1].
        palette_linear: Palette in linear RGB, shape (N, 3).
        strength: 0.0 = no compression, 1.0 = full blend to hue-matching
            palette edge point for pixels far outside the gamut.

    Returns:
        Modified pixels_linear array with out-of-gamut colors compressed.
    """
    if strength <= 0.0:
        return pixels_linear

    lab_pixels = rgb_to_lab(pixels_linear)  # (H, W, 3)
    lab_palette = rgb_to_lab(palette_linear)  # (N, 3)
    n_colors = len(palette_linear)

    pixel_a = lab_pixels[..., 1]  # (H, W) — OKLab a (green↔red)
    pixel_b = lab_pixels[..., 2]  # (H, W) — OKLab b (blue↔yellow)

    # For each palette edge, find the point where the interpolated hue matches
    # the pixel's hue. This is hue-preserving gamut mapping: a vivid purple lands
    # between blue and red rather than collapsing to the Euclidean nearest vertex.
    #
    # Hue equality condition: (a0 + t*da)*pixel_b == (b0 + t*db)*pixel_a
    # Solving for t: t = (b0*pixel_a - a0*pixel_b) / (da*pixel_b - db*pixel_a)
    #
    # t is clipped to [0, 1] so edges that don't span the pixel's hue degrade
    # gracefully to their nearest endpoint.
    best_dist_sq = np.full(pixels_linear.shape[:2], np.inf)
    best_target_rgb = np.zeros_like(pixels_linear)

    for i in range(n_colors):
        for j in range(i + 1, n_colors):
            a0 = lab_palette[i, 1]
            b0 = lab_palette[i, 2]
            da = lab_palette[j, 1] - a0
            db = lab_palette[j, 2] - b0
            denom = da * pixel_b - db * pixel_a  # (H, W)
            numer = b0 * pixel_a - a0 * pixel_b  # (H, W)
            valid = np.abs(denom) > 1e-10
            seg_t = np.clip(
                np.where(valid, numer / np.where(valid, denom, 1.0), 0.0),
                0.0,
                1.0,
            )  # (H, W)
            nearest_lab = lab_palette[i] + seg_t[..., np.newaxis] * (lab_palette[j] - lab_palette[i])
            dist_sq = np.sum((lab_pixels - nearest_lab) ** 2, axis=-1)
            target_rgb = palette_linear[i] + seg_t[..., np.newaxis] * (palette_linear[j] - palette_linear[i])
            better = dist_sq < best_dist_sq
            best_dist_sq = np.where(better, dist_sq, best_dist_sq)
            best_target_rgb = np.where(better[..., np.newaxis], target_rgb, best_target_rgb)

    # Fallback: nearest palette vertex — handles achromatic pixels (hue undefined)
    # and any degenerate cases where all segments clip to endpoints.
    diff_v = lab_pixels[..., np.newaxis, :] - lab_palette[np.newaxis, :, :]  # (H, W, N, 3)
    dist_sq_v = np.sum(diff_v**2, axis=-1)  # (H, W, N)
    nearest_v_idx = np.argmin(dist_sq_v, axis=-1)  # (H, W)
    nearest_v_dist_sq = np.take_along_axis(dist_sq_v, nearest_v_idx[..., np.newaxis], axis=-1).squeeze(-1)
    nearest_v_rgb = palette_linear[nearest_v_idx]
    better_v = nearest_v_dist_sq < best_dist_sq
    best_dist_sq = np.where(better_v, nearest_v_dist_sq, best_dist_sq)
    best_target_rgb = np.where(better_v[..., np.newaxis], nearest_v_rgb, best_target_rgb)

    nearest_dist = np.sqrt(best_dist_sq)

    # Smoothstep blend factor: 0 inside gamut, ramps to 1 far outside
    # Threshold chosen in OKLab space: 0.05 ≈ just-noticeable difference
    _THRESHOLD = 0.05
    _THRESHOLD_MAX = 0.20
    blend_t = np.clip((nearest_dist - _THRESHOLD) / (_THRESHOLD_MAX - _THRESHOLD), 0.0, 1.0)
    blend_factor = blend_t * blend_t * (3.0 - 2.0 * blend_t) * strength  # smoothstep × strength

    result = pixels_linear + blend_factor[..., np.newaxis] * (best_target_rgb - pixels_linear)

    clipped: np.ndarray = np.clip(result, 0.0, 1.0)
    return clipped


def _optimize_compression_strength(
    pixels_linear: np.ndarray,
    palette_linear: np.ndarray,
    compress_fn: object,
    candidates: list[float],
) -> float:
    """Select compression strength that minimizes mean OKLab nearest-palette distance.

    Evaluates each candidate strength by applying compression and measuring the
    mean OKLab distance between compressed pixels and their nearest palette color.
    No dithering required — error diffusion redistributes but does not change
    total quantization error, so pre-diffusion distance is a valid proxy.

    Args:
        pixels_linear: Image in linear RGB, shape (H, W, 3).
        palette_linear: Palette in linear RGB, shape (N, 3).
        compress_fn: Callable(pixels, palette, strength) -> compressed_pixels.
        candidates: Strength values to evaluate (must include 0.0).

    Returns:
        Strength from candidates with minimum mean quantization error.
    """
    lab_palette = rgb_to_lab(palette_linear)  # (N, 3)

    best_strength = candidates[0]
    best_error = float("inf")

    for s in candidates:
        if s > 0.0:
            compressed = compress_fn(pixels_linear, palette_linear, s)  # type: ignore[operator]
        else:
            compressed = pixels_linear
        lab_pixels = rgb_to_lab(compressed)
        diff = lab_pixels[..., np.newaxis, :] - lab_palette[np.newaxis, :, :]  # (H, W, N, 3)
        nearest_dist = np.sqrt(np.min(np.sum(diff**2, axis=-1), axis=-1))  # (H, W)
        error = float(np.mean(nearest_dist))

        if error < best_error:
            best_error = error
            best_strength = s

    return best_strength


def auto_gamut_compress(
    pixels_linear: np.ndarray,
    palette_linear: np.ndarray,
) -> np.ndarray:
    """Conditionally apply gamut compression based on image content.

    Selects the compression strength that minimizes mean OKLab nearest-palette
    distance across 5 candidate values. Strength 0.0 is always a candidate, so
    images already within the display's gamut are returned unchanged.

    Args:
        pixels_linear: Image in linear RGB, shape (H, W, 3), values in [0, 1].
        palette_linear: Palette in linear RGB, shape (N, 3).

    Returns:
        Modified pixels_linear array with optimal gamut compression applied.
    """
    _CANDIDATES = [0.0, 0.25, 0.5, 0.75, 1.0]
    strength = _optimize_compression_strength(pixels_linear, palette_linear, gamut_compress, _CANDIDATES)

    if strength <= 0.0:
        return pixels_linear

    return gamut_compress(pixels_linear, palette_linear, strength=strength)
