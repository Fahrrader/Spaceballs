use crate::characters::{CHARACTER_MAX_HEALTH, CHARACTER_SIZE, CHARACTER_SPEED};
use crate::guns::colours::GunColour;
use crate::guns::stats::{GunPersistentStats, ProjectileSpawnSpace};
use crate::health::HitPoints;
use crate::physics::{ContinuousCollisionDetection, OngoingCollisions, Sensor};
use crate::projectiles::RailGunThing;
use crate::Color;
use bevy::ecs::system::EntityCommands;
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
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum GunPreset {
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

impl Default for GunPreset {
    fn default() -> Self {
        GunPreset::Regular
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
            GunPreset::RailGun => &RAIL_GUN,
            GunPreset::LaserGun => &LASER_GUN,
        }
    }

    /// Insert extra components into a projectile that should be there, determined by its preset.
    pub fn add_projectile_components(&self, projectile_commands: &mut EntityCommands) {
        /// Inserts components onto a projectile repeatedly.
        macro_rules! add_projectile_components {
            ($($e:expr),*) => {
                $(projectile_commands.insert($e));*
            }
        }

        match self {
            GunPreset::RailGun => {
                add_projectile_components!(
                    RailGunThing,
                    Sensor,
                    OngoingCollisions::default(),
                    ContinuousCollisionDetection { enabled: true }
                );
            }
            _ => {}
        };
    }

    /// Does a projectile need to have extra components inserted into it, according to its gun preset?
    // todo probably use a macro to generate the two functions together, if the number of guns becomes too many
    pub fn has_extra_projectile_components(&self) -> bool {
        match self {
            GunPreset::RailGun => true,
            _ => false,
        }
    }

    /// Stats of the base, default weapon, which others might base off.
    pub(crate) const fn regular() -> GunPersistentStats {
        GunPersistentStats {
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
            projectile_color: GunColour::new(Color::ALICE_BLUE),
            projectile_spawn_point: ProjectileSpawnSpace::Gunpoint,
            projectile_damage: BULLET_DAMAGE,
            friendly_fire: false,
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
    reload_time: Duration::from_millis(500),
    shots_before_reload: 15,
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
    projectile_spawn_point: ProjectileSpawnSpace::Perimeter,
    projectile_damage: BULLET_DAMAGE * 0.4,
    projectiles_per_shot: 64,
    projectile_elasticity: 1.0,
    fire_cooldown: Duration::from_millis(1800),
    ..GunPreset::regular()
};

/// Due to the rail gun's penetrative properties, damage is applied per second of travel inside a body.
/// Correlates heavily with projectile speed. Used by a special system.
pub const RAIL_GUN_DAMAGE_PER_SECOND: HitPoints = CHARACTER_MAX_HEALTH / 5.0 // under a normal angle and fully crossing the body, should kill in [5] hits
    * RAIL_GUN.projectile_speed
    / CHARACTER_SIZE;

/// Fast and furious. Penetrates foes, walls, and lusty Argonian maids like butter.
pub const RAIL_GUN: GunPersistentStats = GunPersistentStats {
    gun_neutral_color: GunColour::new(Color::SILVER),
    // Impact damage is nullified. See [`RAIL_GUN_DAMAGE_PER_SECOND`] for penetration damage.
    projectile_damage: 0.0,
    projectile_speed: CHARACTER_SIZE * 15.0,
    projectile_elasticity: 0.0,
    fire_cooldown: Duration::from_millis(1000),
    friendly_fire: true,
    recoil: 15.0,
    ..GunPreset::regular()
};

// todo have a point momentarily travel with an extra Component, bouncing off walls (have distinction in material?),
// making up a series of lines dealing damage on intersection. problem arises in calculating it every frame. also, add warm-up
/// Make a light show! Reflects off walls, your equivalent of a magic missile.
pub const LASER_GUN: GunPersistentStats = GunPersistentStats {
    gun_neutral_color: GunColour::new(Color::AQUAMARINE),
    projectile_color: GunColour::new(Color::LIME_GREEN),
    projectile_damage: BULLET_DAMAGE * 0.025,
    projectile_speed: BULLET_SPEED * 4.,
    projectile_elasticity: 1.0,
    projectile_density: 0.01,
    fire_cooldown: Duration::from_millis(5),
    friendly_fire: true,
    min_speed_to_live_multiplier: 0.3,
    ..GunPreset::regular()
};
