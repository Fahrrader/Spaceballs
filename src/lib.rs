mod actions;
mod ai;
mod characters;
mod controls;
mod health;
mod physics;
mod projectiles;
mod scenes;
mod teams;

pub use crate::ai::{handle_ai_input, AI_DEFAULT_TEAM};
pub use crate::characters::{
    calculate_character_velocity, handle_gunfire, BaseCharacterBundle,
    ControlledPlayerCharacterBundle, PLAYER_DEFAULT_TEAM,
};
pub use crate::controls::handle_player_input;
pub use crate::health::{handle_damage, EntityDamagedEvent};
pub use crate::physics::{
    handle_bullet_collision_events, RectangularObstacleBundle, OBSTACLE_STEP_SIZE,
};
pub use crate::projectiles::handle_bullets_out_of_bounds;
pub use crate::scenes::{summon_scene, SceneArg};

pub use bevy::prelude::*;
pub use heron::PhysicsPlugin;

use clap::Parser;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub const WINDOW_WIDTH: f32 = 800.0;
pub const WINDOW_HEIGHT: f32 = 800.0;

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
