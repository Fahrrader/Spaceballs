use crate::guns::colours::GunColour;
use crate::guns::{GUN_VELOCITY_DAMPING_RATIO, GUN_Z_LAYER};
use crate::physics::{CollisionLayer, KinematicsBundle, PopularCollisionShape};
use bevy::math::Vec3;
use bevy::prelude::Transform;
use std::time::Duration;

/// Enum listing the possibilities where the projectile should spawn when shot out of a gun.
pub enum ProjectileSpawnSpace {
    /// Should spawn at the tip of the gun barrel.
    Gunpoint,
    /// Should spawn around the character, centered on it.
    Perimeter,
}

// todo projectile trajectory dotted lines. So many projectile types, though...
// references: Brigador, PC billiard. Experiment!

/// Fixed variables per gun preset that are typically accessed via a look-up tree.
pub struct GunPersistentStats {
    /// Width (x-axis) of the gun's sprite.
    pub gun_width: f32,
    /// Length (y-axis) of the gun's sprite.
    pub gun_length: f32,
    /// The gun color while it's unequipped.
    pub gun_neutral_color: GunColour,
    /// Standard offset (x-axis) of the gun's sprite's center from the character's center.
    pub gun_center_x: f32,
    /// Standard offset (y-axis) of the gun's sprite's center from the character's center.
    pub gun_center_y: f32,

    /// Time before the next shot after one is taken.
    pub fire_cooldown: Duration,
    // windup? // should also be some indicator, some special effect
    /// Number of shots before the "magazine" is depleted, and the gun must be reloaded. Leave at 0 for no reload.
    pub shots_before_reload: u32,
    /// Time to reload a gun and set [`shots_before_reload`] back to full. // todo place a UI indicator
    pub reload_time: Duration,
    /// Units of distance the character is pushed back when firing.
    // why'd I even want this // refactor later, when dodge is implemented, use the same principle
    pub recoil: f32,
    // transparent? implant? have property that'd prevent gun dropping /

    // displace the following sections to ProjectilePreset if there's ever a gun that can shoot more than one type
    /// Number of projectiles that are fired each shot.
    pub projectiles_per_shot: u32,
    /// The total angle in radians the fired projectiles' directions are randomized in.
    pub projectile_spread_angle: f32,
    /// Speed of each projectile.
    pub projectile_speed: f32,
    /// Proportion of normal speed below which the projectile should disappear, her momentum now harmless.
    pub min_speed_to_live_multiplier: f32,
    // inherits_speed?
    /// How bouncy the projectile is, where 0 is not bouncy at all, and 1 is perfect elasticity.
    pub projectile_elasticity: f32,
    /// How heavy the projectile is, where 0 is massless (must be above), and 1 is the standard density.
    pub projectile_density: f32,
    /// Where the projectile spawns: where the gun barrel ends, or around a character centered on its center
    pub projectile_spawn_point: ProjectileSpawnSpace,

    /// Size of each projectile.
    pub projectile_size: f32,
    // projectile_sprite
    pub projectile_color: GunColour,

    /// Damage each projectile deals to the body it hits.
    pub projectile_damage: f32,
    /// Does the gun deal damage to the bodies it hits that share the team with the shooter?
    pub friendly_fire: bool,
}

impl GunPersistentStats {
    /// Get the standard transform of a gun.
    pub fn get_transform(&self) -> Transform {
        Transform::from_translation(Vec3::new(self.gun_center_x, self.gun_center_y, GUN_Z_LAYER))
    }

    /// Get the standard transform of a gun with some arbitrary scale applied.
    pub fn get_transform_with_scale(&self, scale: Vec3) -> Transform {
        let transform = self.get_transform();
        transform.with_scale(transform.scale * scale)
    }

    /// Get the standard physics components for a gun.
    pub fn get_kinematics(&self, scale: Vec3) -> KinematicsBundle {
        KinematicsBundle::new(
            PopularCollisionShape::get(
                PopularCollisionShape::RectangularCell(self.gun_width, self.gun_length),
                scale,
            ),
            CollisionLayer::Gear,
            &[CollisionLayer::Character, CollisionLayer::Obstacle],
        )
        .with_linear_damping(GUN_VELOCITY_DAMPING_RATIO)
        .with_angular_damping(GUN_VELOCITY_DAMPING_RATIO)
        .with_rigidbody_type(heron::RigidBody::Sensor)
    }

    /// Calculate the y-offset where the bullet spawns (usually, at the tip of the gun barrel).
    pub fn get_bullet_spawn_offset(&self, scale: Vec3) -> f32 {
        self.gun_length / 2.0 * scale.y + self.projectile_size / 2.0
    }

    /// Indicate whether the projectile has reached its threshold for being despawned,
    /// as it is too slow to live and do damage.
    pub fn is_projectile_busted(&self, projectile_speed: f32) -> bool {
        projectile_speed <= self.projectile_speed * self.min_speed_to_live_multiplier
    }
}
