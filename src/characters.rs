use crate::actions::CharacterActionInput;
use crate::guns::{paint_gun, reset_gun_transform, Equipped, GunPreset};
use crate::health::{Health, HitPoints};
use crate::physics::{
    try_get_components_from_entities, CollisionLayer, KinematicsBundle, PopularCollisionShape,
};
use crate::teams::{team_color, Team, TeamNumber};
use bevy::hierarchy::{BuildChildren, Children};
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{
    Bundle, Commands, Component, Entity, EventReader, GlobalTransform, Query, Sprite, SpriteBundle,
    Transform, With, Without,
};
use bevy::utils::default;
use heron::{AxisAngle, CollisionEvent, Velocity};
use std::f32::consts::PI;

/// Standard size for a character body in the prime time of their life.
pub const CHARACTER_SIZE: f32 = 50.0;

/// Standard linear speed per second at full capacity in floating point units.
pub const CHARACTER_SPEED: f32 = 200.0;
/// Standard rotational speed at full capacity per second in radians.
pub const CHARACTER_RAD_SPEED: f32 = PI;

/// The velocity of a gun when thrown. It is only a part of the calculation, the current character velocity is also taken into account.
const GUN_THROW_SPEED: f32 = CHARACTER_SPEED * 2.0;
/// Throw away the gun, it spins. It's a good trick.
const GUN_THROW_SPIN_SPEED: f32 = 4.0 * PI;
/// The damping ratio of the gun when thrown.
const GUN_THROW_DAMPING_RATIO: f32 = 1.15;

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
                    PopularCollisionShape::SquareCell(CHARACTER_SIZE),
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

/// Attach some equippable gear to a character and allow it to be interacted with.
/// Unchecked if actually equippable, or if the equipping entity is a character!
pub(crate) fn equip_gear(
    commands: &mut Commands,
    char_entity: Entity,
    gear_entity: Entity,
    gun_preset: &GunPreset,
    gear_properties: Option<&mut Transform>,
    gear_paint_job: Option<(&mut Sprite, Option<TeamNumber>)>,
) {
    commands.entity(char_entity).add_child(gear_entity);
    commands
        .entity(gear_entity)
        .remove_bundle::<KinematicsBundle>()
        .insert(Equipped { by: char_entity });

    // only guns for now
    if let Some(such) = gear_properties {
        reset_gun_transform(gun_preset, such);
    }
    if let Some(such) = gear_paint_job {
        paint_gun(gun_preset, such.0, such.1);
    }
    // ooh! have multiple guns on a body
}

/// Un-attach something equipped on some entity and give it physics.
/// No safety checks are made.
pub(crate) fn unequip_gear(
    commands: &mut Commands,
    gear_entity: Entity,
    kinematics: KinematicsBundle,
    gun_type: &GunPreset,
    gear_sprite: &mut Sprite,
    gear_transform: &mut Transform,
) {
    commands
        .entity(gear_entity)
        .remove::<Equipped>()
        .insert_bundle(kinematics);

    reset_gun_transform(gun_type, gear_transform);
    paint_gun(gun_type, gear_sprite, None);
}

/// Unequip gear and give it some speed according to its type.
/// No safety checks are made.
pub(crate) fn throw_away_gear(
    commands: &mut Commands,
    gear_entity: Entity,
    gear_linear_velocity: Vec3,
    gun_type: &GunPreset,
    gear_sprite: &mut Sprite,
    gear_transform: &mut Transform,
    char_transform: &Transform,
) {
    let kinematics = gun_type
        .stats()
        .get_kinematics(gear_transform.scale)
        .with_linear_velocity(gear_linear_velocity)
        .with_angular_velocity_in_rads(Vec3::Z, GUN_THROW_SPIN_SPEED)
        .with_linear_damping(GUN_THROW_DAMPING_RATIO)
        .with_angular_damping(GUN_THROW_DAMPING_RATIO);

    unequip_gear(
        commands,
        gear_entity,
        kinematics,
        gun_type,
        gear_sprite,
        gear_transform,
    );

    let offset_forward = char_transform.up() * char_transform.scale.y * CHARACTER_SIZE / 2.0;
    gear_transform.translation = char_transform.translation + offset_forward;
    gear_transform.rotation = char_transform.rotation;
}

/// System to convert a character's action input (human or not) to linear and angular velocities.
pub fn calculate_character_velocity(
    mut query: Query<(&mut Velocity, &Transform, &CharacterActionInput)>,
) {
    for (mut velocity, transform, action_input) in query.iter_mut() {
        velocity.linear = transform.up() * action_input.speed() * CHARACTER_SPEED;
        velocity.angular =
            AxisAngle::new(-Vec3::Z, action_input.angular_speed() * CHARACTER_RAD_SPEED);
    }
}

/// System to, according to a character's input, pick up and equip guns off the ground.
pub fn handle_gun_picking(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    query_characters: Query<(&CharacterActionInput, &Team)>,
    mut query_weapons: Query<(&GunPreset, &mut Sprite, &mut Transform)>,
) {
    for event in collision_events.iter() {
        let (entity_a, entity_b) = event.rigid_body_entities();
        if let Some((char_entity, weapon_entity)) =
            try_get_components_from_entities(&query_characters, &query_weapons, entity_a, entity_b)
        {
            let (char_input, char_team) = query_characters.get(char_entity).unwrap();

            if !char_input.use_environment_1 {
                continue;
            }

            let (weapon_preset, mut weapon_sprite, mut weapon_transform) =
                query_weapons.get_mut(weapon_entity).unwrap();

            equip_gear(
                &mut commands,
                char_entity,
                weapon_entity,
                weapon_preset,
                Some(&mut weapon_transform),
                Some((&mut weapon_sprite, Some(char_team.0))),
            );
        }
    }
}

/// System to, according to a character's input, unequip guns and throw them to the ground with some forward speed.
/// That perfect gun is gone, and the heat never bothered it anyway.
pub fn handle_letting_gear_go(
    mut commands: Commands,
    mut query_characters: Query<
        (
            &CharacterActionInput,
            &Velocity,
            &Transform,
            &mut Children,
            Entity,
        ),
        Without<Equipped>,
    >,
    mut query_gear: Query<
        (&GunPreset, &mut Sprite, &mut Transform, &GlobalTransform),
        With<Equipped>,
    >,
) {
    for (action_input, velocity, transform, children, entity) in query_characters.iter_mut() {
        if !action_input.use_environment_2 {
            continue;
        }
        let mut equipped_gears = Vec::<Entity>::new();
        for child in children.iter() {
            let child = *child;
            if let Ok((gun_type, mut gun_sprite, mut gun_transform, gun_g_transform)) =
                query_gear.get_mut(child)
            {
                equipped_gears.push(child);
                let gun_velocity = velocity.linear + gun_g_transform.up() * GUN_THROW_SPEED;
                throw_away_gear(
                    &mut commands,
                    child,
                    gun_velocity,
                    gun_type,
                    &mut gun_sprite,
                    &mut gun_transform,
                    &transform,
                );
            }
        }
        commands.entity(entity).remove_children(&equipped_gears);
    }
}

// todo dead men walking parsing (dying characters and other entities, through sparse-set components --> un-equip guns)
