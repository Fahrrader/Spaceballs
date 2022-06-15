use bevy::prelude::*;
use heron::PhysicsPlugin;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub mod actions;
pub mod ai;
pub mod characters;
pub mod controls;
pub mod health;
pub mod physics;
pub mod projectiles;
pub mod teams;

use crate::ai::{handle_ai_input, AI_DEFAULT_TEAM};
use crate::characters::{
    calculate_character_velocity, handle_gunfire, BaseCharacterBundle,
    ControlledPlayerCharacterBundle, PLAYER_DEFAULT_TEAM,
};
use crate::controls::handle_player_input;
use crate::health::{handle_damage, EntityDamagedEvent};
use crate::physics::{
    handle_bullet_collision_events, RectangularObstacleBundle, OBSTACLE_STEP_SIZE,
};
use crate::projectiles::handle_bullets_out_of_bounds;

pub const WINDOW_WIDTH: f32 = 800.0;
pub const WINDOW_HEIGHT: f32 = 800.0;

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // ----- The walls of the arena
    commands.spawn_bundle(RectangularObstacleBundle::new(
        Transform::from_translation(Vec3::X * -WINDOW_WIDTH / 2.0).with_scale(Vec3::new(
            1.0,
            WINDOW_HEIGHT / OBSTACLE_STEP_SIZE + 1.0,
            1.0,
        )),
    ));
    commands.spawn_bundle(RectangularObstacleBundle::new(
        Transform::from_translation(Vec3::X * WINDOW_WIDTH / 2.0).with_scale(Vec3::new(
            1.0,
            WINDOW_HEIGHT / OBSTACLE_STEP_SIZE + 1.0,
            1.0,
        )),
    ));
    commands.spawn_bundle(RectangularObstacleBundle::new(
        Transform::from_translation(Vec3::Y * WINDOW_HEIGHT / 2.0).with_scale(Vec3::new(
            WINDOW_WIDTH / OBSTACLE_STEP_SIZE + 1.0,
            1.0,
            1.0,
        )),
    ));
    commands.spawn_bundle(RectangularObstacleBundle::new(
        Transform::from_translation(Vec3::Y * -WINDOW_HEIGHT / 2.0).with_scale(Vec3::new(
            WINDOW_WIDTH / OBSTACLE_STEP_SIZE + 1.0,
            1.0,
            1.0,
        )),
    ));
    // -----

    commands.spawn_bundle(ControlledPlayerCharacterBundle::new(
        PLAYER_DEFAULT_TEAM,
        Transform::from_translation(Vec3::new(-150.0, 0.0, 0.0)),
    ));

    commands.spawn_bundle(BaseCharacterBundle::new(
        AI_DEFAULT_TEAM,
        Transform::from_translation(Vec3::new(150.0, 0.0, 0.0))
            .with_rotation(Quat::from_axis_angle(-Vec3::Z, 30.0))
            .with_scale(Vec3::new(2.0, 3.0, 1.0)),
    ));

    commands.spawn_bundle(RectangularObstacleBundle::new(Transform::from_scale(
        Vec3::new(1.0, 2.0, 1.0),
    )));
}

#[cfg(target_arch = "wasm32")]
fn create_window_descriptor(resolution: (f32, f32)) -> WindowDescriptor {
    let (width, height) = resolution;
    WindowDescriptor {
        width,
        height,
        scale_factor_override: Some(1.0),
        ..default()
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn create_window_descriptor(resolution: (f32, f32)) -> WindowDescriptor {
    let (width, height) = resolution;
    WindowDescriptor {
        width,
        height,
        ..default()
    }
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .insert_resource(create_window_descriptor((WINDOW_WIDTH, WINDOW_HEIGHT)))
        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin::default())
        .insert_resource(ClearColor(Color::BLACK))
        .add_event::<EntityDamagedEvent>()
        .add_startup_system(setup)
        .add_system(handle_player_input)
        .add_system(handle_ai_input)
        .add_system(
            calculate_character_velocity
                .after(handle_player_input)
                .after(handle_ai_input),
        ) // todo plugin?
        .add_system(handle_gunfire.after(calculate_character_velocity))
        .add_system(handle_bullets_out_of_bounds.after(handle_gunfire))
        .add_system(handle_bullet_collision_events)
        .add_system(handle_damage.after(handle_bullet_collision_events))
        .run();
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
