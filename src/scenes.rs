use crate::characters::BuildCharacterBundle;
use crate::{
    BaseCharacterBundle, ControlledPlayerCharacterBundle, GunBundle, GunPreset,
    RectangularObstacleBundle, AI_DEFAULT_TEAM, OBSTACLE_CHUNK_SIZE, PLAYER_DEFAULT_TEAM,
    SCREEN_SPAN,
};
use bevy::math::{Quat, Vec3};
use bevy::prelude::{Camera2dBundle, Commands, Res, ResMut, Transform};
use rand::prelude::StdRng;
use rand::Rng;
use std::f32::consts::PI;

/// Specifier of the scene which to load.
#[derive(clap::ValueEnum, Clone)]
pub enum SceneArg {
    Experimental,
    Lite,
}

impl TryFrom<String> for SceneArg {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "experimental" | "exp" | "e" => Ok(SceneArg::Experimental),
            "lite" | "l" => Ok(SceneArg::Lite),
            _ => Err("Nothing too bad, should use the default scene"),
        }
    }
}

/// System to spawn a scene, the choice of which is based on the scene specifier resource.
pub fn summon_scene(
    commands: Commands,
    scene: Res<Option<SceneArg>>,
    random_state: ResMut<StdRng>,
) {
    match scene.into_inner() {
        None => setup_lite(commands, random_state),
        Some(scene) => match scene {
            SceneArg::Experimental => setup_experimental(commands, random_state),
            SceneArg::Lite => setup_lite(commands, random_state),
        },
    }
}

/// Set up a more complicated and chaotic scene with the latest features and experiments.
pub fn setup_experimental(mut commands: Commands, mut random_state: ResMut<StdRng>) {
    commands.spawn_bundle(Camera2dBundle::default());

    setup_base_arena(&mut commands);

    // Some guns before the player
    commands.spawn_bundle(GunBundle::new(
        GunPreset::LaserGun,
        Some(Transform::from_translation(Vec3::new(-120.0, 50.0, 0.0))),
        random_state.gen(),
    ));
    commands.spawn_bundle(GunBundle::new(
        GunPreset::Imprecise,
        Some(Transform::from_translation(Vec3::new(-180.0, 50.0, 0.0))),
        random_state.gen(),
    ));
    commands.spawn_bundle(GunBundle::new(
        GunPreset::RailGun,
        Some(Transform::from_translation(Vec3::new(-240.0, 50.0, 0.0))),
        random_state.gen(),
    ));

    // Player character
    ControlledPlayerCharacterBundle::new(
        PLAYER_DEFAULT_TEAM,
        Transform::from_translation(Vec3::new(-150.0, 0.0, 0.0)),
    )
    .spawn_with_equipment(
        &mut commands,
        &mut random_state,
        vec![GunPreset::Scattershot],
    );

    // AI character
    BaseCharacterBundle::new(
        AI_DEFAULT_TEAM,
        Transform::from_translation(Vec3::new(150.0, 0.0, 0.0))
            .with_rotation(Quat::from_axis_angle(Vec3::Z, PI / 6.0))
            .with_scale(Vec3::new(2.0, 3.0, 1.0)),
    )
    .spawn_with_equipment(&mut commands, &mut random_state, vec![GunPreset::RailGun]);

    // Random wall in the middle
    commands.spawn_bundle(RectangularObstacleBundle::new(Transform::from_scale(
        Vec3::new(1.0, 2.0, 1.0),
    )));
}

/// Set up a lighter, stable scene. Considered default.
pub fn setup_lite(mut commands: Commands, mut random_state: ResMut<StdRng>) {
    commands.spawn_bundle(Camera2dBundle::default());

    setup_base_arena(&mut commands);

    ControlledPlayerCharacterBundle::new(PLAYER_DEFAULT_TEAM, Transform::default())
        .spawn_with_equipment(&mut commands, &mut random_state, vec![GunPreset::Regular]);
}

/// Set up common stuff attributable to all levels.
fn setup_base_arena(commands: &mut Commands) {
    // ----- Walls of the arena
    commands.spawn_bundle(RectangularObstacleBundle::new(
        Transform::from_translation(Vec3::X * -SCREEN_SPAN / 2.0).with_scale(Vec3::new(
            1.0,
            SCREEN_SPAN / OBSTACLE_CHUNK_SIZE + 1.0,
            1.0,
        )),
    ));
    commands.spawn_bundle(RectangularObstacleBundle::new(
        Transform::from_translation(Vec3::X * SCREEN_SPAN / 2.0).with_scale(Vec3::new(
            1.0,
            SCREEN_SPAN / OBSTACLE_CHUNK_SIZE + 1.0,
            1.0,
        )),
    ));
    commands.spawn_bundle(RectangularObstacleBundle::new(
        Transform::from_translation(Vec3::Y * SCREEN_SPAN / 2.0).with_scale(Vec3::new(
            SCREEN_SPAN / OBSTACLE_CHUNK_SIZE + 1.0,
            1.0,
            1.0,
        )),
    ));
    commands.spawn_bundle(RectangularObstacleBundle::new(
        Transform::from_translation(Vec3::Y * -SCREEN_SPAN / 2.0).with_scale(Vec3::new(
            SCREEN_SPAN / OBSTACLE_CHUNK_SIZE + 1.0,
            1.0,
            1.0,
        )),
    ));
    // Walls of the arena -----
}
