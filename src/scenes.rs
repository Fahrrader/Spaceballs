use crate::characters::{AICharacterBundle, BuildCharacter, PlayerCharacterBundle};
use crate::multiplayer::{LocalPlayer, LocalPlayerHandle};
use crate::{
    EntropyGenerator, GunBundle, GunPreset, RectangularObstacleBundle, AI_DEFAULT_TEAM, CHUNK_SIZE,
    PLAYER_DEFAULT_TEAM, SCREEN_SPAN,
};
use bevy::math::{Quat, Vec3};
use bevy::prelude::{
    Camera, Commands, Entity, Query, Res, ResMut, Resource, Transform, Window, Without,
};
use std::f32::consts::PI;

/// Specifier of the scene which to load.
#[derive(clap::ValueEnum, Resource, Clone, Copy, Debug)]
pub enum SceneSelector {
    Experimental,
    Lite,
}

impl TryFrom<String> for SceneSelector {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "experimental" | "exp" | "e" => Ok(SceneSelector::Experimental),
            "lite" | "l" => Ok(SceneSelector::Lite),
            _ => Err("Nothing too bad, should use the default scene"),
        }
    }
}

/// System to spawn a scene, the choice of which is based on the scene specifier resource.
pub fn summon_scene(
    commands: Commands,
    scene: Option<Res<SceneSelector>>,
    random_state: ResMut<EntropyGenerator>,
    local_player_handle: Res<LocalPlayerHandle>,
) {
    match scene {
        None => setup_lite(commands, random_state),
        Some(scene) => match scene.into_inner() {
            SceneSelector::Experimental => {
                setup_experimental(commands, random_state, local_player_handle)
            }
            SceneSelector::Lite => setup_lite(commands, random_state),
        },
    }
}

/// Delete every entity! Only leave the cameras and windows.
///
/// Extremely unsafe, but I don't care at this point. Maybe return later.
pub fn despawn_everything(
    mut commands: Commands,
    query: Query<Entity, (Without<Camera>, Without<Window>)>,
) {
    query
        .iter()
        .for_each(|entity| commands.entity(entity).despawn());
}

/// Set up a more complicated and chaotic scene with the latest features and experiments.
pub fn setup_experimental(
    mut commands: Commands,
    mut random_state: ResMut<EntropyGenerator>,
    local_player_handle: Res<LocalPlayerHandle>,
) {
    setup_base_arena(&mut commands);

    // Some guns before the player
    commands.spawn(GunBundle::new(
        GunPreset::LaserGun,
        Some(Transform::from_translation(Vec3::new(-120.0, 50.0, 0.0))),
        random_state.fork(),
    ));
    commands.spawn(GunBundle::new(
        GunPreset::Imprecise,
        Some(Transform::from_translation(Vec3::new(-180.0, 50.0, 0.0))),
        random_state.fork(),
    ));
    commands.spawn(GunBundle::new(
        GunPreset::RailGun,
        Some(Transform::from_translation(Vec3::new(-240.0, 50.0, 0.0))),
        random_state.fork(),
    ));

    // Player character
    let player_0_entity = PlayerCharacterBundle::new(
        Transform::from_translation(Vec3::new(-150.0, 0.0, 0.0)),
        PLAYER_DEFAULT_TEAM,
        0,
    )
    .spawn_with_equipment(
        &mut commands,
        random_state.fork(),
        vec![GunPreset::Scattershot],
    )[0];

    // todo:mp player generation on drop-in
    // Player character 2
    let player_1_entity = PlayerCharacterBundle::new(
        Transform::from_translation(Vec3::new(-50.0, 150.0, 0.0)),
        PLAYER_DEFAULT_TEAM + 1,
        1,
    )
    .spawn_with_equipment(
        &mut commands,
        random_state.fork(),
        vec![GunPreset::Imprecise],
    )[0];

    // todo respawning? conjoin with drop-in
    // AI character
    AICharacterBundle::new(
        Transform::from_translation(Vec3::new(150.0, 0.0, 0.0))
            .with_rotation(Quat::from_axis_angle(Vec3::Z, PI / 6.0))
            .with_scale(Vec3::new(2.0, 3.0, 1.0)),
        AI_DEFAULT_TEAM,
        usize::MAX,
    )
    .spawn_with_equipment(&mut commands, random_state.fork(), vec![GunPreset::RailGun]);

    // Random wall in the middle
    commands.spawn(RectangularObstacleBundle::new(Transform::from_scale(
        Vec3::new(1.0, 2.0, 1.0),
    )));

    // Some minute trash - this system is going to get overhauled with repeated player spawn soon anyway.
    commands
        .entity(match local_player_handle.0 {
            0 => player_0_entity,
            _ => player_1_entity,
        })
        .insert(LocalPlayer);
}

/// Set up a lighter, stable scene. Considered default.
pub fn setup_lite(mut commands: Commands, mut random_state: ResMut<EntropyGenerator>) {
    setup_base_arena(&mut commands);

    let player_entity = PlayerCharacterBundle::new(Transform::default(), PLAYER_DEFAULT_TEAM, 0)
        .spawn_with_equipment(&mut commands, random_state.fork(), vec![GunPreset::Regular])[0];

    commands.entity(player_entity).insert(LocalPlayer);
}

/// Set up common stuff attributable to all levels.
fn setup_base_arena(commands: &mut Commands) {
    // ----- Walls of the arena
    commands.spawn(RectangularObstacleBundle::new(
        Transform::from_translation(Vec3::X * -SCREEN_SPAN / 2.0).with_scale(Vec3::new(
            0.5,
            SCREEN_SPAN / CHUNK_SIZE + 0.5,
            1.0,
        )),
    ));
    commands.spawn(RectangularObstacleBundle::new(
        Transform::from_translation(Vec3::X * SCREEN_SPAN / 2.0).with_scale(Vec3::new(
            0.5,
            SCREEN_SPAN / CHUNK_SIZE + 0.5,
            1.0,
        )),
    ));
    commands.spawn(RectangularObstacleBundle::new(
        Transform::from_translation(Vec3::Y * SCREEN_SPAN / 2.0).with_scale(Vec3::new(
            SCREEN_SPAN / CHUNK_SIZE + 0.5,
            0.5,
            1.0,
        )),
    ));
    commands.spawn(RectangularObstacleBundle::new(
        Transform::from_translation(Vec3::Y * -SCREEN_SPAN / 2.0).with_scale(Vec3::new(
            SCREEN_SPAN / CHUNK_SIZE + 0.5,
            0.5,
            1.0,
        )),
    ));
    // Walls of the arena -----
}
