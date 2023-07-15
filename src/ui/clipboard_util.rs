use bevy::log::prelude::*;
use bevy::prelude::Component;
#[cfg(target_arch = "wasm32")]
use bevy::tasks::AsyncComputeTaskPool;
#[cfg(not(target_arch = "wasm32"))]
use clipboard::{ClipboardContext, ClipboardProvider};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub struct Clipboard;

impl Clipboard {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn write(contents: String) {
        write_to_clipboard(contents).unwrap_or_else(|err| {
            error!(err);
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn read() -> String {
        read_from_clipboard().unwrap_or_else(|err| {
            error!(err);
            String::new()
        })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn write(contents: String) {
        let clipboard_task_pool = AsyncComputeTaskPool::get();
        clipboard_task_pool
            .spawn(async move {
                write_to_clipboard(contents)
                    .await
                    .map_err(|err| error!(err))
            })
            .detach();
    }
}

#[derive(Component)]
pub(crate) struct PasteJob;

#[cfg(not(target_arch = "wasm32"))]
/// Copy a string to the OS' clipboard.
fn write_to_clipboard(contents: String) -> Result<(), String> {
    ClipboardProvider::new()
        .and_then(|mut ctx: ClipboardContext| ctx.set_contents(contents))
        .map_err(|err| err.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
/// Copy a string from the OS' clipboard.
fn read_from_clipboard() -> Result<String, String> {
    ClipboardProvider::new()
        .and_then(|mut ctx: ClipboardContext| ctx.get_contents())
        .map_err(|err| err.to_string())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
/// Copy a string to the WebAssembly's clipboard.
pub async fn write_to_clipboard(data: String) -> Result<(), String> {
    let window =
        web_sys::window().ok_or("No window object found. Are you running this in a browser?")?;
    let promise = window
        .navigator()
        .clipboard()
        .ok_or("Permission denied: Clipboard access is not allowed.".to_string())?
        .write_text(&data);

    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(|_| "Failed to write to the clipboard.".to_string())?;

    Ok(())
}
