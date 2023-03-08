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
pub use crate::controls::{
    handle_gamepad_connections, handle_gamepad_input, handle_keyboard_input, reset_input,
};
pub use crate::health::{handle_damage, EntityDamagedEvent};
pub use crate::physics::{
    handle_bullet_collision_events, RectangularObstacleBundle, OBSTACLE_STEP_SIZE,
};
pub use crate::projectiles::handle_bullets_out_of_bounds;
pub use crate::scenes::{summon_scene, SceneArg};

pub use bevy::prelude::*;
pub use heron::PhysicsPlugin;
pub use bevy::render::camera::{camera_system, RenderTarget};
use bevy::window::WindowResized;

use clap::Parser;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// todo after the adjustment of body sizes, change to probably 200.0
pub const SCREEN_SPAN: f32 = 800.0;

pub fn calculate_projection_scale(
    mut window_resized_events: EventReader<WindowResized>,
    windows: Res<Windows>,
    mut query: Query<(&Camera, &mut OrthographicProjection)>,
) {
    if window_resized_events.len() == 0 {
        return;
    }

    let mut changed_window_ids = Vec::new();
    for event in window_resized_events.iter().rev() {
        if changed_window_ids.contains(&event.id) {
            continue;
        }

        changed_window_ids.push(event.id);
    }

    for (camera, mut projection) in query.iter_mut() {
        let window_id = match camera.target {
            RenderTarget::Window(window_id) => window_id,
            RenderTarget::Image(_) => continue,
        };
        if changed_window_ids.contains(&window_id) {
            if let Some(window) = windows
                .get(window_id)
            {
                projection.scale = SCREEN_SPAN / (window.width()).min(window.height());
            }
        }
    }
}

// todo add this optionally to a system set
pub fn handle_browser_window_resizing(
    #[cfg(target_arch = "wasm32")]
    mut windows: ResMut<Windows>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        if !detect_window_resize_from_js() { return; }
        let size = get_new_window_size_from_js();
        for window in windows.iter_mut() {
            window.set_resolution(size[0].into(), size[1].into());
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub fn create_window_descriptor(resolution: (f32, f32)) -> WindowDescriptor {
    let (width, height) = resolution;
    WindowDescriptor {
        title: "Cosmic Spaceball Tactical Action Arena".to_string(),
        width,
        height,
        // scale_factor_override: Some(1.0),
        ..default()
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn create_window_descriptor(resolution: (f32, f32)) -> WindowDescriptor {
    let (width, height) = resolution;
    WindowDescriptor {
        title: "Cosmic Spaceball Tactical Action Arena".to_string(),
        width,
        height,
        // todo uncomment if window size must be fixed (events of resizing at the window creation still apply)
        // resizable: false,
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
#[wasm_bindgen(module = "/web/main.ts")]
extern "C" {
    #[wasm_bindgen(js_name = getSceneFromUrl)]
    fn get_scene_from_js() -> String;

    #[wasm_bindgen(js_name = detectWindowResize)]
    fn detect_window_resize_from_js() -> bool;

    #[wasm_bindgen(js_name = getNewWindowSize)]
    fn get_new_window_size_from_js() -> Vec<f32>;
}
