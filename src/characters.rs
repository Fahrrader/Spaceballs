use crate::actions::CharacterActionInput;
use crate::health::{Health, HitPoints};
use crate::physics::{CollisionLayer, KinematicsBundle, PopularCollisionShape};
use crate::projectiles::{BulletBundle, BULLET_SIZE, BULLET_SPEED};
use crate::teams::{team_color, Team, TeamNumber};
use crate::Vec3;
use bevy::core::{Time, Timer};
use bevy::math::Vec2;
use bevy::prelude::{Bundle, Commands, Component, Query, Res, Sprite, SpriteBundle, Transform};
use bevy::utils::default;
use std::time::Duration;

pub const CHARACTER_SIZE: f32 = 50.0;

pub const CHARACTER_SPEED: f32 = 200.0;
pub const CHARACTER_RAD_SPEED: f32 = 5.0;

pub const CHARACTER_MAX_HEALTH: HitPoints = 100.0;
pub const CHARACTER_FIRE_COOLDOWN: Duration = Duration::from_millis(25);

pub const PLAYER_DEFAULT_TEAM: TeamNumber = 0;

#[derive(Bundle)]
pub struct BaseCharacterBundle {
    character: Character,
    health: Health,
    team: Team,
    action_input: CharacterActionInput,
    #[bundle]
    kinematics: KinematicsBundle,
    #[bundle]
    sprite_bundle: SpriteBundle,
}

impl BaseCharacterBundle {
    pub fn new(team: TeamNumber, transform: Transform) -> Self {
        Self {
            character: Character { ..default() },
            health: Health::new(CHARACTER_MAX_HEALTH),
            team: Team(team),
            action_input: CharacterActionInput::default(),
            kinematics: KinematicsBundle::new(
                PopularCollisionShape::get(
                    PopularCollisionShape::Cell(CHARACTER_SIZE),
                    transform.scale,
                ),
                CollisionLayer::Character,
                CollisionLayer::all(),
            ),
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: team_color(team),
                    custom_size: Some(Vec2::new(CHARACTER_SIZE, CHARACTER_SIZE)),
                    ..default()
                },
                transform,
                ..default()
            },
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
    pub fn new(team: TeamNumber, transform: Transform) -> Self {
        Self {
            character_bundle: BaseCharacterBundle::new(team, transform),
            player_controlled_marker: PlayerControlled,
        }
    }
}

#[derive(Component)]
pub struct Character {
    pub fire_cooldown: Timer,
}

#[derive(Component)]
pub struct PlayerControlled;

impl Default for Character {
    fn default() -> Self {
        Self {
            fire_cooldown: Timer::new(CHARACTER_FIRE_COOLDOWN, false),
        }
    }
}

impl Character {
    pub fn check_fire_cooldown(&self) -> bool {
        self.fire_cooldown.finished()
    }

    fn tick_fire_cooldown(&mut self, time_delta: Duration) -> bool {
        self.fire_cooldown.tick(time_delta).finished()
    }

    fn reset_fire_cooldown(&mut self) {
        self.fire_cooldown.reset();
    }
}

pub fn calculate_character_velocity(
    mut query: Query<(&mut heron::Velocity, &Transform, &CharacterActionInput)>,
) {
    for (mut velocity, transform, action_input) in query.iter_mut() {
        velocity.linear = transform.up() * action_input.speed() * CHARACTER_SPEED;
        velocity.angular =
            heron::AxisAngle::new(-Vec3::Z, action_input.angular_speed() * CHARACTER_RAD_SPEED);
    }
}

pub fn handle_gunfire(
    mut commands: Commands,
    time: Res<Time>,
    mut query_characters: Query<(&mut Character, &Team, &Transform, &CharacterActionInput)>,
) {
    for (mut character, team, character_transform, input) in query_characters.iter_mut() {
        if character.tick_fire_cooldown(time.delta()) && input.fire {
            let facing_direction = character_transform.up() * character_transform.scale;

            let character_movement_offset = input.speed() * CHARACTER_SPEED * time.delta_seconds();
            let size_offset = CHARACTER_SIZE / 1.4 + BULLET_SIZE;
            let bullet_spawn_offset = facing_direction * (size_offset + character_movement_offset);

            for _ in 0..1 {
                commands.spawn_bundle(BulletBundle::new(
                    team.0,
                    character_transform
                        .with_translation(character_transform.translation + bullet_spawn_offset)
                        .with_scale(Vec3::ONE),
                    facing_direction * BULLET_SPEED,
                ));
            }

            character.reset_fire_cooldown();
        }
    }
}
