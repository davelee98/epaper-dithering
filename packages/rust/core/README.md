# epaper-dithering-core

High-quality dithering for e-paper / e-ink displays.

- **Weighted Cartesian OKLab color matching** — perceptually accurate, hue-preserving, no achromatic-attractor bug
- **7 error diffusion kernels** — Floyd-Steinberg, Atkinson, Burkes, Stucki, Sierra, Sierra Lite, Jarvis-Judice-Ninke
- **Ordered (Bayer 4×4) dithering** — perceptually-correct sRGB-space thresholding, parallelized via rayon
- **Measured palettes** — calibrated colors for real displays (Spectra 7.3", BWRY 3.97", and more)
- **Pre-dither knobs** — exposure, saturation, shadows, highlights, dynamic-range compression, gamut compression
- **Serpentine scanning** — reduces directional artifacts in error diffusion

## Examples

![Frankfurt at night — Spectra 6-color, auto tone + gamut](../../../docs/examples/frankfurt_before_after.png)

## Usage

```rust
use epaper_dithering_core::{
    dither, DitherConfig,
    enums::DitherMode,
    palettes::ColorScheme,
    types::ImageBuffer,
};

// pixels: flat RGB bytes, row-major (width × height × 3)
let img = ImageBuffer::new(&pixels, width);
let indices = dither(&img, ColorScheme::Bwr, DitherConfig {
    mode: DitherMode::FloydSteinberg,
    ..Default::default()
});
```

With a measured palette and pre-dither adjustments:

```rust
use epaper_dithering_core::{
    dither, DitherConfig,
    enums::{DitherMode, ToneCompression, GamutCompression},
    measured_palettes::SPECTRA_7_3_6COLOR,
    types::ImageBuffer,
};

let img = ImageBuffer::new(&pixels, width);
let indices = dither(&img, &SPECTRA_7_3_6COLOR, DitherConfig {
    mode: DitherMode::Stucki,
    saturation: 1.3,           // boost saturation
    shadows: 0.4,              // lift shadows
    tone:  ToneCompression::Auto,
    gamut: GamutCompression::Auto,
    ..Default::default()
});
```

`DitherConfig` defaults: `Burkes`, `serpentine: true`, `exposure: 1.0`, `saturation: 1.0`,
`shadows: 0.0`, `highlights: 0.0`, `tone: Auto`, `gamut: Auto`.

Pipeline order: `exposure → saturation → shadows/highlights → tone → gamut → dither`.

## Related packages

- Python: [`epaper-dithering`](https://pypi.org/project/epaper-dithering/)
- JavaScript: [`@opendisplay/epaper-dithering`](https://www.npmjs.com/package/@opendisplay/epaper-dithering)
