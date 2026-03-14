"""OKLab color space conversions and LCH-weighted color matching for dithering.

OKLab (Ottosson 2020) is a perceptually uniform color space with better hue
linearity than CIELAB. Equal distances in OKLab represent more consistent
perceived color differences across all hue angles, including the yellow/purple
regions where CIELAB is known to warp.

Why LCH Weighting for Dithering:
---------------------------------
Standard perceptual distance (Delta E) weights lightness, chroma, and hue
equally. But for error-diffusion dithering, hue preservation matters MORE
than lightness accuracy because:
- Error diffusion compensates for lightness by mixing dark+light pixels spatially
- Error diffusion CANNOT compensate for hue errors (no way to mix green from
  non-green palette colors)

The LCH decomposition uses the identity: da^2 + db^2 = dC^2 + dH^2,
allowing us to weight the three perceptual dimensions independently:
- Lightness (WL=0.5): de-emphasized, error diffusion handles this
- Chroma (WC=1.0): standard weight
- Hue (WH=2.0): emphasized, prevents cross-hue errors like green->yellow

References:
----------
- Ottosson, B. — "A perceptual color space for image processing", 2020
  https://bottosson.github.io/posts/oklab/
- http://www.brucelindbloom.com/index.html?Eqn_RGB_XYZ_Matrix.html
"""

from __future__ import annotations

import math

import numpy as np

# =============================================================================
# Constants
# =============================================================================

# sRGB to XYZ matrix (D65 illuminant, sRGB primaries)
# From http://www.brucelindbloom.com/index.html?Eqn_RGB_XYZ_Matrix.html
_M_RGB_TO_XYZ = np.array(
    [
        [0.4124564, 0.3575761, 0.1804375],
        [0.2126729, 0.7151522, 0.0721750],
        [0.0193339, 0.1191920, 0.9503041],
    ],
    dtype=np.float64,
)

# OKLab matrices (Ottosson 2020)
# M1: XYZ → LMS (Hunt-Pointer-Estevez with Bradford adaptation)
_M1 = np.array(
    [
        [0.8189330101, 0.3618667424, -0.1288597137],
        [0.0329845436, 0.9293118715, 0.0361456387],
        [0.0482003018, 0.2643662691, 0.6338517070],
    ],
    dtype=np.float64,
)

# M2: cbrt(LMS) → OKLab
_M2 = np.array(
    [
        [0.2104542553, 0.7936177850, -0.0040720468],
        [1.9779984951, -2.4285922050, 0.4505937099],
        [0.0259040371, 0.7827717662, -0.8086757660],
    ],
    dtype=np.float64,
)

# LCH distance weights for dithering (tuned for OKLab scale)
#
# OKLab: L ∈ [0, 1], C ∈ [0, ~0.4] — very different scale from CIELAB
# (CIELAB: L ∈ [0, 100], C ∈ [0, ~130]).
#
# To maintain the same relative emphasis as the original CIELAB weights
# (WL=0.5, WC=1.0, WH=2.0), the chroma/hue weights must be scaled up:
#   CIELAB: effective L range = 100×0.5 = 50, C range = 130×1.0 = 130 → C is 2.6× L
#   OKLab:  effective L range =   1×0.5 = 0.5, target C = 2.6×0.5 = 1.3 → WC = 1.3/0.4 ≈ 3.0
#
# Without this scaling, chroma penalties become negligible in OKLab units,
# causing achromatic pixels to map to intermediate-L chromatic palette colors
# (e.g. green at L=0.43) instead of neutral black/white.
_WL = 0.5  # lightness: de-emphasized (error diffusion compensates)
_WC = 3.0  # chroma: scaled up for OKLab's smaller C range [0, ~0.4]
_WH = 6.0  # hue: emphasized (error diffusion cannot compensate)


# =============================================================================
# Vectorized Functions (for batch operations: direct mapping, ordered dither)
# =============================================================================


def rgb_to_lab(rgb: np.ndarray) -> np.ndarray:
    """Convert linear RGB to OKLab color space.

    Args:
        rgb: Linear RGB values in [0, 1] range. Shape: (..., 3)

    Returns:
        OKLab values. L in [0, 1], a and b in [-0.5, 0.5]. Shape: (..., 3)
    """
    xyz = rgb @ _M_RGB_TO_XYZ.T
    lms = xyz @ _M1.T
    lms_ = np.cbrt(lms)
    return lms_ @ _M2.T


def find_closest_palette_color_lab(
    rgb_linear: np.ndarray,
    palette_linear: np.ndarray,
) -> np.ndarray:
    """Find closest palette color using LCH-weighted OKLab distance.

    Optimized for batch operations (entire image at once). Uses numpy
    broadcasting to compute distances for all pixels simultaneously.

    Args:
        rgb_linear: Linear RGB values. Shape:
            - (3,) for single pixel
            - (height, width, 3) for entire image
        palette_linear: Palette colors in linear space. Shape: (num_colors, 3)

    Returns:
        Palette indices. Shape matches input without last dimension.
    """
    lab_pixels = rgb_to_lab(rgb_linear)
    lab_palette = rgb_to_lab(palette_linear)

    # Chroma of each palette color
    C_palette = np.sqrt(lab_palette[:, 1] ** 2 + lab_palette[:, 2] ** 2)

    # Chroma of each pixel
    C_pixels = np.sqrt(lab_pixels[..., 1] ** 2 + lab_pixels[..., 2] ** 2)

    # Broadcast differences: (..., 1, 3) - (num_colors, 3) -> (..., num_colors, 3)
    diff = lab_pixels[..., np.newaxis, :] - lab_palette[np.newaxis, :, :]
    dL = diff[..., 0]
    da = diff[..., 1]
    db = diff[..., 2]

    # LCH decomposition: da^2 + db^2 = dC^2 + dH^2
    dC = C_pixels[..., np.newaxis] - C_palette[np.newaxis, :]
    dH_sq = np.maximum(0.0, da**2 + db**2 - dC**2)

    # Weighted distance
    distances = (_WL * dL) ** 2 + (_WC * dC) ** 2 + _WH**2 * dH_sq

    return np.argmin(distances, axis=-1)  # type: ignore[no-any-return]


# =============================================================================
# Scalar Functions (for per-pixel error diffusion — no numpy overhead)
# =============================================================================


def _rgb_to_lab_scalar(r: float, g: float, b: float) -> tuple[float, float, float]:
    """Convert a single linear RGB pixel to OKLab (scalar, no numpy)."""
    # RGB -> XYZ (inline matrix multiply)
    x = 0.4124564 * r + 0.3575761 * g + 0.1804375 * b
    y = 0.2126729 * r + 0.7151522 * g + 0.0721750 * b
    z = 0.0193339 * r + 0.1191920 * g + 0.9503041 * b

    # XYZ -> LMS (M1)
    l = 0.8189330101 * x + 0.3618667424 * y + (-0.1288597137) * z  # noqa: E741
    m = 0.0329845436 * x + 0.9293118715 * y + 0.0361456387 * z
    s = 0.0482003018 * x + 0.2643662691 * y + 0.6338517070 * z

    # Cube root
    l_ = math.cbrt(l)
    m_ = math.cbrt(m)
    s_ = math.cbrt(s)

    # cbrt(LMS) -> OKLab (M2)
    L = 0.2104542553 * l_ + 0.7936177850 * m_ + (-0.0040720468) * s_
    a = 1.9779984951 * l_ + (-2.4285922050) * m_ + 0.4505937099 * s_
    b_val = 0.0259040371 * l_ + 0.7827717662 * m_ + (-0.8086757660) * s_
    return L, a, b_val


def _match_pixel_lch(
    r: float,
    g: float,
    b: float,
    palette_L: tuple[float, ...],
    palette_a: tuple[float, ...],
    palette_b: tuple[float, ...],
    palette_C: tuple[float, ...],
) -> int:
    """Find closest palette color for a single pixel using LCH distance.

    Pure Python (no numpy) for minimal per-call overhead in error diffusion.

    Args:
        r, g, b: Pixel in linear RGB [0, 1]
        palette_L, palette_a, palette_b: Pre-computed OKLab components of palette
        palette_C: Pre-computed chroma of palette colors

    Returns:
        Index of closest palette color
    """
    pL, pa, pb = _rgb_to_lab_scalar(r, g, b)
    pC = math.sqrt(pa * pa + pb * pb)

    best_idx = 0
    best_dist = float("inf")

    for i in range(len(palette_L)):
        dL = pL - palette_L[i]
        da = pa - palette_a[i]
        db = pb - palette_b[i]
        dC = pC - palette_C[i]
        dH_sq = da * da + db * db - dC * dC
        if dH_sq < 0.0:
            dH_sq = 0.0

        dist = (_WL * dL) ** 2 + (_WC * dC) ** 2 + _WH * _WH * dH_sq
        if dist < best_dist:
            best_dist = dist
            best_idx = i

    return best_idx


def precompute_palette_lab(
    palette_linear: np.ndarray,
) -> tuple[tuple[float, ...], tuple[float, ...], tuple[float, ...], tuple[float, ...]]:
    """Pre-compute palette OKLab components for scalar matching.

    Call once before the error diffusion loop, then pass results to
    _match_pixel_lch() for each pixel.

    Args:
        palette_linear: Palette in linear RGB. Shape: (num_colors, 3)

    Returns:
        (palette_L, palette_a, palette_b, palette_C) as tuples of floats
    """
    lab = rgb_to_lab(palette_linear)
    L = tuple(float(x) for x in lab[:, 0])
    a = tuple(float(x) for x in lab[:, 1])
    b = tuple(float(x) for x in lab[:, 2])
    C = tuple(math.sqrt(float(ai) ** 2 + float(bi) ** 2) for ai, bi in zip(lab[:, 1], lab[:, 2]))
    return L, a, b, C
