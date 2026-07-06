# Changelog

## Unreleased

### Bug Fixes

* **ordered dither:** use the standard zero-mean Bayer normalization `((v+0.5)/16 − 0.5)`.
  The previous matrix had mean −1/32 and darkened every ordered-dithered image by ~8/255.
* **ordered dither:** scale the threshold amplitude to the palette's quantization step for
  grayscale palettes (`1/(levels−1)`) so dense ramps (GRAYSCALE_16) are no longer swamped
  by full-range dither noise; mono/color palettes are unchanged.
* **tone:** clamp the auto dynamic-range remap to `[0, 1]` so percentile outliers saturate
  at the display black/white points instead of extrapolating past them (dark outliers were
  crushed to pure black).
* **tone:** honor `strength` and preserve chroma for near-black pixels in the auto compressor;
  clamp skewness before `powf` to avoid a NaN that could poison every output pixel on
  high-key images.
* **tone:** pivot the shadows/highlights S-curve at perceptual mid-gray (gamma space) rather
  than linear 0.5 (≈ sRGB 188), and clamp the strength inputs to `[0, 1]`.
* **types:** `ImageBuffer::new` now validates dimensions (panics on zero width / ragged
  buffer) instead of silently truncating in release builds.

### Performance

* Fold the sRGB→XYZ→LMS conversion into Ottosson's single combined matrix (and matching
  inverse), halving the per-pixel matrix work in OKLab conversion.
* **ordered dither:** precompute a per-threshold gamma LUT, removing three `powf` calls per
  pixel (~13% faster); output is byte-identical.
* **gamut:** hoist the O(n²) palette-edge geometry out of the per-pixel loop.
## [4.0.1](https://github.com/OpenDisplay/epaper-dithering/compare/epaper-dithering-core-v4.0.0...epaper-dithering-core-v4.0.1) (2026-07-05)


### Bug Fixes

* **dither:** correct ordered-dither brightness, palette scaling, and tone-map edge cases ([6765760](https://github.com/OpenDisplay/epaper-dithering/commit/676576051fd3581df53ddcae8ee3f892a4133ba3))


### Performance Improvements

* **dither:** eliminate per-pixel powf in ordered dither; hoist gamut edges ([bafc435](https://github.com/OpenDisplay/epaper-dithering/commit/bafc435fbb4bd3c38af39d573fdd4392c01e86ff))

## [4.0.0](https://github.com/OpenDisplay/epaper-dithering/compare/epaper-dithering-core-v3.0.0...epaper-dithering-core-v4.0.0) (2026-05-21)


### ⚠ BREAKING CHANGES

* ColorScheme.GRAYSCALE_8 is now value 7 and ColorScheme.GRAYSCALE_16 is now value 6. Code that hardcoded these integer values needs to be updated.
* **api:** positional args replaced with kwargs/options/config struct. Callers must update. See README for migration guide.
* unify versioning at 3.0.0 and add crates.io release

### Features

* add criterion benchmarks ([dfbafb2](https://github.com/OpenDisplay/epaper-dithering/commit/dfbafb289c0bf8ffaaf32d4bf4372c971eb42e53))
* **api:** flatten v4 API across Rust/Python/JS ([f74cbff](https://github.com/OpenDisplay/epaper-dithering/commit/f74cbff94404eac994e8222a4f0683653b73e898))
* derive measured palettes from Rust FFI ([081f2c4](https://github.com/OpenDisplay/epaper-dithering/commit/081f2c41f3a205c529ad6c209b6de1aabf672c18))
* preserve exact display colors for measured palettes ([2108514](https://github.com/OpenDisplay/epaper-dithering/commit/21085148ed11851790c7349e33566a892e46a7d7))
* **python:** rewrite as maturin mixed Rust/Python package ([3f54dfe](https://github.com/OpenDisplay/epaper-dithering/commit/3f54dfebc8c6132f096f901f08ec13ea587fa601))
* **rust:** add tone mapping and gamut compression to core ([1a0b4ae](https://github.com/OpenDisplay/epaper-dithering/commit/1a0b4ae633dd6c6350108456454a6306aa5f76a3))


### Bug Fixes

* correct GRAYSCALE_16=6 and GRAYSCALE_8=7 to match firmware ([a991067](https://github.com/OpenDisplay/epaper-dithering/commit/a991067820529081afb25ff5ab786c58c1c527ee))
* fix x86_64 wheel publishing and reduce crate size ([54d625c](https://github.com/OpenDisplay/epaper-dithering/commit/54d625c6a20ef0cfa2d1fdca9e5bfc99fcb8ee2a))
* optimize tone_map and fix LUT rounding ([766f41b](https://github.com/OpenDisplay/epaper-dithering/commit/766f41b62943d0a8e12be5be0d5e7ab37eae6756))
* publish musllinux wheel for Alpine Linux compatibility ([9c14bc6](https://github.com/OpenDisplay/epaper-dithering/commit/9c14bc66c77be2b3fb7608b977bcafb1d932e372))
* **rust:** apply Bayer threshold in sRGB space ([#27](https://github.com/OpenDisplay/epaper-dithering/issues/27)) ([1844f3b](https://github.com/OpenDisplay/epaper-dithering/commit/1844f3b801690ec9eb7424f8a2e88948481ba3e3))
* **rust:** replace LCH-weighted matcher with Cartesian OKLab ([#28](https://github.com/OpenDisplay/epaper-dithering/issues/28)) ([8b6686b](https://github.com/OpenDisplay/epaper-dithering/commit/8b6686b0933f078d8f08bd263eaad5bf2db3b3d5))
* satisfy rust clippy checks ([5563269](https://github.com/OpenDisplay/epaper-dithering/commit/5563269889c52909f7d88a01337241bf4043dd92))


### Miscellaneous Chores

* unify versioning at 3.0.0 and add crates.io release ([d1e6222](https://github.com/OpenDisplay/epaper-dithering/commit/d1e62222ef1e7f2d3c19a1c412b278877771337a))
