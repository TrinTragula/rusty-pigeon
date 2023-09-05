import * as wasm from "./rustypigeonwasm_bg.wasm";
import { __wbg_set_wasm } from "./rustypigeonwasm_bg.js";
__wbg_set_wasm(wasm);
export * from "./rustypigeonwasm_bg.js";
