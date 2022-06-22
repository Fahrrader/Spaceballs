use crate::actions::CharacterActionInput;
use crate::characters::{Character, CHARACTER_SIZE};
use crate::physics::{CollisionLayer, KinematicsBundle, PopularCollisionShape};
use crate::projectiles::{BulletBundle, BULLET_SIZE, BULLET_SPEED};
use crate::teams::{team_color, Team, TeamNumber};
use bevy::core::{Time, Timer};
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{
    Bundle, Color, Commands, Component, Entity, GlobalTransform, Query, Res, Sprite, SpriteBundle,
    Transform, With, Without,
};
use bevy::utils::default;
use std::time::Duration;

/// The gun is slightly transparent to let the players see the projectiles and whatnot underneath,
/// since the gun doesn't have a collider.
const GUN_TRANSPARENCY: f32 = 0.95;
/// The gun is slightly darker than the main color of the character body to be distinct.
const GUN_COLOR_MULTIPLIER: f32 = 0.75;
/// The gun color while it's unequipped.
const GUN_NEUTRAL_COLOR: Color = Color::Rgba {
    red: 0.25,
    green: 0.25,
    blue: 0.25,
    alpha: GUN_TRANSPARENCY,
}; // Color::DARK_GRAY

/// Will deprecate in favor of sprites/varying gun sizes.
const GUN_LENGTH: f32 = CHARACTER_SIZE * 1.25;
const GUN_WIDTH: f32 = CHARACTER_SIZE * 0.25;

// todo replace with enum unpacking
const GUN_FIRE_COOLDOWN: Duration = Duration::from_millis(25);

const GUN_CENTER_X: f32 = 0.0;
const GUN_CENTER_Y: f32 = CHARACTER_SIZE * -0.15 + GUN_LENGTH * 0.5;
const GUN_Z_LAYER: f32 = 5.0;

// rename to weapon? nah dude this is spaceballs
#[derive(Bundle)]
pub struct GunBundle {
    preset: GunPreset,
    gun: Gun,
    #[bundle]
    kinematics: KinematicsBundle,
    #[bundle]
    sprite_bundle: SpriteBundle,
}

impl Default for GunBundle {
    fn default() -> Self {
        let preset = GunPreset::default();
        let transform = preset.get_transform();
        Self {
            gun: Gun::default(),
            kinematics: Self::get_kinematics(transform.scale),
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: GUN_NEUTRAL_COLOR,
                    custom_size: Some(Vec2::new(GUN_WIDTH, GUN_LENGTH)),
                    ..default()
                },
                transform,
                ..default()
            },
            preset,
        }
    }
}

impl GunBundle {
    pub fn new(preset: GunPreset, transform: Option<Transform>) -> Self {
        let mut gun = Self::default();
        gun.preset = preset;
        // todo set new transform/sprite according to preset?
        if let Some(transform) = transform {
            gun.sprite_bundle.transform = transform;
        }
        gun
    }

    pub fn with_paint_job(mut self, team_number: TeamNumber) -> Self {
        paint_gun(&mut self.sprite_bundle.sprite, Some(team_number));
        self
    }

    pub fn get_kinematics(scale: Vec3) -> KinematicsBundle {
        KinematicsBundle::new(
            PopularCollisionShape::get(
                PopularCollisionShape::RectangularCell(GUN_WIDTH, GUN_LENGTH),
                scale,
            ),
            CollisionLayer::Gear,
            &[CollisionLayer::Character, CollisionLayer::Obstacle],
        ) //.with_rigidbody_type(heron::RigidBody::KinematicVelocityBased)
    }
}

/// Holder of all non-constant properties of a weapon.
#[derive(Component)]
pub struct Gun {
    fire_cooldown: Timer,
}

impl Default for Gun {
    fn default() -> Self {
        Self {
            fire_cooldown: Timer::new(GUN_FIRE_COOLDOWN, false),
        }
    }
}

impl Gun {
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

/// Array of guns for your taste and pleasure. All fixed variables per type are found via a look-up tree by a value of this enum.
#[derive(Component, Clone)]
pub enum GunPreset {
    Regular,
    Imprecise,
    RailGun,
    Scattershot,
    Typhoon,
    // EMPCannon, SmokeCannon, LaserGun, RocketLauncher, Termite, PortalGun, MechSword?,
    // AssemblyNanoSwarmLauncher, MinePlanter, TeslaCoilLauncher
}

impl Default for GunPreset {
    fn default() -> Self {
        GunPreset::Regular
    }
}

impl GunPreset {
    pub fn get_transform(&self) -> Transform {
        // match self {
        Transform::from_translation(Vec3::new(GUN_CENTER_X, GUN_CENTER_Y, GUN_Z_LAYER))
        // right- and left-handedness?
    }
}
// todo projectiles_per_shot, fire cooldown, spread, damage, recoil

// gun behavior for different aspects, have gun presets -- just functions to create a new gun bundle? does it have to be a bundle, components are not attached to anything else, though
// but gun bundle possesses extra vars that don't change, a waste of memory; just have an enum that'd match to a specific preset? looks like it's going to be a lookup tree, so no biggie

/*let good_distance = match character.firing_mode {
FiringMode::Regular | FiringMode::RailGun => Quat::IDENTITY,
FiringMode::Imprecise => {
Quat::from_axis_angle(-Vec3::Z, (rand::random::<f32>() - 0.5) * PI / 12.0)
}
FiringMode::Scattershot => {
Quat::from_axis_angle(-Vec3::Z, (rand::random::<f32>() - 0.5) * PI / 2.0)
}
FiringMode::Typhoon => {
Quat::from_axis_angle(-Vec3::Z, (rand::random::<f32>() - 0.5) * PI * 2.0)
}
} * character_transform.up();*/

pub(crate) fn reset_gun(preset: &GunPreset, transform: &mut Transform) {
    let preset_transform = preset.get_transform();
    transform.translation = preset_transform.translation;
    transform.rotation = preset_transform.rotation;
}

pub(crate) fn paint_gun(sprite: &mut Sprite, team_number: Option<TeamNumber>) {
    if let Some(team_number) = team_number {
        sprite.color = (team_color(team_number) * GUN_COLOR_MULTIPLIER)
            .set_a(GUN_TRANSPARENCY)
            .as_rgba();
    } else {
        sprite.color = GUN_NEUTRAL_COLOR;
    }
}

/// Marker signifying that the entity is equipped "by" another entity and is a child (transforms are shared).
#[derive(Component)]
pub struct Equipped {
    pub by: Entity,
}

pub fn handle_gunfire(
    mut commands: Commands,
    time: Res<Time>,
    mut query_weapons: Query<(&mut Gun, &GlobalTransform, &Equipped)>,
    query_characters: Query<(&CharacterActionInput, &Team), With<Character>>,
) {
    for (mut gun, gun_transform, equipped) in query_weapons.iter_mut() {
        let (is_firing, team) = query_characters
            .get(equipped.by)
            .map(|(input, team)| (input.fire, team))
            .unwrap();

        if gun.tick_fire_cooldown(time.delta()) && is_firing {
            let facing_direction = gun_transform.up();

            let barrel_offset = GUN_LENGTH / 2.0 * gun_transform.scale.y + BULLET_SIZE / 2.0;
            let bullet_spawn_offset = facing_direction * barrel_offset;
            // todo add a ray cast from the body to the gun barrel to check for collisions
            // but currently it's kinda like shooting from cover / over shoulder, fun

            for _ in 0..1 {
                commands.spawn_bundle(BulletBundle::new(
                    team.0,
                    gun_transform
                        .with_translation(gun_transform.translation + bullet_spawn_offset)
                        .with_scale(Vec3::ONE)
                        .into(),
                    facing_direction * BULLET_SPEED,
                ));
            }

            gun.reset_fire_cooldown();
        }
    }
}

pub fn handle_pulsation(
    time: Res<Time>,
    mut query_weapons: Query<(&mut Transform, &heron::Velocity), Without<Equipped>>,
) {
    for (mut transform, velocity) in query_weapons.iter() {
        //transform.scale = cosf32();
    }
}
