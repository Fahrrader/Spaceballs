use bevy::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub mod controls;
pub mod ai;
pub mod characters;
pub mod collisions;
pub mod health;
pub mod movement;
pub mod projectiles;
pub mod teams;

use crate::controls::{handle_player_input, ActionInput};
use crate::ai::AI_DEFAULT_TEAM;
use crate::characters::{
    handle_gunfire, BaseCharacterBundle, ControlledPlayerCharacterBundle, PLAYER_DEFAULT_TEAM,
};
use crate::health::{calculate_damages, EntityDamagedEvent};
use crate::movement::handle_movement;
use crate::projectiles::{handle_bullet_collision, handle_bullet_flight};

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    commands.spawn_bundle(ControlledPlayerCharacterBundle::new(
        PLAYER_DEFAULT_TEAM,
        Transform::default(),
    ));

    commands.spawn_bundle(BaseCharacterBundle::new(
        AI_DEFAULT_TEAM,
        Transform::from_scale(Vec3::new(2.0, 3.0, 1.0))
            .with_rotation(Quat::from_axis_angle(-Vec3::Z, 30.0)),
    ));
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
        .insert_resource(create_window_descriptor((800.0, 800.0)))
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .init_resource::<ActionInput>()
        .add_startup_system(setup)
        .add_event::<EntityDamagedEvent>()
        .add_system(handle_player_input)
        .add_system(handle_movement)
        .add_system(handle_gunfire)
        .add_system(handle_bullet_flight)
        .add_system(handle_bullet_collision.after(handle_bullet_flight))
        .add_system(calculate_damages.after(handle_bullet_collision))
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

