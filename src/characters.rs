use crate::actions::CharacterActionInput;
use crate::guns::{paint_gun, reset_gun_transform, Equipped, Gun, GunPreset, Thrown};
use crate::health::{Health, HitPoints};
use crate::physics::{CollisionLayer, KinematicsBundle, PopularCollisionShape};
use crate::teams::{team_color, Team, TeamNumber};
use bevy::hierarchy::{BuildChildren, Children};
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{
    Bundle, Changed, Commands, Component, Entity, Query, Sprite, SpriteBundle, Transform, With,
    Without,
};
use bevy::utils::default;
use heron::{AxisAngle, Velocity};
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

/// Standard maximum health for a player character.
pub const CHARACTER_MAX_HEALTH: HitPoints = 100.0;

/// The Character base all other Character bundles should use and add to.
#[derive(Bundle)]
pub struct BaseCharacterBundle {
    pub character: Character,
    pub health: Health,
    pub team: Team,
    pub action_input: CharacterActionInput,
    #[bundle]
    pub kinematics: KinematicsBundle,
    #[bundle]
    pub sprite_bundle: SpriteBundle,
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
    pub character_bundle: BaseCharacterBundle,
    pub player_controlled_marker: PlayerControlled,
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
    gun_preset: GunPreset,
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
}

/// Un-attach something equipped on some entity and give it physics.
/// No safety checks are made.
pub(crate) fn unequip_gear(
    commands: &mut Commands,
    gear_entity: Entity,
    kinematics: KinematicsBundle,
    gun_type: GunPreset,
    gear_sprite: &mut Sprite,
) {
    commands
        .entity(gear_entity)
        .remove::<Equipped>()
        .insert_bundle(kinematics)
        .insert(Thrown);

    // reset_gun_transform(gun_type, gear_transform);
    paint_gun(gun_type, gear_sprite, None);
}

/// Unequip gear and give it some speed according to its type.
/// No safety checks are made.
pub(crate) fn throw_away_gear(
    commands: &mut Commands,
    gear_entity: Entity,
    gear_linear_velocity: Vec3,
    gun_type: GunPreset,
    gear_sprite: &mut Sprite,
    gear_transform: &mut Transform,
    char_transform: &Transform,
) {
    let kinematics = gun_type
        .stats()
        .get_kinematics(gear_transform.scale)
        .with_linear_velocity(gear_linear_velocity)
        .with_angular_velocity_in_rads(Vec3::Z, GUN_THROW_SPIN_SPEED)
        .with_rigidbody_type(heron::RigidBody::Dynamic);

    unequip_gear(commands, gear_entity, kinematics, gun_type, gear_sprite);

    let gear_offset_forward = char_transform.up() * char_transform.scale.y * CHARACTER_SIZE / 2.;
    *gear_transform = Transform::from_translation(
        char_transform.translation
            + gear_offset_forward
            + char_transform.rotation * gear_transform.translation,
    )
    .with_rotation(char_transform.rotation);
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
    query_characters: Query<(&CharacterActionInput, &Team, Entity)>,
    mut query_weapons: Query<
        (
            &Gun,
            &heron::Collisions,
            &mut Sprite,
            &mut Transform,
            Entity,
        ),
        With<heron::RigidBody>,
    >,
) {
    for (weapon, collisions, mut weapon_sprite, mut weapon_transform, weapon_entity) in
        query_weapons.iter_mut()
    {
        if collisions.is_empty() {
            continue;
        }
        for (char_input, char_team, char_entity) in query_characters.iter() {
            if !char_input.use_environment_1 || !collisions.contains(&char_entity) {
                continue;
            }

            let weapon_preset = weapon.preset;

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

/// System to, according to either to a character's input or its untimely demise, unequip guns and throw them to the ground with some gusto.
/// That perfect gun is gone, and the heat never bothered it anyway.
pub fn handle_letting_gear_go(
    mut commands: Commands,
    mut query_characters: Query<
        (
            &CharacterActionInput,
            &Velocity,
            &Transform,
            &mut Children,
            &Health,
            Entity,
        ),
        Without<Equipped>,
    >,
    mut query_gear: Query<(&Gun, &mut Sprite, &mut Transform), With<Equipped>>,
) {
    for (action_input, velocity, transform, children, health, entity) in query_characters.iter_mut()
    {
        // Only proceed with the throwing away if either the drop-gear button is pressed, or if the guy's wasted.
        if !(action_input.use_environment_2 || health.is_dead()) || children.is_empty() {
            continue;
        }

        let mut equipped_gears = Vec::<Entity>::new();
        for child in children.iter() {
            let child = *child;
            if let Ok((gun, mut gun_sprite, mut gun_transform)) = query_gear.get_mut(child) {
                let gun_type = gun.preset;
                equipped_gears.push(child);
                let gun_velocity = velocity.linear + transform.up() * GUN_THROW_SPEED;
                throw_away_gear(
                    &mut commands,
                    child,
                    gun_velocity,
                    gun_type,
                    &mut gun_sprite,
                    &mut gun_transform,
                    transform,
                );
            }
        }

        commands.entity(entity).remove_children(&equipped_gears);
    }
}

/// System to distribute guns around a character's face whenever a new one is added or an old one removed.
pub fn handle_inventory_layout_change(
    query_characters: Query<
        (&Transform, &Children),
        (With<CharacterActionInput>, Changed<Children>, Without<Gun>),
    >,
    mut query_gear: Query<(&Gun, &mut Transform), With<Equipped>>,
) {
    for (char_transform, children) in query_characters.iter() {
        let step_size = (CHARACTER_SIZE / (children.len() as f32 + 1.0)) * char_transform.scale.x;
        let far_left_x = -CHARACTER_SIZE * char_transform.scale.x / 2.0;
        for (i, child) in children.iter().enumerate() {
            if let Ok((gun, mut gun_transform)) = query_gear.get_mut(*child) {
                let original_transform = gun
                    .preset
                    .stats()
                    .get_transform_with_scale(char_transform.scale);

                gun_transform.translation.x =
                    original_transform.translation.x + far_left_x + step_size * (i + 1) as f32;
            }
        }
    }
}

// dead men walking parsing (dying characters and other entities, through sparse-set components --> do a variety of laying-to-rest activities to them prior to their passing)
