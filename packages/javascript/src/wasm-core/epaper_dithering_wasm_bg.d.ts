/** Internal wasm-bindgen background module — do not import directly in user code. */
export function dither_image(pixels: Uint8Array, width: number, scheme_id: number, mode_id: number, serpentine: boolean, tone_compression?: number | null, gamut_compression?: number | null): Uint8Array;
export function dither_image_palette(pixels: Uint8Array, width: number, palette_bytes: Uint8Array, accent_idx: number, mode_id: number, serpentine: boolean, tone_compression?: number | null, gamut_compression?: number | null): Uint8Array;
export function __wbg_set_wasm(val: WebAssembly.Exports): void;
export function __wbindgen_init_externref_table(): void;
export function __wbindgen_cast_0000000000000001(arg0: number, arg1: number): string;
