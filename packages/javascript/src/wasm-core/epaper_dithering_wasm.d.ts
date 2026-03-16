/* tslint:disable */
/* eslint-disable */

/**
 * Dither a flat RGB image for an idealized e-paper color scheme.
 *
 * - `pixels`: flat RGB bytes, row-major (len = width × height × 3)
 * - `scheme_id`: firmware color scheme (0=mono … 7=grayscale16)
 * - `mode_id`: dither algorithm (0=none … 8=jjn)
 * - `tone_compression`: ignored for idealized palettes — pass 0.0
 * - `gamut_compression`: 0.0 = off, 0.0–1.0 = fixed strength
 *
 * Returns a Uint8Array of palette indices (one per pixel).
 */
export function dither_image(pixels: Uint8Array, width: number, scheme_id: number, mode_id: number, serpentine: boolean, tone_compression?: number | null, gamut_compression?: number | null): Uint8Array;

/**
 * Dither a flat RGB image using a measured ColorPalette.
 *
 * - `pixels`: flat RGB bytes, row-major (len = width × height × 3)
 * - `palette_bytes`: flat RGB bytes for each palette color (len = n_colors × 3)
 * - `accent_idx`: index of the accent color in the palette
 * - `mode_id`: dither algorithm (0=none … 8=jjn)
 * - `tone_compression`: null = auto, 0.0 = off, 0.0–1.0 = fixed
 * - `gamut_compression`: null = auto, 0.0 = off, 0.0–1.0 = fixed
 *
 * Returns a Uint8Array of palette indices (one per pixel).
 */
export function dither_image_palette(pixels: Uint8Array, width: number, palette_bytes: Uint8Array, accent_idx: number, mode_id: number, serpentine: boolean, tone_compression?: number | null, gamut_compression?: number | null): Uint8Array;
