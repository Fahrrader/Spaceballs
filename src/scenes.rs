use crate::characters::{AICharacterBundle, BuildCharacter, PlayerCharacterBundle};
use crate::network::session::{LocalPlayer, LocalPlayerHandle};
use crate::physics::{Chunks, ChunksAnchor};
use crate::{
    Color, EntropyGenerator, GunBundle, GunPreset, RectangularObstacleBundle, AI_DEFAULT_TEAM,
    PLAYER_DEFAULT_TEAM,
};
use bevy::math::{Quat, Vec3};
use bevy::prelude::{
    Camera, Commands, Entity, Query, Res, ResMut, Resource, Transform, Window, Without,
};
use std::f32::consts::PI;

/// Specifier of the scene which to load.
#[derive(clap::ValueEnum, Resource, Clone, Copy, Debug)]
pub enum SceneSelector {
    Main,
    Experimental,
}

impl TryFrom<String> for SceneSelector {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "m" | "main" | "master" => Ok(SceneSelector::Main),
            "experimental" | "exp" | "e" => Ok(SceneSelector::Experimental),
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
        None => setup_main(commands, random_state),
        Some(scene) => match scene.into_inner() {
            SceneSelector::Main => setup_main(commands, random_state),
            SceneSelector::Experimental => {
                setup_experimental(commands, random_state, local_player_handle)
            }
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
    commands.spawn(RectangularObstacleBundle::new_chunk(
        ChunksAnchor::Center,
        ChunksAnchor::Center,
        1.0,
        2.0,
    ));

    // Some minute trash - this system is going to get overhauled with repeated player spawn soon anyway.
    commands
        .entity(match local_player_handle.0 {
            0 => player_0_entity,
            _ => player_1_entity,
        })
        .insert(LocalPlayer);
}

/// Set up a lighter, stable scene. Considered default.
pub fn setup_main(mut commands: Commands, mut random_state: ResMut<EntropyGenerator>) {
    setup_base_arena(&mut commands);

    // TOP BLOCK
    commands.spawn(RectangularObstacleBundle::new_chunk(
        ChunksAnchor::Center,
        Chunks::Blocks(3.0),
        Chunks::Screen(0.55) - 2.,
        Chunks::Blocks(1.5),
    ));

    // LITTLE BLOCKS ENCLOSING THE TOP BLOCK
    commands.spawn(RectangularObstacleBundle::new_chunk(
        Chunks::Screen(-0.55 / 2.),
        1.0,
        1.,
        2.,
    ));
    commands.spawn(RectangularObstacleBundle::new_chunk(
        Chunks::Screen(0.55 / 2.).right(),
        1.0,
        // todo:physics allow negative scale to work like something from inside the usual dimension
        1., // -1.,
        2.,
    ));

    // ROTATED WEDGES SURROUNDING THE TOP BLOCK
    let wedge_len = (1.5f32.powi(2) + 1.).sqrt();
    commands.spawn(
        RectangularObstacleBundle::new_chunk(
            Chunks::Screen(-0.55 / 2.),
            Chunks::Blocks(3.0),
            1.5 / wedge_len,
            wedge_len,
        )
        .with_rotation(-(1. / wedge_len).asin())
        .with_color(Color::ORANGE_RED * 3.),
    );
    commands.spawn(
        RectangularObstacleBundle::new_chunk(
            Chunks::Screen(0.55 / 2.) - 1.,
            Chunks::Blocks(4.5),
            1.5 / wedge_len,
            wedge_len,
        )
        .with_rotation(PI + (1. / wedge_len).asin())
        .with_color(Color::ORANGE_RED * 3.),
    );

    // BOTTOM BLOCK
    let bottom_block_len: Chunks = Chunks::Screen(0.5) - 3.;
    commands.spawn(RectangularObstacleBundle::new_chunk(
        ChunksAnchor::Center,
        -Chunks::Screen(0.5) + 1.75,
        2.5,
        bottom_block_len,
    ));

    // SIDE BLOCKS
    commands.spawn(RectangularObstacleBundle::new_chunk(
        Chunks::Screen(-0.5),
        -2.,
        1.,
        3.,
    ));
    commands.spawn(RectangularObstacleBundle::new_chunk(
        Chunks::Screen(0.5).right(),
        -2.,
        1.,
        3.,
    ));

    let player_entity = PlayerCharacterBundle::new(Transform::default(), PLAYER_DEFAULT_TEAM, 0)
        .spawn_with_equipment(&mut commands, random_state.fork(), vec![GunPreset::Regular])[0];

    commands.entity(player_entity).insert(LocalPlayer);
}

/// Set up common stuff attributable to all levels.
fn setup_base_arena(commands: &mut Commands) {
    // ----- Walls of the arena
    commands.spawn(RectangularObstacleBundle::new_chunk(
        Chunks::Screen(-0.5) - 0.5 / 2.,
        Chunks::Screen(-0.5) - 0.5 / 2.,
        0.5,
        Chunks::Screen(1.0) + 0.5,
    ));
    commands.spawn(RectangularObstacleBundle::new_chunk(
        Chunks::Screen(-0.5) - 0.5 / 2.,
        Chunks::Screen(-0.5) - 0.5 / 2.,
        Chunks::Screen(1.0) + 0.5,
        0.5,
    ));
    commands.spawn(RectangularObstacleBundle::new_chunk(
        Chunks::Screen(0.5) - 0.5 / 2.,
        Chunks::Screen(-0.5) - 0.5 / 2.,
        0.5,
        // make it pretty with Chunks::Screen(-1.0) of negative scale once physics are overhauled
        Chunks::Screen(1.0) + 0.5,
    ));
    commands.spawn(RectangularObstacleBundle::new_chunk(
        Chunks::Screen(-0.5) - 0.5 / 2.,
        Chunks::Screen(0.5) - 0.5 / 2.,
        Chunks::Screen(1.0) + 0.5,
        0.5,
    ));
    // Walls of the arena -----
}
