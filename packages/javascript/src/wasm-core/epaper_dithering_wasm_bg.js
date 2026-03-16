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
 * @param {Uint8Array} pixels
 * @param {number} width
 * @param {number} scheme_id
 * @param {number} mode_id
 * @param {boolean} serpentine
 * @param {number | null} [tone_compression]
 * @param {number | null} [gamut_compression]
 * @returns {Uint8Array}
 */
export function dither_image(pixels, width, scheme_id, mode_id, serpentine, tone_compression, gamut_compression) {
    const ptr0 = passArray8ToWasm0(pixels, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.dither_image(ptr0, len0, width, scheme_id, mode_id, serpentine, !isLikeNone(tone_compression), isLikeNone(tone_compression) ? 0 : tone_compression, !isLikeNone(gamut_compression), isLikeNone(gamut_compression) ? 0 : gamut_compression);
    if (ret[3]) {
        throw takeFromExternrefTable0(ret[2]);
    }
    var v2 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
    return v2;
}

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
 * @param {Uint8Array} pixels
 * @param {number} width
 * @param {Uint8Array} palette_bytes
 * @param {number} accent_idx
 * @param {number} mode_id
 * @param {boolean} serpentine
 * @param {number | null} [tone_compression]
 * @param {number | null} [gamut_compression]
 * @returns {Uint8Array}
 */
export function dither_image_palette(pixels, width, palette_bytes, accent_idx, mode_id, serpentine, tone_compression, gamut_compression) {
    const ptr0 = passArray8ToWasm0(pixels, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArray8ToWasm0(palette_bytes, wasm.__wbindgen_malloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.dither_image_palette(ptr0, len0, width, ptr1, len1, accent_idx, mode_id, serpentine, !isLikeNone(tone_compression), isLikeNone(tone_compression) ? 0 : tone_compression, !isLikeNone(gamut_compression), isLikeNone(gamut_compression) ? 0 : gamut_compression);
    if (ret[3]) {
        throw takeFromExternrefTable0(ret[2]);
    }
    var v3 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
    return v3;
}
export function __wbindgen_cast_0000000000000001(arg0, arg1) {
    // Cast intrinsic for `Ref(String) -> Externref`.
    const ret = getStringFromWasm0(arg0, arg1);
    return ret;
}
export function __wbindgen_init_externref_table() {
    const table = wasm.__wbindgen_externrefs;
    const offset = table.grow(4);
    table.set(0, undefined);
    table.set(offset + 0, undefined);
    table.set(offset + 1, null);
    table.set(offset + 2, true);
    table.set(offset + 3, false);
}
function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

let WASM_VECTOR_LEN = 0;


let wasm;
export function __wbg_set_wasm(val) {
    wasm = val;
}
