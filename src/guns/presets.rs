use crate::guns::stats::{
    GunPersistentStats, BULLET_DAMAGE, BULLET_SPEED, REGULAR_GUN_FIRE_COOLDOWN_TIME_MILLIS,
};
use crate::Color;
use bevy::prelude::Component;
use std::f32::consts::PI;
use std::time::Duration;

/// Array of guns for your taste and pleasure. All fixed variables per type are found via a look-up tree by a value of this enum.
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
    pub fn stats(&self) -> &GunPersistentStats {
        match self {
            GunPreset::Regular => &REGULAR,
            GunPreset::Imprecise => IMPRECISE,
            GunPreset::Scattershot => SCATTERSHOT,
            GunPreset::Typhoon => TYPHOON,
            GunPreset::RailGun => RAIL_GUN,
            GunPreset::LaserGun => LASER_GUN,
        }
    }
}

pub const REGULAR: GunPersistentStats = GunPersistentStats::regular();

pub const IMPRECISE: GunPersistentStats = GunPersistentStats {
    projectile_spread_angle: PI / 12.,
    projectile_damage: BULLET_DAMAGE * 0.6,
    projectile_speed: BULLET_SPEED * 2.,
    ..GunPersistentStats::regular()
};

pub const SCATTERSHOT: GunPersistentStats = GunPersistentStats {
    projectile_spread_angle: PI / 6.,
    projectile_damage: BULLET_DAMAGE * 0.2,
    projectiles_per_shot: 36,
    fire_cooldown: Duration::from_millis(600),
    ..GunPersistentStats::regular()
};

pub const TYPHOON: GunPersistentStats = GunPersistentStats {
    projectile_spread_angle: 2. * PI,
    projectile_damage: BULLET_DAMAGE * 0.4,
    projectiles_per_shot: 64,
    fire_cooldown: Duration::from_millis(1800),
    friendly_fire: true,
    ..GunPersistentStats::regular()
};

pub const RAIL_GUN: GunPersistentStats = GunPersistentStats {
    gun_neutral_color: Color::SILVER,
    projectile_damage: BULLET_DAMAGE * 3.,
    projectile_speed: BULLET_SPEED * 3.,
    projectiles_per_shot: 5,
    projectile_elasticity: 0.0,
    fire_cooldown: Duration::from_millis(1000),
    friendly_fire: true,
    ..GunPersistentStats::regular()
};

pub const LASER_GUN: GunPersistentStats = GunPersistentStats {
    gun_neutral_color: Color::AQUAMARINE,
    projectile_color: Color::LIME_GREEN,
    projectile_damage: BULLET_DAMAGE * 0.1,
    projectile_speed: BULLET_SPEED * 4.,
    projectile_elasticity: 1.0,
    fire_cooldown: Duration::from_millis(10),
    friendly_fire: true,
    ..GunPersistentStats::regular()
};
