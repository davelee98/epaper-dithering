//! OKLab color space and LCH-weighted color matching.
//!
//! LCH weighting: hue errors can't be corrected by diffusion, lightness errors can.

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

// LCH distance weights (hue > chroma > lightness)
const WL: f64 = 0.5;
const WC: f64 = 3.0; // scaled for OKLab's C range [0, ~0.4]
const WH: f64 = 6.0;

#[derive(Debug, Clone, Copy)]
pub struct OkLab {
    pub l: f64,
    pub a: f64,
    pub b: f64,
}

impl OkLab {
    pub fn chroma(&self) -> f64 {
        (self.a * self.a + self.b * self.b).sqrt()
    }
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

pub struct PaletteLab {
    pub colors: Vec<OkLab>,
    pub chromas: Vec<f64>,
}

impl PaletteLab {
    pub fn from_linear_rgb(palette: &[[f64; 3]]) -> Self {
        let colors: Vec<OkLab> = palette
            .iter()
            .map(|c| rgb_to_oklab(c[0], c[1], c[2]))
            .collect();
        let chromas = colors.iter().map(|c| c.chroma()).collect();
        Self { colors, chromas }
    }
}

/// Returns the index of the closest palette color (LCH-weighted OKLab distance).
pub fn match_pixel_lch(pixel: OkLab, palette: &PaletteLab) -> usize {
    let pc = pixel.chroma();

    let mut best_idx = 0;
    let mut best_dist = f64::INFINITY;

    for (i, (pal, &pal_c)) in palette.colors.iter().zip(palette.chromas.iter()).enumerate() {
        let dl = pixel.l - pal.l;
        let da = pixel.a - pal.a;
        let db = pixel.b - pal.b;
        let dc = pc - pal_c;
        let dh_sq = (da * da + db * db - dc * dc).max(0.0);

        let dist = (WL * dl) * (WL * dl) + (WC * dc) * (WC * dc) + WH * WH * dh_sq;
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
        assert_eq!(match_pixel_lch(pixel, &palette), 0);
    }

    #[test]
    fn lch_weights_favor_hue_over_lightness() {
        // Target: medium-dark red-ish pixel
        // Option A (index 0): darker red — same hue, lower lightness
        // Option B (index 1): neutral gray — same-ish lightness, completely different hue
        //
        // With WH=6.0 >> WL=0.5, option A (correct hue) should win over option B.
        let target         = rgb_to_oklab(0.30, 0.04, 0.04); // medium red
        let option_a_rgb   = [0.10_f64, 0.012, 0.012];       // darker red — hue match
        let option_b_rgb   = [0.30_f64, 0.30,  0.30];        // neutral gray — lightness match

        let palette = PaletteLab::from_linear_rgb(&[option_a_rgb, option_b_rgb]);
        let result = match_pixel_lch(target, &palette);
        assert_eq!(
            result, 0,
            "LCH matching should prefer correct hue (darker red) over correct lightness (gray)"
        );
    }
}
