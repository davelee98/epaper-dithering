import type { ImageBuffer, PaletteImageBuffer, ColorPalette } from './types';
import { DitherMode } from './enums';
import { ColorScheme, getPalette } from './palettes';
import {
  dither_image as wasmDitherImage,
  dither_image_palette as wasmDitherImagePalette,
  __wbg_set_wasm,
  __wbindgen_init_externref_table,
  __wbindgen_cast_0000000000000001,
} from './wasm-core/epaper_dithering_wasm_bg.js';
import wasmBytes from './wasm-core/epaper_dithering_wasm_bg.wasm';

// Synchronous WASM initialization — runs once at module load time.
// The WASM module imports two callbacks from the JS side; we provide them here.
const wasmModule = new WebAssembly.Module(wasmBytes as unknown as ArrayBuffer);
const wasmInstance = new WebAssembly.Instance(wasmModule, {
  './epaper_dithering_wasm_bg.js': {
    __wbindgen_init_externref_table,
    __wbindgen_cast_0000000000000001,
  },
});
__wbg_set_wasm(wasmInstance.exports);
(wasmInstance.exports as Record<string, () => void>).__wbindgen_start?.();

/**
 * Apply dithering algorithm to image for e-paper display.
 *
 * @param image - Input image buffer (RGBA format). Alpha is composited on white.
 * @param colorScheme - Target color scheme (ColorScheme enum) or measured palette (ColorPalette)
 * @param mode - Dithering algorithm (default: BURKES)
 * @param serpentine - Alternate row direction to reduce artifacts (default: true)
 * @param toneCompression - Dynamic range compression for measured palettes (default: 'auto').
 *   'auto' = analyze histogram and fit to display range.
 *   0.0 = disabled, 0.0–1.0 = fixed strength. Ignored for ColorScheme.
 * @param gamutCompression - Pre-dithering gamut compression (default: 'auto').
 *   Blends out-of-gamut pixels toward nearest palette color before dithering.
 *   'auto' = activate only when image exceeds palette gamut (measured palettes only).
 *   0.0 = disabled, 0.7–0.9 = fixed strength. Ignored for ColorScheme.
 * @returns Palette-indexed image buffer
 */
export function ditherImage(
  image: ImageBuffer,
  colorScheme: ColorScheme | ColorPalette,
  mode: DitherMode = DitherMode.BURKES,
  serpentine: boolean = true,
  toneCompression: number | 'auto' = 'auto',
  gamutCompression: number | 'auto' = 'auto',
): PaletteImageBuffer {
  const pixels = rgbaToRgb(image);

  if (typeof colorScheme === 'number') {
    // ColorScheme (idealized palette): tone/gamut compression are disabled
    const indices = wasmDitherImage(pixels, image.width, colorScheme, mode, serpentine, 0.0, 0.0);
    const palette = Object.values(getPalette(colorScheme).colors);
    return { width: image.width, height: image.height, indices, palette };
  } else {
    // Measured ColorPalette: pass user-supplied compression params
    const paletteColors = Object.values(colorScheme.colors);
    const paletteBytes = new Uint8Array(paletteColors.flatMap(c => [c.r, c.g, c.b]));
    const accentIdx = Object.keys(colorScheme.colors).indexOf(colorScheme.accent);
    const indices = wasmDitherImagePalette(
      pixels, image.width, paletteBytes, accentIdx, mode, serpentine,
      parseCompression(toneCompression),
      parseCompression(gamutCompression),
    );
    return { width: image.width, height: image.height, indices, palette: paletteColors };
  }
}

/** Convert RGBA input to flat RGB bytes, compositing transparency on white (sRGB space). */
function rgbaToRgb(image: ImageBuffer): Uint8Array {
  const n = image.width * image.height;
  const rgb = new Uint8Array(n * 3);
  for (let i = 0; i < n; i++) {
    const s = i * 4;
    const a = image.data[s + 3] / 255;
    const inv = 1 - a;
    rgb[i * 3]     = Math.round(image.data[s]     * a + 255 * inv);
    rgb[i * 3 + 1] = Math.round(image.data[s + 1] * a + 255 * inv);
    rgb[i * 3 + 2] = Math.round(image.data[s + 2] * a + 255 * inv);
  }
  return rgb;
}

/** Map 'auto' → undefined (Rust None = auto), number → pass through (Rust Some). */
function parseCompression(v: number | 'auto'): number | undefined {
  return v === 'auto' ? undefined : v;
}
