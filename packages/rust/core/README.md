# epaper-dithering-core

High-quality dithering for e-paper / e-ink displays.

- **OKLab + LCH-weighted color matching** — perceptually accurate, hue-preserving
- **7 error diffusion kernels** — Floyd-Steinberg, Atkinson, Burkes, Stucki, Sierra, Sierra Lite, Jarvis-Judice-Ninke
- **Ordered (Bayer 4×4) dithering** — parallelized via rayon
- **Measured palettes** — calibrated colors for real displays (Spectra 7.3", BWRY 3.97", and more)
- **Serpentine scanning** — reduces directional artifacts in error diffusion

## Examples

| Original | Mono (Burkes) | Spectra 6-color (Burkes) |
|---|---|---|
| ![Original](../../../docs/examples/katzi_original.png) | ![Mono — Burkes](../../../docs/examples/katzi_mono.png) | ![Spectra 6-color — Burkes](../../../docs/examples/katzi_spectra6.png) |

![Frankfurt at night — Spectra 6-color, auto tone compression](../../../docs/examples/frankfurt_nacht_spectra6.png)

## Usage

```rust
use epaper_dithering_core::{dither, enums::DitherMode, palettes::ColorScheme, types::ImageBuffer};

// pixels: flat RGB bytes, row-major (width × height × 3)
let img = ImageBuffer::new(&pixels, width);
let indices = dither(&img, ColorScheme::Bwr, DitherMode::FloydSteinberg, true);
```

With a measured palette:

```rust
use epaper_dithering_core::{dither, enums::DitherMode, measured_palettes::SPECTRA_7_3_6COLOR, types::ImageBuffer};

let img = ImageBuffer::new(&pixels, width);
let indices = dither(&img, &SPECTRA_7_3_6COLOR, DitherMode::Stucki, true);
```

## Related packages

- Python: [`epaper-dithering`](https://pypi.org/project/epaper-dithering/)
- JavaScript: [`@opendisplay/epaper-dithering`](https://www.npmjs.com/package/@opendisplay/epaper-dithering)

