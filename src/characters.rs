use crate::actions::CharacterActionInput;
use crate::guns::Equipped;
use crate::health::{Health, HitPoints};
use crate::physics::{CollisionLayer, KinematicsBundle, PopularCollisionShape};
use crate::teams::{team_color, Team, TeamNumber};
use crate::Vec3;
use bevy::hierarchy::BuildChildren;
use bevy::math::Vec2;
use bevy::prelude::{Bundle, Commands, Component, Entity, Query, Sprite, SpriteBundle, Transform};
use bevy::utils::default;
use std::f32::consts::PI;

/// Standard size for a character body in the prime time of their life.
pub const CHARACTER_SIZE: f32 = 50.0;

/// Standard linear speed per second at full capacity in floating point units.
pub const CHARACTER_SPEED: f32 = 200.0;
/// Standard rotational speed at full capacity per second in radians.
pub const CHARACTER_RAD_SPEED: f32 = PI;

/// Standard maximum health for a player character.
pub const CHARACTER_MAX_HEALTH: HitPoints = 100.0;

/// The Character base all other Character bundles should use and add to.
#[derive(Bundle)]
pub struct BaseCharacterBundle {
    character: Character,
    health: Health,
    team: Team,
    pub(crate) action_input: CharacterActionInput,
    #[bundle]
    kinematics: KinematicsBundle,
    #[bundle]
    sprite_bundle: SpriteBundle,
}

impl BaseCharacterBundle {
    pub fn new(team: TeamNumber, transform: Transform) -> Self {
        Self {
            character: Character,
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

/// Bundle for a Player Character, controlled locally
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

/// [deprecated] Marker designating an entity serving as a character body and character.
#[derive(Component)]
pub struct Character;

/// Marker designating an entity controlled by the local player.
#[derive(Component)]
pub struct PlayerControlled;

// todo figure where to use the equipping weapons
// - picking up guns on the ground or from dead enemies
// - switching guns on the fly from a selection (With<Equipped>?)
// useless to have it in character
// check that the entities possess the char and gun elements? probably just whatever
pub(crate) fn equip_weapon(commands: &mut Commands, char_entity: Entity, weapon_entity: Entity) {
    //ensure!(); is a gun/equippable, doesn't have equipped -- though should panic, trying to insert an existing component
    // what else could be equippable,
    commands.entity(char_entity).add_child(weapon_entity);
    commands
        .entity(weapon_entity)
        .insert(Equipped { by: char_entity });
    // insert/replace team, equipped(char_entity.id())
    // set velocity to 0 and rotation to identity
    // ooh! have multiple guns on a body
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
