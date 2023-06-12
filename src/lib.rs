mod ai;
mod characters;
mod controls;
mod guns;
mod health;
mod multiplayer;
mod physics;
mod projectiles;
mod scenes;
mod teams;
mod ui;

pub use ai::{handle_ai_input, AIActionRoutine};
pub use characters::{
    calculate_character_velocity, handle_gun_picking, handle_inventory_layout_change,
    handle_letting_gear_go, PlayerCharacterBundle,
};
pub use controls::{
    handle_gamepad_connections, handle_online_player_input, process_input, CharacterActionInput,
    InputHandlingSet,
};
pub use guns::{systems::*, Equipped, Gun, GunBundle, GunPreset};
pub use health::{handle_death, Dying, Health};
pub use multiplayer::{
    detect_desync, sever_connection, start_matchbox_socket, wait_for_players, GGRSConfig,
    GGRSPlugin, GGRSSchedule, PlayerCount,
};
pub use physics::{
    handle_entities_out_of_bounds, ActiveEvents, RectangularObstacleBundle, Sleeping,
    SpaceballsPhysicsPlugin, Velocity, CHUNK_SIZE,
};
pub use projectiles::handle_bullet_collision_events;
pub use scenes::{summon_scene, SceneSelector};
pub use teams::{AI_DEFAULT_TEAM, PLAYER_DEFAULT_TEAM};
pub use ui::menu::{MenuPlugin, MenuState};

pub use bevy::prelude::*;
pub use bevy::render::camera::{camera_system, RenderTarget};
pub use bevy_rapier2d::prelude::{RapierConfiguration, RapierPhysicsPlugin};
pub use rand::{
    distributions::Standard,
    prelude::{Distribution, StdRng},
    Rng, SeedableRng,
};

use bevy::core_pipeline::bloom::{BloomPrefilterSettings, BloomSettings};
use bevy::reflect::ReflectFromReflect;
use bevy::window::{PrimaryWindow, WindowRef, WindowResized};
use clap::Parser;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Client's current state of the game.
#[derive(States, Clone, Default, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    #[default]
    MainMenu,
    Matchmaking,
    InGame,
}

pub fn standard_setup(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            ..default()
        },
        BloomSettings {
            prefilter_settings: BloomPrefilterSettings {
                threshold: 0.5,
                threshold_softness: 0.0, // play around
            },
            ..default()
        },
    ));
}

/// State of chaos!
#[derive(Resource, Clone, Debug, PartialEq, Eq, Reflect, FromReflect)]
#[reflect_value(Debug, Resource, FromReflect)]
pub struct EntropyGenerator(pub StdRng);

impl EntropyGenerator {
    #[inline]
    pub fn gen<T>(&mut self) -> T
    where
        Standard: Distribution<T>,
    {
        self.0.gen()
    }

    pub fn new(seed: u64) -> Self {
        Self(StdRng::seed_from_u64(seed))
    }

    pub fn fork(&mut self) -> Self {
        Self(StdRng::from_rng(&mut self.0).unwrap())
    }
}

impl Default for EntropyGenerator {
    fn default() -> Self {
        Self::new(0)
    }
}

// todo after the adjustment of body sizes, change to probably 200.0
/// The size of a side of the arena, in in-game units of distance.
pub const SCREEN_SPAN: f32 = 800.0;

/// If the primary game window was resized, change the main camera's projection scale accordingly
/// to keep the arena size the same.
pub fn calculate_main_camera_projection_scale(
    mut window_resized_events: EventReader<WindowResized>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<(&Camera, &mut OrthographicProjection)>,
) {
    if window_resized_events.len() == 0 {
        return;
    }

    // find the primary changed window (of the game arena)
    let mut changed_window_ids = Vec::new();
    for event in window_resized_events.iter() {
        if changed_window_ids.contains(&event.window) {
            continue;
        }
        changed_window_ids.push(event.window);
    }

    // find the camera and projection that use the primary window
    for (camera, mut projection) in query.iter_mut() {
        let window = match camera.target {
            RenderTarget::Window(window) => window,
            RenderTarget::Image(_) => continue,
        };

        if match window {
            WindowRef::Primary => true,
            WindowRef::Entity(window_entity) => changed_window_ids.contains(&window_entity),
        } {
            if let Ok(window) = primary_window_query.get_single() {
                projection.scale = SCREEN_SPAN / (window.width()).min(window.height());
            }
        }
    }
}

// todo add this optionally to a system set
/// System to query JS whether the browser window size has changed, and resize the game window
/// according to the JS-supplied data.
pub fn handle_browser_window_resizing(
    #[cfg(target_arch = "wasm32")] mut primary_window_query: Query<
        &mut Window,
        With<PrimaryWindow>,
    >,
) {
    #[cfg(target_arch = "wasm32")]
    {
        if !detect_window_resize_from_js() {
            return;
        }
        let size = get_new_window_size_from_js();
        for mut window in primary_window_query.iter_mut() {
            window.resolution = (size[0], size[1]).into();
        }
    }
}

/// Make a "window" for browser.
#[cfg(target_arch = "wasm32")]
pub fn create_window(width: f32, height: f32) -> Window {
    Window {
        title: "Cosmic Spaceball Tactical Action Arena".to_owned(),
        resolution: (width, height).into(),
        // scale_factor_override: Some(1.0),
        // fill the entire browser window
        // fit_canvas_to_parent: true,
        // don't hijack keyboard shortcuts like F5, F6, F12, Ctrl+R etc.
        // prevent_default_event_handling: false,
        ..default()
    }
}

/// Make a window for desktop.
#[cfg(not(target_arch = "wasm32"))]
pub fn create_window(width: f32, height: f32) -> Window {
    Window {
        title: "Cosmic Spaceball Tactical Action Arena".to_owned(),
        resolution: (width, height).into(),
        // uncomment if window size must be fixed (events of resizing at the window creation still apply)
        // resizable: false,
        ..default()
    }
}

/// The set of acceptable arguments for the command line interface.
#[derive(Parser)]
#[clap(version, about)]
struct Cli {
    /// The scene to load at the game start
    #[clap(value_enum, short, long)]
    scene: Option<SceneSelector>,
}

/// Try to get input from the command line interface on which scene to load.
#[cfg(not(target_arch = "wasm32"))]
pub fn parse_scene_ext_input() -> Option<SceneSelector> {
    let args = Cli::parse();
    args.scene
}

/// Try to get input from the JS side's URL arguments on which scene to load.
#[cfg(target_arch = "wasm32")]
pub fn parse_scene_ext_input() -> Option<SceneSelector> {
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
    /// Get input from the JS side on which scene to load as an argument in its raw form.
    #[wasm_bindgen(js_name = getSceneFromUrl)]
    fn get_scene_from_js() -> String;

    /// Ask JS whether the window size got changed lately.
    #[wasm_bindgen(js_name = detectWindowResize)]
    fn detect_window_resize_from_js() -> bool;

    /// Take from JS its new window size.
    #[wasm_bindgen(js_name = getNewWindowSize)]
    fn get_new_window_size_from_js() -> Vec<f32>;
}
