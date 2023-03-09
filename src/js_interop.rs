use wasm_bindgen::prelude::*;


#[wasm_bindgen(module = "/web/main.ts")]
extern "C" {
    #[wasm_bindgen(js_name = getSceneFromUrl)]
    pub fn get_scene_from_js() -> String;

    #[wasm_bindgen(js_name = detectWindowResize)]
    pub fn detect_window_resize_from_js() -> bool;

    #[wasm_bindgen(js_name = getNewWindowSize)]
    pub fn get_new_window_size_from_js() -> Vec<f32>;

    #[wasm_bindgen(js_name = getSticksPosition)]
    pub fn get_sticks_positions_from_js() -> Vec<f32>;
}
