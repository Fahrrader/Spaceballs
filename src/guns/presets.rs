use crate::characters::CHARACTER_SIZE;
use crate::guns::colours::GunColour;
use crate::guns::stats::{GunPersistentStats, ProjectileSpawnPoint};
use crate::health::HitPoints;
use crate::Color;
use bevy::prelude::Component;
use std::f32::consts::PI;
use std::time::Duration;

const REGULAR_GUN_LENGTH: f32 = CHARACTER_SIZE * 1.25;
const REGULAR_GUN_WIDTH: f32 = CHARACTER_SIZE * 0.25;

const REGULAR_GUN_CENTER_X: f32 = 0.0;
const REGULAR_GUN_CENTER_Y: f32 = CHARACTER_SIZE * -0.15 + REGULAR_GUN_LENGTH * 0.5;

pub const REGULAR_GUN_FIRE_COOLDOWN_TIME_MILLIS: u64 = 100;

pub const BULLET_SIZE: f32 = 5.0;
pub const BULLET_SPEED: f32 = 300.0;
pub const BULLET_DAMAGE: HitPoints = 5.0;
pub const BULLET_STOP_SPEED_MULTIPLIER: f32 = 0.67;

/// Array of guns for your taste and pleasure. All fixed variables per type are found via a look-up table by a value of this enum.
#[derive(Component, Clone)]
pub enum GunPreset {
    Regular,
    Imprecise,
    RailGun,
    Scattershot,
    Typhoon,
    LaserGun,
    // EMPCannon, SmokeCannon, RocketLauncher, RemoteShrapnelLauncher, Termite, PortalGun, MechSword?,
    // NanoSwarmLauncher, AssemblyNanoSwarmLauncher, MinePlanter, TeslaCoilLauncher, ArtilleryBattery,
}

impl Default for GunPreset {
    fn default() -> Self {
        GunPreset::Regular
    }
}

impl GunPreset {
    /// Look-up table, mapping an enum of a weapon to its constant stats.
    pub fn stats(&self) -> GunPersistentStats {
        match self {
            GunPreset::Regular => REGULAR,
            GunPreset::Imprecise => IMPRECISE,
            GunPreset::Scattershot => SCATTERSHOT,
            GunPreset::Typhoon => TYPHOON,
            GunPreset::RailGun => RAIL_GUN,
            GunPreset::LaserGun => LASER_GUN,
        }
    }

    pub(crate) const fn regular() -> GunPersistentStats {
        GunPersistentStats {
            gun_width: REGULAR_GUN_WIDTH,
            gun_length: REGULAR_GUN_LENGTH,
            gun_neutral_color: GunColour::new(Color::DARK_GRAY),
            gun_center_x: REGULAR_GUN_CENTER_X,
            gun_center_y: REGULAR_GUN_CENTER_Y,
            fire_cooldown: Duration::from_millis(REGULAR_GUN_FIRE_COOLDOWN_TIME_MILLIS),
            shots_before_reload: 0,
            reload_time: 0.0,
            recoil: 0.0,
            projectiles_per_shot: 1,
            projectile_spread_angle: 0.0,
            projectile_speed: BULLET_SPEED,
            min_speed_to_live_multiplier: BULLET_STOP_SPEED_MULTIPLIER,
            // Elasticity of .5 or below will not trigger collision's Stopped events until another collision!
            // Projectiles will just slide along. That means it will also not change its velocity until then.
            projectile_elasticity: 0.51,
            projectile_size: BULLET_SIZE,
            projectile_color: GunColour::new(Color::ALICE_BLUE),
            projectile_spawn_point: ProjectileSpawnPoint::Gunpoint,
            projectile_damage: BULLET_DAMAGE,
            friendly_fire: false,
            //projectile_extra_components: Vec::new(),//vec![]
        }
    }
}

/// Regular, default gun. Shoots straight. Trusty and simple.
pub const REGULAR: GunPersistentStats = GunPreset::regular();

/// An experimental "upgrade" over a regular gun. Faster, inaccurate, doesn't hit as hard.
pub const IMPRECISE: GunPersistentStats = GunPersistentStats {
    gun_neutral_color: GunColour::new(GunColour::DARK_CHESTNUT),
    projectile_spread_angle: PI / 12.,
    projectile_damage: BULLET_DAMAGE * 1.1,
    projectile_speed: BULLET_SPEED * 2.,
    ..GunPreset::regular()
};

/// Shotgun. Individual pellets don't hit as hard and spread apart with time, but devastating at close range.
pub const SCATTERSHOT: GunPersistentStats = GunPersistentStats {
    gun_neutral_color: GunColour::new(GunColour::BRASS),
    projectile_spread_angle: PI / 6.,
    projectile_damage: BULLET_DAMAGE * 0.85,
    projectiles_per_shot: 12,
    fire_cooldown: Duration::from_millis(600),
    recoil: 6.0,
    ..GunPreset::regular()
};

/// Discombobulate foes surrounding you with this. Spreads many projectiles in a circle.
pub const TYPHOON: GunPersistentStats = GunPersistentStats {
    gun_neutral_color: GunColour::new(GunColour::CORAL),
    projectile_spread_angle: 2. * PI,
    projectile_spawn_point: ProjectileSpawnPoint::Perimeter,
    projectile_damage: BULLET_DAMAGE * 0.4,
    projectiles_per_shot: 64,
    projectile_elasticity: 1.0,
    fire_cooldown: Duration::from_millis(1800),
    ..GunPreset::regular()
};

/// Fast and furious. Penetrates foes, walls, and lusty Argonian maids like butter.
pub const RAIL_GUN: GunPersistentStats = GunPersistentStats {
    gun_neutral_color: GunColour::new(Color::SILVER),
    projectile_damage: BULLET_DAMAGE * 3.,
    projectile_speed: BULLET_SPEED * 3.,
    projectiles_per_shot: 5,
    // forwarding message here: Elasticity of .5 or below will not trigger collision's Stopped events until another collision
    projectile_elasticity: 0.0,
    fire_cooldown: Duration::from_millis(1000),
    friendly_fire: true,
    recoil: 15.0,
    min_speed_to_live_multiplier: 0.33,
    ..GunPreset::regular()
};

/// Make a light show! Reflects off walls, your equivalent of a magic missile.
pub const LASER_GUN: GunPersistentStats = GunPersistentStats {
    gun_neutral_color: GunColour::new(Color::AQUAMARINE),
    projectile_color: GunColour::new(Color::LIME_GREEN),
    projectile_damage: BULLET_DAMAGE * 0.1,
    projectile_speed: BULLET_SPEED * 4.,
    projectile_elasticity: 1.0,
    fire_cooldown: Duration::from_millis(10),
    friendly_fire: true,
    min_speed_to_live_multiplier: 0.3,
    ..GunPreset::regular()
};
