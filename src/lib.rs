mod scenes;
mod actions;
mod ai;
mod character;
mod collisions;
mod health;
mod movement;
mod projectiles;
mod team;

pub use crate::actions::{handle_player_input, ActionInput};
pub use crate::ai::AI_DEFAULT_TEAM;
pub use crate::character::{
    handle_gunfire, BaseCharacterBundle, ControlledPlayerCharacterBundle, PLAYER_DEFAULT_TEAM,
};
pub use crate::health::{calculate_damages, EntityDamagedEvent};
pub use crate::movement::handle_movement;
pub use crate::projectiles::{handle_bullet_collision, handle_bullet_flight};
pub use crate::scenes::{summon_scene, SceneArg};

pub use bevy::prelude::*;
use clap::Parser;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
pub fn create_window_descriptor(resolution: (f32, f32)) -> WindowDescriptor {
    let (width, height) = resolution;
    WindowDescriptor {
        width,
        height,
        scale_factor_override: Some(1.0),
        ..default()
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn create_window_descriptor(resolution: (f32, f32)) -> WindowDescriptor {
    let (width, height) = resolution;
    WindowDescriptor {
        width,
        height,
        ..default()
    }
}

#[derive(Parser)]
#[clap(version, about)]
struct Cli {
    /// The scene to load at the game start
    #[clap(value_enum, short, long)]
    scene: Option<SceneArg>,
}

#[cfg(not(target_arch = "wasm32"))]
pub fn parse_scene_ext_input() -> Option<SceneArg> {
    let args = Cli::parse();
    args.scene
}

#[cfg(target_arch = "wasm32")]
pub fn parse_scene_ext_input() -> Option<SceneArg> {
    get_scene_from_js().try_into().ok()
}

#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! log {
    () => (println!());
    ($($arg:tt)*) => ({
        println!($($arg)*)
    })
}

#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! log {
    () => (log("\n"));
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(module = "/public/main.js")]
extern "C" {
    #[wasm_bindgen(js_name = getSceneFromUrl)]
    fn get_scene_from_js() -> String;
}
