use crate::characters::{AICharacterBundle, BuildCharacter, PlayerCharacterBundle};
use crate::network::session::{LocalPlayer, LocalPlayerHandle};
use crate::network::{PlayerHandle, PlayerRegistry, MAINTAINED_FPS_F64};
use crate::physics::{Chunks, ChunksAnchor};
use crate::{
    Color, EntropyGenerator, GunBundle, GunPreset, PlayerCount, RectangularObstacleBundle,
    ReflectResource, TimerMode, AI_DEFAULT_TEAM, PLAYER_DEFAULT_TEAM,
};
use bevy::math::{Quat, Vec3};
use bevy::prelude::{
    default, Bundle, Camera, Commands, Component, Entity, EventReader, FromReflect, Query, Reflect,
    Res, ResMut, Resource, Timer, Transform, Window, Without,
};
use bevy::reflect::ReflectFromReflect;
use std::collections::VecDeque;
use std::f32::consts::PI;
use std::time::Duration;

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

pub const RESPAWN_TIMEOUT: Duration = Duration::from_millis(3500);

#[derive(Component, Debug, Reflect, FromReflect)]
pub struct SpawnPoint {
    pub occupant_handle: Option<PlayerHandle>,
    pub timeout: Timer,
}

impl Default for SpawnPoint {
    fn default() -> Self {
        let mut new = Self {
            occupant_handle: None,
            timeout: Timer::new(RESPAWN_TIMEOUT, TimerMode::Once),
        };
        new.timeout.pause();
        new
    }
}

impl SpawnPoint {
    pub fn is_free(&self) -> bool {
        self.occupant_handle.is_none()
    }

    pub fn occupy(&mut self, occupant_handle: PlayerHandle) {
        self.occupant_handle = Some(occupant_handle);
        self.timeout.reset();
        self.timeout.unpause();
    }

    pub fn free(&mut self) {
        self.occupant_handle = None;
        self.timeout.pause();
    }

    pub fn tick(&mut self, time_delta: Duration) -> bool {
        self.timeout.tick(time_delta).finished()
    }

    pub fn skip_timeout(&mut self) {
        self.timeout.tick(self.timeout.duration());
    }
}

use crate::network::players::{PlayerDied, PlayerJoined};
use bevy::prelude::{Sprite, SpriteBundle, Vec2};
use rand::prelude::SliceRandom;

#[derive(Bundle, Default)]
pub struct SpawnPointBundle {
    // pub transform: Transform,
    pub respawn_point: SpawnPoint,
    #[bundle]
    pub sprite_bundle: SpriteBundle,
}

impl SpawnPointBundle {
    pub fn new(transform: Transform) -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                transform,
                sprite: Sprite {
                    #[cfg(feature = "diagnostic")]
                    color: Color::PINK * 3.,
                    #[cfg(not(feature = "diagnostic"))]
                    color: Color::NONE,
                    custom_size: Some(Vec2::new(10., 30.)),
                    ..default()
                },
                ..default()
            },
            ..default()
        }
    }

    pub fn new_at(x: f32, y: f32) -> Self {
        Self::new(Transform::from_translation(Vec3::new(x, y, 0.0)))
    }

    pub fn with_rotation(mut self, angle: f32) -> Self {
        self.sprite_bundle.transform.rotation = Quat::from_rotation_z(angle);
        self
    }
}

/// System to spawn a scene, the choice of which is based on the scene specifier resource.
pub fn summon_scene(
    commands: Commands,
    scene: Option<Res<SceneSelector>>,
    random_state: ResMut<EntropyGenerator>,
    player_count: Res<PlayerCount>,
) {
    match scene {
        None => setup_main(commands),
        Some(scene) => match scene.into_inner() {
            SceneSelector::Main => setup_main(commands),
            SceneSelector::Experimental => setup_experimental(commands, random_state, player_count),
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
    player_count: Res<PlayerCount>,
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

    // Non-existent player character 2, whose death will cause a crash *shrug*
    if player_count.0 == 1 {
        PlayerCharacterBundle::new(
            Transform::from_translation(Vec3::new(-50.0, 150.0, 0.0)),
            PLAYER_DEFAULT_TEAM + 1,
            1,
        )
        .spawn_with_equipment(
            &mut commands,
            random_state.fork(),
            vec![GunPreset::Imprecise],
        )[0];
    }

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

    // Some spawn points, for your pleasure <3
    commands.spawn(SpawnPointBundle::new_at(-50.0, 150.0));

    commands.spawn(SpawnPointBundle::new_at(-150.0, 0.0));
}

/// Set up a lighter, stable scene. Considered default.
pub fn setup_main(mut commands: Commands) {
    setup_base_arena(&mut commands);

    // TOP BLOCK
    commands.spawn(RectangularObstacleBundle::new_chunk(
        ChunksAnchor::Center,
        Chunks::Blocks(3.0),
        Chunks::Screen(0.55) - 2.,
        Chunks::Blocks(1.5),
    ));

    // LITTLE BLOCKS SURROUNDING THE TOP BLOCK
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
    let bottom_block_y_start: Chunks = -Chunks::Screen(0.5) + 1.75;
    let bottom_block_len: Chunks = Chunks::Screen(0.5) - 3.;
    commands.spawn(RectangularObstacleBundle::new_chunk(
        ChunksAnchor::Center,
        bottom_block_y_start,
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

    // bottom-left spawn point
    commands.spawn(
        SpawnPointBundle::new_at(
            (Chunks::Screen(-0.55 / 2.) - 1.).to_px(),
            (bottom_block_y_start + bottom_block_len / 2.).to_px(),
        )
        .with_rotation(-PI / 4.),
    );

    // bottom-right spawn point
    commands.spawn(
        SpawnPointBundle::new_at(
            (Chunks::Screen(0.55 / 2.) + 1.).to_px(),
            (bottom_block_y_start + bottom_block_len / 2.).to_px(),
        )
        .with_rotation(PI / 4.),
    );

    // top-left spawn point
    commands.spawn(
        SpawnPointBundle::new_at(
            (Chunks::Screen(-0.55 / 2.) - 1.).to_px(),
            Chunks::Screen(0.33).to_px(),
        )
        .with_rotation(-PI * 3. / 4.),
    );

    // top-right spawn point
    commands.spawn(
        SpawnPointBundle::new_at(
            (Chunks::Screen(0.55 / 2.) + 1.).to_px(),
            Chunks::Screen(0.33).to_px(),
        )
        .with_rotation(PI * 3. / 4.),
    );
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

#[derive(Resource, Debug, Default, Clone, Reflect, FromReflect)]
#[reflect_value(Debug, Resource, FromReflect)]
// bool to skip timeout or not to skip timeout
pub struct SpawnQueue(VecDeque<(PlayerHandle, bool)>);

pub fn handle_respawn_point_occupation(
    mut new_player_events: EventReader<PlayerJoined>,
    mut dead_player_events: EventReader<PlayerDied>,
    mut spawn_point_query: Query<(&mut SpawnPoint, /* temporary */ &mut Sprite)>,
    mut random_state: ResMut<EntropyGenerator>,
    mut spawn_queue: ResMut<SpawnQueue>,
) {
    let mut queue_player_for_respawn_if_not_queued =
        |player_handle: PlayerHandle, respawn_immediately: bool| {
            // inefficient! but spawn queue is rarely > 0, so not critical
            if !spawn_queue.0.iter().any(|&(h, _)| h == player_handle)
                && !spawn_point_query
                    .iter()
                    .any(|(point, _)| point.occupant_handle == Some(player_handle))
            {
                spawn_queue
                    .0
                    .push_back((player_handle, respawn_immediately));
            }
        };

    new_player_events.iter().for_each(|event| {
        queue_player_for_respawn_if_not_queued(event.player_handle, true);
    });
    dead_player_events.iter().for_each(|event| {
        queue_player_for_respawn_if_not_queued(event.player_handle, false);
    });

    if spawn_queue.0.is_empty() {
        return;
    }

    let mut spawn_point_vec: Vec<_> = spawn_point_query.iter_mut().collect();
    spawn_point_vec.shuffle(&mut random_state.0);

    for (ref mut spawn_point, ref mut sprite) in spawn_point_vec.iter_mut() {
        if !spawn_point.is_free() {
            continue;
        }

        if let Some(player_to_spawn) = spawn_queue.0.pop_front() {
            spawn_point.occupy(player_to_spawn.0);
            if player_to_spawn.1 {
                spawn_point.skip_timeout();
            }
            sprite.color = Color::GOLD * 3.;
        } else {
            return;
        }
    }
}

pub fn handle_player_respawning(
    mut commands: Commands,
    mut spawn_point_query: Query<(
        &mut SpawnPoint,
        &Transform,
        /* temporary */ &mut Sprite,
    )>,
    player_registry: Res<PlayerRegistry>,
    local_player: Res<LocalPlayerHandle>,
    mut random_state: ResMut<EntropyGenerator>,
) {
    for (mut spawn_point, transform, mut sprite) in spawn_point_query.iter_mut() {
        if spawn_point.is_free()
            || !spawn_point.tick(Duration::from_secs_f64(1. / MAINTAINED_FPS_F64))
        {
            continue;
        }

        // todo handle the experimental case where the player is not registered, but whose husk is present
        // just don't respawn then? then, if a player ends up joining, either react to an event, or ... well, it's not the case yet.
        let player_handle = spawn_point
            .occupant_handle
            .expect("Spawn beacon is occupied, but occupant handle is `None`? Preposterous!");
        let player_entity = AICharacterBundle::new(
            *transform,
            *player_registry.get(player_handle).expect("Spawn beacon is occupied, but occupant handle is not registered as a player? Preposterous!").team,
            player_handle,
        )
        .spawn_with_equipment(
            &mut commands,
            random_state.fork(),
            vec![GunPreset::random(&mut random_state.0)],
        )[0];

        if player_handle == local_player.0 {
            // this is with assumption that if we're resurrecting the local player, no other must exist.
            commands.entity(player_entity).insert(LocalPlayer);
        }

        spawn_point.free();
        #[cfg(feature = "diagnostic")]
        {
            sprite.color = Color::TOMATO * 2.;
        }
        #[cfg(not(feature = "diagnostic"))]
        {
            sprite.color = Color::NONE;
        }
    }
}

pub fn reset_spawn_queue(mut spawn_queue: ResMut<SpawnQueue>) {
    spawn_queue.0.clear();
}
