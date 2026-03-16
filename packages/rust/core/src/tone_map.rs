/// Dynamic range and gamut compression for measured e-paper palettes.
///
/// Applied before dithering to map image content into the display's reproducible range.
/// All functions operate on linear RGB pixels (`&mut [[f64; 3]]`).

use rayon::prelude::*;

use crate::color_space::srgb_channel_to_linear;
use crate::color_space_lab::rgb_to_oklab;
use crate::palettes::Palette;

// ITU-R BT.709 / sRGB luminance coefficients
const LUM_R: f64 = 0.2126729;
const LUM_G: f64 = 0.7151522;
const LUM_B: f64 = 0.0721750;

fn luminance([r, g, b]: [f64; 3]) -> f64 {
    LUM_R * r + LUM_G * g + LUM_B * b
}

fn palette_to_linear(palette: &Palette) -> Vec<[f64; 3]> {
    palette
        .colors
        .iter()
        .map(|&[r, g, b]| {
            [
                srgb_channel_to_linear(r),
                srgb_channel_to_linear(g),
                srgb_channel_to_linear(b),
            ]
        })
        .collect()
}

/// p-th percentile of a slice of f64 values (0–100). Allocates a sorted copy.
fn percentile(values: &[f64], p: f64) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((p / 100.0) * (sorted.len().saturating_sub(1)) as f64).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

// ── compress_dynamic_range ────────────────────────────────────────────────────

/// Remap pixel luminance from [0, 1] to the display's [black_Y, white_Y] range.
///
/// `palette.colors[0]` = black ink, `palette.colors[1]` = white/paper.
/// `strength` blends between no compression (0.0) and full remap (1.0).
pub fn compress_dynamic_range(pixels: &mut [[f64; 3]], palette: &Palette, strength: f64) {
    if strength <= 0.0 {
        return;
    }

    let pal = palette_to_linear(palette);
    let black_y = luminance(pal[0]);
    let white_y = luminance(pal[1]);
    let display_range = white_y - black_y;

    if display_range <= 0.0 {
        return;
    }


    pixels.par_iter_mut().for_each(|pixel| {
        let y = luminance(*pixel);
        let compressed_y = black_y + y * display_range;
        let target_y = y + strength * (compressed_y - y);
        if y > 1e-6 {
            let scale = (target_y / y).clamp(0.0, 1e6);
            pixel[0] = (pixel[0] * scale).clamp(0.0, 1.0);
            pixel[1] = (pixel[1] * scale).clamp(0.0, 1.0);
            pixel[2] = (pixel[2] * scale).clamp(0.0, 1.0);
        } else {
            // For very dark pixels, set to a blend of black_y and original (which is near zero)
            let blended_black = black_y * strength;
            pixel[0] = blended_black.clamp(0.0, 1.0);
            pixel[1] = blended_black.clamp(0.0, 1.0);
            pixel[2] = blended_black.clamp(0.0, 1.0);
        }
    });
}

// ── auto_compress_dynamic_range ───────────────────────────────────────────────

/// Conditionally compress dynamic range using Reinhard 2004 log-skewness for strength.
///
/// Only compresses when the image genuinely exceeds the display range (±10% tolerance).
/// Strength = clip(skew^1.4, 0, 1) where skew = position of log-average in log range.
pub fn auto_compress_dynamic_range(pixels: &mut [[f64; 3]], palette: &Palette) {
    let pal = palette_to_linear(palette);
    let black_y = luminance(pal[0]);
    let white_y = luminance(pal[1]);
    let display_range = white_y - black_y;

    if display_range <= 0.0 {
        return;
    }

    let lum_values: Vec<f64> = pixels.iter().map(|&p| luminance(p)).collect();

    let p_low = percentile(&lum_values, 2.0);
    let p_high = percentile(&lum_values, 98.0);
    let image_range = p_high - p_low;

    if image_range < 1e-6 {
        compress_dynamic_range(pixels, palette, 1.0);
        return;
    }

    const TOLERANCE: f64 = 0.10;
    let fits_shadows = p_low >= black_y - TOLERANCE * display_range;
    let fits_highlights = p_high <= white_y + TOLERANCE * display_range;

    if fits_shadows && fits_highlights {
        return;
    }

    // Reinhard 2004: strength from log-histogram skewness
    let nonzero: Vec<f64> = lum_values.iter().copied().filter(|&y| y > 1e-6).collect();
    let strength = if !nonzero.is_empty() {
        let l_lav = (nonzero.iter().map(|&y| (y + 1e-5).ln()).sum::<f64>() / nonzero.len() as f64).exp();
        let log_min = p_low.max(1e-5).ln();
        let log_max = p_high.max(1e-5).ln();
        let log_range = log_max - log_min;
        if log_range > 1e-6 {
            let skew = (log_max - (l_lav + 1e-5).ln()) / log_range;
            skew.powf(1.4).clamp(0.0, 1.0)
        } else {
            1.0
        }
    } else {
        1.0
    };

    // Remap [p_low, p_high] → [black_y, white_y] at computed strength
    for pixel in pixels.iter_mut() {
        let y = luminance(*pixel);
        let normalized = (y - p_low) / image_range;
        let target_y_full = black_y + normalized * display_range;
        let target_y = y + strength * (target_y_full - y);
        if y > 1e-6 {
            let scale = (target_y / y).clamp(0.0, 1e6);
            pixel[0] = (pixel[0] * scale).clamp(0.0, 1.0);
            pixel[1] = (pixel[1] * scale).clamp(0.0, 1.0);
            pixel[2] = (pixel[2] * scale).clamp(0.0, 1.0);
        } else {
            *pixel = [black_y, black_y, black_y];
        }
    }
}

// ── gamut_compress ────────────────────────────────────────────────────────────

const GAMUT_THRESHOLD: f64 = 0.15;
const GAMUT_THRESHOLD_MAX: f64 = 0.45;

/// Nearest point on a line segment [a, b] to point p, in 3D.
/// Returns the interpolation parameter t ∈ [0, 1].
fn nearest_on_segment(p: [f64; 3], a: [f64; 3], b: [f64; 3]) -> f64 {
    let edge = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let edge_len_sq = edge[0] * edge[0] + edge[1] * edge[1] + edge[2] * edge[2];
    if edge_len_sq < 1e-10 {
        return 0.0;
    }
    let diff = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
    let dot = diff[0] * edge[0] + diff[1] * edge[1] + diff[2] * edge[2];
    (dot / edge_len_sq).clamp(0.0, 1.0)
}

fn dist_sq(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    d[0] * d[0] + d[1] * d[1] + d[2] * d[2]
}

fn lerp3(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [
        a[0] + t * (b[0] - a[0]),
        a[1] + t * (b[1] - a[1]),
        a[2] + t * (b[2] - a[2]),
    ]
}

fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Blend out-of-gamut pixels toward the nearest point on the palette hull in OKLab space.
///
/// Smoothstep from no effect at `GAMUT_THRESHOLD` to full at `GAMUT_THRESHOLD_MAX`.
pub fn gamut_compress(pixels: &mut [[f64; 3]], palette: &Palette, strength: f64) {
    if strength <= 0.0 {
        return;
    }

    let pal_linear = palette_to_linear(palette);
    let pal_lab: Vec<_> = pal_linear
        .iter()
        .map(|&[r, g, b]| rgb_to_oklab(r, g, b))
        .collect();
    let n = pal_linear.len();

    pixels.par_iter_mut().for_each(|pixel| {
        let px_lab = rgb_to_oklab(pixel[0], pixel[1], pixel[2]);
        let px_lab_arr = [px_lab.l, px_lab.a, px_lab.b];

        // Find nearest point on any palette edge or vertex
        let mut best_dist_sq = f64::INFINITY;
        let mut best_target = *pixel;

        // Edges
        for i in 0..n {
            for j in (i + 1)..n {
                let a_lab = [pal_lab[i].l, pal_lab[i].a, pal_lab[i].b];
                let b_lab = [pal_lab[j].l, pal_lab[j].a, pal_lab[j].b];
                let t = nearest_on_segment(px_lab_arr, a_lab, b_lab);
                let nearest_lab = lerp3(a_lab, b_lab, t);
                let d = dist_sq(px_lab_arr, nearest_lab);
                if d < best_dist_sq {
                    best_dist_sq = d;
                    best_target = lerp3(pal_linear[i], pal_linear[j], t);
                }
            }
        }

        // Vertices
        for i in 0..n {
            let v_lab = [pal_lab[i].l, pal_lab[i].a, pal_lab[i].b];
            let d = dist_sq(px_lab_arr, v_lab);
            if d < best_dist_sq {
                best_dist_sq = d;
                best_target = pal_linear[i];
            }
        }

        let nearest_dist = best_dist_sq.sqrt();
        let blend = smoothstep(GAMUT_THRESHOLD, GAMUT_THRESHOLD_MAX, nearest_dist) * strength;

        pixel[0] = (pixel[0] + blend * (best_target[0] - pixel[0])).clamp(0.0, 1.0);
        pixel[1] = (pixel[1] + blend * (best_target[1] - pixel[1])).clamp(0.0, 1.0);
        pixel[2] = (pixel[2] + blend * (best_target[2] - pixel[2])).clamp(0.0, 1.0);
    });
}

/// Apply gamut compression at full strength (1.0).
pub fn auto_gamut_compress(pixels: &mut [[f64; 3]], palette: &Palette) {
    gamut_compress(pixels, palette, 1.0);
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    fn spectra_palette() -> &'static Palette {
        use crate::measured_palettes::SPECTRA_7_3_6COLOR;
        &SPECTRA_7_3_6COLOR
    }

    #[test]
    fn compress_strength_zero_is_identity() {
        let mut pixels = vec![[0.8, 0.4, 0.2_f64]];
        let original = pixels[0];
        compress_dynamic_range(&mut pixels, spectra_palette(), 0.0);
        assert_eq!(pixels[0], original);
    }

    #[test]
    fn compress_reduces_highlights() {
        // A bright pixel should be dimmed toward the display white point
        let mut pixels = vec![[1.0, 1.0, 1.0_f64]];
        compress_dynamic_range(&mut pixels, spectra_palette(), 1.0);
        assert!(luminance(pixels[0]) < 1.0, "highlight should be compressed");
    }

    #[test]
    fn gamut_compress_identity_on_palette_colors() {
        // A pixel that exactly matches a palette color should be unchanged
        let pal = spectra_palette();
        let [r, g, b] = pal.colors[0];
        let mut pixels = vec![[
            srgb_channel_to_linear(r),
            srgb_channel_to_linear(g),
            srgb_channel_to_linear(b),
        ]];
        let original = pixels[0];
        gamut_compress(&mut pixels, pal, 1.0);
        // Should be very close to original (distance = 0)
        let d = dist_sq(pixels[0], original);
        assert!(d < 1e-10, "palette color should not be moved: d={d}");
    }
}
