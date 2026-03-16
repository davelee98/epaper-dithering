/* @ts-self-types="./epaper_dithering_wasm.d.ts" */

import * as wasm from "./epaper_dithering_wasm_bg.wasm";
import { __wbg_set_wasm } from "./epaper_dithering_wasm_bg.js";
__wbg_set_wasm(wasm);
wasm.__wbindgen_start();
export {
    dither_image, dither_image_palette
} from "./epaper_dithering_wasm_bg.js";
