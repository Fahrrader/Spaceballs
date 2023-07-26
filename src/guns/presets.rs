// todo make it a separate crate -- take common consts and types outside, too
use crate::characters::{CHARACTER_MAX_HEALTH, CHARACTER_SIZE, CHARACTER_SPEED};
use crate::guns::additives::*;
use crate::guns::stats::{GunColour, GunPersistentStats, ProjectileSpawnSpace};
use crate::health::HitPoints;
use crate::physics::{ContinuousCollisionDetection, OngoingCollisions, Sensor};
use crate::ui::colors;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::Color;
use bevy::reflect::{FromReflect, Reflect};
use rand::Rng;
use std::f32::consts::PI;
use std::time::Duration;

const REGULAR_GUN_LENGTH: f32 = CHARACTER_SIZE * 1.25;
const REGULAR_GUN_WIDTH: f32 = CHARACTER_SIZE * 0.25;

const REGULAR_GUN_CENTER_X: f32 = 0.0;
const REGULAR_GUN_CENTER_Y: f32 = CHARACTER_SIZE * -0.15 + REGULAR_GUN_LENGTH * 0.5;

pub const REGULAR_FIRE_COOLDOWN_TIME_MILLIS: u64 = 100;

pub const BULLET_SIZE: f32 = 5.0;
pub const BULLET_SPEED: f32 = CHARACTER_SPEED * 1.5; // 300.0
pub const BULLET_DAMAGE: HitPoints = CHARACTER_MAX_HEALTH / 20.0; // 5.0 - 20 direct hits
pub const BULLET_STOP_SPEED_MULTIPLIER: f32 = 0.8;

/// Array of guns for your taste and pleasure. All fixed variables per type are found via a look-up table by a value of this enum.
#[derive(Copy, Clone, Debug, Default, Hash, PartialEq, Eq, Reflect, FromReflect)]
pub enum GunPreset {
    #[default]
    Regular,
    Imprecise,
    RailGun,
    Scattershot,
    Typhoon,
    LaserGun,
    // EMPCannon, SmokeCannon, RocketLauncher, RemoteShrapnelLauncher, Termite, PortalGun, MechSword?, -MechScythe?
    // NanoSwarmLauncher, AssemblyNanoSwarmLauncher, MinePlanter, TeslaCoilLauncher, ArtilleryBattery,
    // Flammenwerfer, Vulkan, Boomerang, HookMineLauncher, TurretAssembler, ScorpionStinger, HackTaser,
    // IncendiaryBeam, WallRaiser, AcidTrailer, OneWayShield (better used for coop), BombardmentBeacon,
    // SonicBoomer (push enemies into traps!), TunnelDrillClaws (make tunnels in the second plane!), NitrogenSpewer (ice skates and flash freeze),
    // some melee attack always available, ram forward kinda like dodge?, parry to increase bullet speed?,
    // WMDs? Something that would sufficiently impact the game as to make a zone unlivable. Craters. But need penalties...

    // not really a gun, but why not -- TRAVEL THROUGH TIME?? (forward, like do some stuff in advance) - also, reverse entropy
}

// probably implement some sort of reflection later on, to read new guns from .xml or .json, and get rid of this macro
macro_rules! generate_extra_projectile_components_fns {
    ($($preset:path => [$($component:expr),* $(,)?]),* $(,)?) => {
        /// Insert extra components into a projectile that should be there, determined by its preset.
        pub fn add_projectile_components(&self, projectile_commands: &mut EntityCommands) {
            match self {
                $($preset => {
                    $(projectile_commands.insert($component);)*
                },)*
                _ => {}
            };
        }

        /// Does a projectile need to have extra components inserted into it, according to its gun preset?
        pub const fn has_extra_projectile_components(&self) -> bool {
            match self {
                $($preset => true,)*
                _ => false,
            }
        }
    }
}

impl GunPreset {
    /// Map of an enum of a weapon to its constant stats, hopefully converted to a look-up table on compilation.
    #[inline]
    pub const fn stats(&self) -> &'static GunPersistentStats {
        match self {
            GunPreset::Regular => &REGULAR,
            GunPreset::Imprecise => &IMPRECISE,
            GunPreset::Scattershot => &SCATTERSHOT,
            GunPreset::Typhoon => &TYPHOON,
            GunPreset::RailGun => &RAILGUN,
            GunPreset::LaserGun => &LASER_GUN,
        }
    }

    pub fn random<R: Rng + ?Sized>(rng: &mut R) -> GunPreset {
        match rng.gen_range(0..5) {
            0 => GunPreset::Imprecise,
            1 => GunPreset::RailGun,
            2 => GunPreset::Scattershot,
            3 => GunPreset::Typhoon,
            4 => GunPreset::LaserGun,
            // 0 .. 5 would never reach Regular; we don't serve default in this establishment
            _ => GunPreset::Regular,
        }
    }

    generate_extra_projectile_components_fns!(
        GunPreset::RailGun => [
            railgun::RailGunThing::default(),
            Sensor,
            OngoingCollisions::default(),
            ContinuousCollisionDetection { enabled: true },
        ],
    );
}

/// Regular, default gun. Shoots straight. Trusty and simple.
pub const REGULAR: GunPersistentStats = GunPersistentStats {
    name: "Regular",
    gun_width: REGULAR_GUN_WIDTH,
    gun_length: REGULAR_GUN_LENGTH,
    gun_neutral_color: GunColour::new(Color::DARK_GRAY),
    gun_center_x: REGULAR_GUN_CENTER_X,
    gun_center_y: REGULAR_GUN_CENTER_Y,
    fire_cooldown: Duration::from_millis(REGULAR_FIRE_COOLDOWN_TIME_MILLIS),
    shots_before_reload: 0,
    reload_time: Duration::from_millis(0),
    recoil: 0.0,
    projectiles_per_shot: 1,
    projectile_spread_angle: 0.0,
    projectile_speed: BULLET_SPEED,
    min_speed_to_live_multiplier: BULLET_STOP_SPEED_MULTIPLIER,
    // Elasticity of 0 or below will not trigger collision's Stopped events until another collision!
    projectile_elasticity: 0.10,
    projectile_density: 1.0,
    projectile_size: BULLET_SIZE,
    projectile_color: GunColour::new(Color::rgb(0.8 * 4., 0.2 * 4., 0.2 * 4.)),
    projectile_spawn_point: ProjectileSpawnSpace::Gunpoint,
    projectile_damage: BULLET_DAMAGE,
    friendly_fire: false,
};

/// An experimental "upgrade" over a regular gun. Faster, inaccurate, doesn't hit as hard.
pub const IMPRECISE: GunPersistentStats = GunPersistentStats {
    name: "Irregular",
    gun_neutral_color: GunColour::new(colors::DARK_CHESTNUT),
    projectile_spread_angle: PI / 12.,
    projectile_damage: BULLET_DAMAGE * 1.1,
    projectile_speed: BULLET_SPEED * 2.,
    reload_time: Duration::from_millis(500),
    shots_before_reload: 15,
    ..REGULAR
};

/// Shotgun. Individual pellets don't hit as hard and spread apart with time, but devastating at close range.
pub const SCATTERSHOT: GunPersistentStats = GunPersistentStats {
    name: "Scattergun",
    gun_neutral_color: GunColour::new(colors::BRASS),
    projectile_color: GunColour::new(Color::rgb(0.8 * 4., 0.5 * 4., 0.2 * 4.)),
    projectile_spread_angle: PI / 6.,
    projectile_damage: BULLET_DAMAGE * 0.85,
    projectiles_per_shot: 12,
    fire_cooldown: Duration::from_millis(600),
    recoil: 6.0,
    reload_time: Duration::from_millis(1200),
    shots_before_reload: 6,
    ..REGULAR
};

/// Discombobulate foes surrounding you with this. Spreads many projectiles in a circle.
pub const TYPHOON: GunPersistentStats = GunPersistentStats {
    name: "Typhoon",
    gun_neutral_color: GunColour::new(colors::CORAL),
    projectile_color: GunColour::new(Color::rgb(0.8 * 4., 0.2 * 4., 0.8 * 4.)),
    projectile_spread_angle: 2. * PI,
    projectile_spawn_point: ProjectileSpawnSpace::Perimeter,
    projectile_damage: BULLET_DAMAGE * 0.4,
    projectiles_per_shot: 64,
    projectile_elasticity: 1.0,
    fire_cooldown: Duration::from_millis(200),
    reload_time: Duration::from_millis(1600),
    shots_before_reload: 2,
    ..REGULAR
};

/// Fast and furious. Penetrates foes, walls, and lusty Argonian maids like butter.
pub const RAILGUN: GunPersistentStats = GunPersistentStats {
    name: "Railgun",
    gun_neutral_color: GunColour::new(Color::SILVER),
    projectile_color: GunColour::new(Color::rgb(0.2 * 4., 0.5 * 4., 0.8 * 4.)),
    // Impact damage is minimal. See [`railgun::PENETRATION_DAMAGE_PER_CHUNK`] for penetration damage.
    projectile_damage: 10.0,
    projectile_speed: CHARACTER_SIZE * 15.0,
    projectile_elasticity: 0.0,
    // Doesn't matter, since `shots_before_reload` = 1, reload will be triggered immediately.
    fire_cooldown: Duration::from_millis(100),
    reload_time: Duration::from_millis(1000),
    shots_before_reload: 1,
    friendly_fire: true,
    recoil: 15.0,
    ..REGULAR
};

// todo have a point momentarily travel with an extra Component, bouncing off walls (have distinction in material?),
// making up a series of lines dealing damage on intersection. problem arises in calculating it every frame. also, add warm-up
/// Make a light show! Reflects off walls, your equivalent of a magic missile.
pub const LASER_GUN: GunPersistentStats = GunPersistentStats {
    name: "Deadly Laser",
    gun_neutral_color: GunColour::new(Color::AQUAMARINE),
    projectile_color: GunColour::new(Color::rgb(0.2 * 5., 0.8 * 5., 0.2 * 5.)),
    projectile_damage: BULLET_DAMAGE * 0.025,
    projectile_speed: BULLET_SPEED * 4.,
    projectile_elasticity: 1.0,
    projectile_density: 0.01,
    fire_cooldown: Duration::from_millis(5),
    friendly_fire: true,
    min_speed_to_live_multiplier: 0.3,
    ..REGULAR
};
