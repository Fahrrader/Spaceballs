mod actions;
mod ai;
mod characters;
mod controls;
mod guns;
mod health;
mod physics;
mod projectiles;
mod scenes;
mod teams;

pub use crate::actions::CharacterActionInput;
pub use crate::ai::handle_ai_input;
pub use crate::characters::{
    calculate_character_velocity, handle_gun_picking, handle_inventory_layout_change,
    handle_letting_gear_go, PlayerCharacterBundle,
};
pub use crate::controls::{
    handle_gamepad_connections, handle_online_player_input, process_input, InputHandlingSet,
};
pub use crate::guns::{
    handle_gun_arriving_at_rest, handle_gun_idle_bobbing, handle_gunfire, Gun, GunBundle, GunPreset,
};
pub use crate::health::handle_death;
pub use crate::physics::{
    handle_entities_out_of_bounds, RectangularObstacleBundle, SpaceballsPhysicsPlugin, Velocity,
    CHUNK_SIZE,
};
pub use crate::projectiles::{handle_bullet_collision_events, handle_railgun_penetration_damage};
pub use crate::scenes::{summon_scene, SceneArg};
pub use crate::teams::{AI_DEFAULT_TEAM, PLAYER_DEFAULT_TEAM};

pub use bevy::prelude::*;
pub use bevy::render::camera::{camera_system, RenderTarget};
use bevy::window::{PrimaryWindow, WindowRef, WindowResized};
pub use bevy_rapier2d::prelude::{RapierConfiguration, RapierPhysicsPlugin};

pub use rand::{
    distributions::Standard,
    prelude::{Distribution, StdRng},
    Rng, SeedableRng,
};

use clap::Parser;

use crate::scenes::OptionalSceneArg;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use bevy_ggrs::{ggrs, Session};
pub use bevy_ggrs::{GGRSPlugin, GGRSSchedule};
use bevy_matchbox::prelude::{MatchboxSocket, PeerId, SingleChannel};

/// Client's current state of the game.
#[derive(States, Clone, Default, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    // MainMenu,
    #[default]
    Matchmaking,
    InGame,
}

// todo:mp displace into 'multiplayer' module
/// Placeholder struct onto which the GGRS config's types are mapped.
pub struct GgrsConfig;

// todo:mp state sync with the host
impl ggrs::Config for GgrsConfig {
    // 4 directions, fire, reload and 2 interact actions fit perfectly in a single byte
    // but todo:mp rework for a more complex struct
    type Input = u8;
    type State = u8;
    // Matchbox' WebRtcSocket addresses are called `PeerId`s
    type Address = PeerId;
}

/// Expected - and maximum - player count for the game session.
#[derive(Resource)]
pub struct PlayerCount(usize);

/// Initialize a socket for connecting to the matchbox server.
pub fn start_matchbox_socket(mut commands: Commands) {
    let room_url = "ws://127.0.0.1:3536/extreme_bevy?next=2";
    info!("connecting to matchbox server: {:?}", room_url);
    commands.insert_resource(MatchboxSocket::new_ggrs(room_url));
}

/// Initialize the multiplayer session.
/// Having input systems in GGRS schedule will not execute them until a session is initialized.
pub fn wait_for_players(
    mut commands: Commands,
    mut socket: ResMut<MatchboxSocket<SingleChannel>>,
    player_count: Res<PlayerCount>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Check for new connections
    socket.update_peers();
    let players = socket.players();

    // todo:mp try players.len() (i.e. drop-in)
    // if there is not enough players, wait
    if players.len() < player_count.0 {
        /*if session.is_none() {
           // remove resource
           // unneeded, do drop-in, drop-out
        }*/
        return; // wait for more players
    }

    info!("All peers have joined, going in-game");

    // create a GGRS P2P session
    let mut session_builder = ggrs::SessionBuilder::<GgrsConfig>::new()
        .with_num_players(player_count.0)
        .with_input_delay(2);

    for (i, player) in players.into_iter().enumerate() {
        session_builder = session_builder
            .add_player(player, i)
            .expect("failed to add player");
        // todo:mp add players here?
    }

    // move the channel out of the socket (required because GGRS takes ownership of it)
    let channel = socket.take_channel(0).unwrap();

    // start the GGRS session
    let ggrs_session = session_builder
        .start_p2p_session(channel)
        .expect("failed to start session");

    commands.insert_resource(Session::P2PSession(ggrs_session));
    next_state.set(GameState::InGame);
}

/// State of chaos!
#[derive(Resource)]
pub struct RandomState(pub StdRng);

impl RandomState {
    #[inline]
    pub fn gen<T>(&mut self) -> T
    where
        Standard: Distribution<T>,
    {
        self.0.gen()
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
        title: "Cosmic Spaceball Tactical Action Arena".to_string(),
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
        title: "Cosmic Spaceball Tactical Action Arena".to_string(),
        resolution: (width, height).into(),
        // todo uncomment if window size must be fixed (events of resizing at the window creation still apply)
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
    scene: Option<SceneArg>,
}

/// Try to get input from the command line interface on which scene to load.
#[cfg(not(target_arch = "wasm32"))]
pub fn parse_scene_ext_input() -> OptionalSceneArg {
    let args = Cli::parse();
    OptionalSceneArg(args.scene)
}

/// Try to get input from the JS side's URL arguments on which scene to load.
#[cfg(target_arch = "wasm32")]
pub fn parse_scene_ext_input() -> OptionalSceneArg {
    OptionalSceneArg(get_scene_from_js().try_into().ok())
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
