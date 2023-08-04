use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/web/main.ts")]
extern "C" {
    /// Get info from the browser with JS on whether the client is a phone or similar.
    #[wasm_bindgen(js_name = detectMob)]
    pub fn is_mobile() -> bool;

    /// Get input from the JS side on which scene to load as an argument in its raw form.
    #[wasm_bindgen(js_name = getSceneFromUrl)]
    pub fn get_scene_from_js() -> String;

    /// Ask JS whether the window size got changed lately.
    #[wasm_bindgen(js_name = detectWindowResize)]
    pub fn detect_window_resize_from_js() -> bool;

    /// Take from JS its new window size.
    #[wasm_bindgen(js_name = getNewWindowSize)]
    pub fn get_new_window_size_from_js() -> Vec<f32>;

    /// Get player input from JS joysticks.
    #[wasm_bindgen(js_name = getSticksPosition)]
    pub fn get_sticks_positions_from_js() -> Vec<f32>;

    /// Send JS to make a promise to fill a buffer with paste data from the clipboard for the entity to access later.
    #[wasm_bindgen(js_name = setPasteBuffer)]
    pub fn set_js_paste_buffer(entity_index: u32);

    /// Check and get the paste from the clipboard filled into a buffer with a prior JS promise.
    #[wasm_bindgen(catch, js_name = getPasteBuffer)]
    pub fn get_js_paste_buffer(entity_index: u32) -> Result<Option<String>, JsValue>;
}
