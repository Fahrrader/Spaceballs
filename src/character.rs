use crate::actions::ActionInput;
use crate::collisions::Collider;
use crate::health::{Health, HitPoints};
use crate::projectiles::{BulletBundle, BULLET_SIZE, BULLET_SPEED};
use crate::team::{team_color, Team};
use bevy::core::{Time, Timer};
use bevy::math::Vec2;
use bevy::prelude::{
    Bundle, Commands, Component, Query, Res, Sprite, SpriteBundle, Transform, With,
};
use bevy::utils::default;
use std::time::Duration;

pub const CHARACTER_SIZE: f32 = 50.0;

pub const CHARACTER_SPEED: f32 = 200.0;
pub const CHARACTER_RAD_SPEED: f32 = 5.0;

pub const CHARACTER_MAX_HEALTH: HitPoints = 100.0;
pub const CHARACTER_FIRE_COOLDOWN: Duration = Duration::from_millis(25);

pub const PLAYER_DEFAULT_TEAM: Team = 0;

#[derive(Bundle)]
pub struct BaseCharacterBundle {
    character: Character,
    health: Health,
    action_input: ActionInput,
    #[bundle]
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

impl BaseCharacterBundle {
    pub fn new(team: Team, transform: Transform) -> Self {
        Self {
            character: Character { team, ..default() },
            health: Health::new(CHARACTER_MAX_HEALTH),
            action_input: ActionInput::default(),
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: team_color(team),
                    custom_size: Some(Vec2::new(CHARACTER_SIZE, CHARACTER_SIZE)),
                    ..default()
                },
                transform: transform.clone(),
                ..default()
            },
            collider: Collider,
        }
    }
}

#[derive(Bundle)]
pub struct ControlledPlayerCharacterBundle {
    #[bundle]
    character_bundle: BaseCharacterBundle,
    player_controlled_marker: PlayerControlled,
}

impl ControlledPlayerCharacterBundle {
    pub fn new(team: Team, transform: Transform) -> Self {
        Self {
            character_bundle: BaseCharacterBundle::new(team, transform),
            player_controlled_marker: PlayerControlled,
        }
    }
}

#[derive(Component)]
pub struct Character {
    pub team: Team,
    pub fire_cooldown: Timer,
}

#[derive(Component)]
pub struct PlayerControlled;

impl Default for Character {
    fn default() -> Self {
        Self {
            team: PLAYER_DEFAULT_TEAM,
            fire_cooldown: Timer::new(CHARACTER_FIRE_COOLDOWN, false),
        }
    }
}

impl Character {
    pub fn check_fire(&mut self, time_delta: Duration) -> bool {
        self.fire_cooldown.tick(time_delta).finished()
    }

    pub fn fire(&mut self) {
        self.fire_cooldown.reset();
    }
}

pub fn handle_gunfire(
    mut commands: Commands,
    time: Res<Time>,
    input: Res<ActionInput>,
    mut query_characters: Query<(&mut Character, &Transform), With<PlayerControlled>>,
) {
    for (mut character, character_transform) in query_characters.iter_mut() {
        if character.check_fire(time.delta()) && input.fire {
            commands.spawn_bundle(BulletBundle::new(
                character.team,
                character_transform.with_translation(
                    character_transform.translation
                        + character_transform.up()
                            * (CHARACTER_SIZE / 2.0
                                + BULLET_SIZE
                                + input.speed() * CHARACTER_SPEED * time.delta_seconds()),
                ),
                character_transform.up().truncate() * BULLET_SPEED,
            ));

            character.fire();
        }
    }
}
