/// Measured color palettes for real e-paper displays.
///
/// These are photographically calibrated — colors reflect what the display
/// actually produces, not the ideal sRGB values. Use these for best dithering
/// quality on known hardware.
///
/// Color order within each palette matches the Python package (firmware contract).

use crate::palettes::Palette;

// ── Spectra 7.3" 6-color ─────────────────────────────────────────────────────

/// Spectra 7.3" 6-color (BWGBRY layout).
/// Measured 2026-02-03, iPhone 15 Pro Max RAW, 6500K reference.
pub static SPECTRA_7_3_6COLOR: Palette = Palette {
    colors: &[
        [26,  13,  35],   // black
        [185, 202, 205],  // white
        [202, 184,   0],  // yellow
        [121,   9,   0],  // red
        [  0,  69, 139],  // blue
        [ 40,  82,  57],  // green
    ],
    accent_idx: 3, // red
};

/// Spectra 7.3" 6-color v2.
/// Measured 2026-03-15, DNG with linear tone curve.
pub static SPECTRA_7_3_6COLOR_V2: Palette = Palette {
    colors: &[
        [ 31,  24,  41],  // black
        [168, 180, 182],  // white
        [180, 173,   0],  // yellow
        [113,  24,  19],  // red
        [ 36,  70, 139],  // blue
        [ 50,  84,  60],  // green
    ],
    accent_idx: 3, // red
};

// ── Monochrome displays ───────────────────────────────────────────────────────

/// 4.26" Monochrome. TODO: measure actual display.
pub static MONO_4_26: Palette = Palette {
    colors: &[
        [  5,   5,   5],  // black
        [220, 220, 220],  // white
    ],
    accent_idx: 0,
};

// ── BWRY displays ─────────────────────────────────────────────────────────────

/// 4.2" BWRY. TODO: measure actual display.
pub static BWRY_4_2: Palette = Palette {
    colors: &[
        [  5,   5,   5],  // black
        [200, 200, 200],  // white
        [200, 180,   0],  // yellow
        [120,  15,   5],  // red
    ],
    accent_idx: 3,
};

/// 3.97" BWRY — EP397YR_800x480.
/// Measured 2026-03-06, iPhone RAW, paper reference RGB(205,205,205).
pub static BWRY_3_97: Palette = Palette {
    colors: &[
        [ 10,   7,  14],  // black
        [173, 178, 174],  // white
        [172, 128,   0],  // yellow
        [ 85,  24,  14],  // red
    ],
    accent_idx: 3,
};

// ── Harvested displays ────────────────────────────────────────────────────────

/// Solum BWR (harvested display). TODO: measure.
pub static SOLUM_BWR: Palette = Palette {
    colors: &[
        [  5,   5,   5],
        [200, 200, 200],
        [120,  15,   5],
    ],
    accent_idx: 2,
};

/// Hanshow BWR (harvested display). TODO: measure.
pub static HANSHOW_BWR: Palette = Palette {
    colors: &[
        [  5,   5,   5],
        [200, 200, 200],
        [120,  15,   5],
    ],
    accent_idx: 2,
};

/// Hanshow BWY (harvested display). TODO: measure.
pub static HANSHOW_BWY: Palette = Palette {
    colors: &[
        [  5,   5,   5],
        [200, 200, 200],
        [200, 180,   0],
    ],
    accent_idx: 2,
};
