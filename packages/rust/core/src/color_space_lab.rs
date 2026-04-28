//! OKLab color space and color matching for dithering.

// sRGB -> XYZ matrix (D65 illuminant, BruceLinbloom)
const M_RGB_XYZ: [[f64; 3]; 3] = [
    [0.4124564, 0.3575761, 0.1804375],
    [0.2126729, 0.7151522, 0.0721750],
    [0.0193339, 0.1191920, 0.9503041],
];

// XYZ -> LMS (M1, Ottosson)
const M1: [[f64; 3]; 3] = [
    [0.8189330101, 0.3618667424, -0.1288597137],
    [0.0329845436, 0.9293118715, 0.0361456387],
    [0.0482003018, 0.2643662691, 0.6338517070],
];

// cbrt(LMS) -> OKLab (M2, Ottosson)
const M2: [[f64; 3]; 3] = [
    [0.2104542553, 0.7936177850, -0.0040720468],
    [1.9779984951, -2.4285922050, 0.4505937099],
    [0.0259040371, 0.7827717662, -0.8086757660],
];

/// Chromatic-axes weight for `match_pixel_oklab`.
///
/// Empirically validated by `examples/wab_sweep.rs` against the regression fixture set
/// (Burkes + Spectra 6-color, mean OKLab ΔE on 4×4-block-averaged outputs). 1.5 is a
/// conservative choice that improves saturated subjects without over-saturating neutrals;
/// see GitHub issue #28 for the methodology and justification.
pub const WAB: f64 = 1.5;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OkLab {
    pub l: f64,
    pub a: f64,
    pub b: f64,
}

/// Linear RGB → OKLab. Pipeline: RGB → XYZ → LMS → cbrt → OKLab.
pub fn rgb_to_oklab(r: f64, g: f64, b: f64) -> OkLab {
    let x  = M_RGB_XYZ[0][0] * r + M_RGB_XYZ[0][1] * g + M_RGB_XYZ[0][2] * b;
    let y  = M_RGB_XYZ[1][0] * r + M_RGB_XYZ[1][1] * g + M_RGB_XYZ[1][2] * b;
    let z  = M_RGB_XYZ[2][0] * r + M_RGB_XYZ[2][1] * g + M_RGB_XYZ[2][2] * b;

    let l = M1[0][0] * x + M1[0][1] * y + M1[0][2] * z;
    let m = M1[1][0] * x + M1[1][1] * y + M1[1][2] * z;
    let s = M1[2][0] * x + M1[2][1] * y + M1[2][2] * z;

    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    OkLab {
        l: M2[0][0] * l_ + M2[0][1] * m_ + M2[0][2] * s_,
        a: M2[1][0] * l_ + M2[1][1] * m_ + M2[1][2] * s_,
        b: M2[2][0] * l_ + M2[2][1] * m_ + M2[2][2] * s_,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PaletteLab {
    pub colors: Vec<OkLab>,
}

impl PaletteLab {
    pub fn from_linear_rgb(palette: &[[f64; 3]]) -> Self {
        let colors = palette.iter().map(|c| rgb_to_oklab(c[0], c[1], c[2])).collect();
        Self { colors }
    }
}

// M2_inv: OKLab -> LMS^(1/3) (Ottosson)
const M2_INV: [[f64; 3]; 3] = [
    [1.0,  0.3963377774,  0.2158037573],
    [1.0, -0.1055613458, -0.0638541728],
    [1.0, -0.0894841775, -1.2914855480],
];

// M1_inv: LMS -> XYZ (Ottosson)
const M1_INV: [[f64; 3]; 3] = [
    [ 1.2270138511035211, -0.5577999806518222,  0.2812561489664678],
    [-0.0405801784232806,  1.1122568696168302, -0.0716766786656012],
    [-0.0763812845057069, -0.4214819784180127,  1.5861632204407947],
];

// XYZ -> linear sRGB (IEC 61966-2-1 D65)
const M_XYZ_RGB: [[f64; 3]; 3] = [
    [ 3.2404542, -1.5371385, -0.4985314],
    [-0.9692660,  1.8760108,  0.0415560],
    [ 0.0556434, -0.2040259,  1.0572252],
];

/// OKLab → linear RGB. Output is clamped to [0, 1]. Inverse of `rgb_to_oklab`.
pub fn oklab_to_rgb(lab: OkLab) -> [f64; 3] {
    let l_ = M2_INV[0][0] * lab.l + M2_INV[0][1] * lab.a + M2_INV[0][2] * lab.b;
    let m_ = M2_INV[1][0] * lab.l + M2_INV[1][1] * lab.a + M2_INV[1][2] * lab.b;
    let s_ = M2_INV[2][0] * lab.l + M2_INV[2][1] * lab.a + M2_INV[2][2] * lab.b;

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    let x = M1_INV[0][0] * l + M1_INV[0][1] * m + M1_INV[0][2] * s;
    let y = M1_INV[1][0] * l + M1_INV[1][1] * m + M1_INV[1][2] * s;
    let z = M1_INV[2][0] * l + M1_INV[2][1] * m + M1_INV[2][2] * s;

    let r = M_XYZ_RGB[0][0] * x + M_XYZ_RGB[0][1] * y + M_XYZ_RGB[0][2] * z;
    let g = M_XYZ_RGB[1][0] * x + M_XYZ_RGB[1][1] * y + M_XYZ_RGB[1][2] * z;
    let b = M_XYZ_RGB[2][0] * x + M_XYZ_RGB[2][1] * y + M_XYZ_RGB[2][2] * z;

    [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)]
}

/// Returns the index of the closest palette color, using weighted Cartesian OKLab distance.
///
/// `dist² = (1·dL)² + (wab·da)² + (wab·db)²`
///
/// `wab > 1.0` boosts the chromatic axes relative to lightness, encouraging use of color
/// inks on saturated subjects without the achromatic singularity that the LCH formulation
/// suffers from. With `wab = 1.0` this reduces to plain Euclidean OKLab.
pub fn match_pixel_oklab(pixel: OkLab, palette: &PaletteLab, wab: f64) -> usize {
    let mut best_idx = 0;
    let mut best_dist = f64::INFINITY;
    let wab_sq = wab * wab;
    for (i, pal) in palette.colors.iter().enumerate() {
        let dl = pixel.l - pal.l;
        let da = pixel.a - pal.a;
        let db = pixel.b - pal.b;
        let dist = dl * dl + wab_sq * (da * da + db * db);
        if dist < best_dist {
            best_dist = dist;
            best_idx = i;
        }
    }
    best_idx
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn black_is_zero() {
        let lab = rgb_to_oklab(0.0, 0.0, 0.0);
        assert_relative_eq!(lab.l, 0.0, epsilon = 1e-6);
        assert_relative_eq!(lab.a, 0.0, epsilon = 1e-6);
        assert_relative_eq!(lab.b, 0.0, epsilon = 1e-6);
    }

    #[test]
    fn white_l_is_one() {
        let lab = rgb_to_oklab(1.0, 1.0, 1.0);
        assert_relative_eq!(lab.l, 1.0, epsilon = 1e-4);
        assert_relative_eq!(lab.a, 0.0, epsilon = 1e-4);
        assert_relative_eq!(lab.b, 0.0, epsilon = 1e-4);
    }

    #[test]
    fn match_exact_palette_color() {
        let red_linear = [0.2126, 0.0, 0.0_f64];
        let palette = PaletteLab::from_linear_rgb(&[red_linear, [0.0, 0.7152, 0.0]]);
        let pixel = rgb_to_oklab(red_linear[0], red_linear[1], red_linear[2]);
        assert_eq!(match_pixel_oklab(pixel, &palette, WAB), 0);
    }

    /// Cartesian OKLab matching does not collapse against achromatic palette colors;
    /// vivid purple that the legacy LCH formulation sent to black gets routed to a chromatic ink.
    #[test]
    fn oklab_cartesian_avoids_achromatic_attractor() {
        let palette = PaletteLab::from_linear_rgb(&[
            [0.005, 0.005, 0.005], // black (idx 0)
            [0.85,  0.85,  0.85],  // white (idx 1)
            [0.55,  0.45,  0.0],   // dim yellow (idx 2)
            [0.18,  0.01,  0.0],   // dim red (idx 3)
            [0.01,  0.02,  0.18],  // dim blue (idx 4)
            [0.02,  0.18,  0.04],  // dim green (idx 5)
        ]);
        let purple = rgb_to_oklab(0.4, 0.0, 0.6);
        for wab in [1.0_f64, 1.25, 1.5, 1.75, 2.0] {
            let idx = match_pixel_oklab(purple, &palette, wab);
            assert_ne!(idx, 0, "wab={wab}: purple must not map to black");
            assert_ne!(idx, 1, "wab={wab}: purple must not map to white");
        }
    }

    /// Sanity: with wab = 1.0 the function is plain Euclidean OKLab, so it picks the
    /// palette entry that minimizes `dL² + da² + db²`.
    #[test]
    fn oklab_unweighted_matches_euclidean() {
        let palette_rgb = [[0.1_f64, 0.0, 0.0], [0.0, 0.7152, 0.0]];
        let palette = PaletteLab::from_linear_rgb(&palette_rgb);
        let pixel = rgb_to_oklab(0.0, 0.7152, 0.0);
        assert_eq!(match_pixel_oklab(pixel, &palette, 1.0), 1);
        assert_eq!(match_pixel_oklab(pixel, &palette, 1.5), 1);
    }

    /// Hue still wins under Cartesian OKLab matching: a medium-red pixel goes to a darker
    /// red rather than a same-lightness gray. The `wab > 1.0` boost on `(da, db)`
    /// ensures color separation dominates lightness.
    #[test]
    fn oklab_cartesian_favors_hue_over_lightness() {
        let target       = rgb_to_oklab(0.30, 0.04, 0.04); // medium red
        let darker_red   = [0.10_f64, 0.012, 0.012];       // hue match
        let neutral_gray = [0.30_f64, 0.30,  0.30];        // lightness match
        let palette = PaletteLab::from_linear_rgb(&[darker_red, neutral_gray]);
        for wab in [1.25, 1.5, 1.75, 2.0_f64] {
            assert_eq!(
                match_pixel_oklab(target, &palette, wab), 0,
                "wab={wab}: should prefer correct hue (darker red) over correct lightness (gray)"
            );
        }
    }

}
