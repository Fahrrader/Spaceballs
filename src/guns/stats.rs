use crate::characters::CHARACTER_SIZE;
use crate::guns::{Gun, GUN_TRANSPARENCY, GUN_VELOCITY_DAMPING_RATIO, GUN_Z_LAYER};
use crate::health::HitPoints;
use crate::physics::{CollisionLayer, KinematicsBundle, PopularCollisionShape};
use crate::projectiles::BulletBundle;
use crate::teams::Team;
use crate::GunPreset;
use bevy::math::{Quat, Vec3};
use bevy::prelude::{Color, GlobalTransform, Transform};
use rand::Rng;
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

pub enum ProjectileSpawnPoint {
    Gunpoint,
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
    // gun_sprite
    /// The gun color while it's unequipped.
    pub gun_neutral_color: Color,
    // pub adapts_to_player_color: bool,
    /// Standard offset (x-axis) of the gun's sprite's center from the character's center.
    pub gun_center_x: f32,
    /// Standard offset (y-axis) of the gun's sprite's center from the character's center.
    pub gun_center_y: f32,

    /// Time before the next shot after one is taken.
    pub fire_cooldown: Duration,
    /// Number of shots before the "magazine" is depleted, and the gun must be reloaded. Leave at 0 for no reload.
    pub shots_before_reload: u32, // todo
    /// Time in ms to reload a gun and set [`shots_before_reload`] to 0.
    pub reload_time: f32, // todo
    /// Units of distance the character is pushed back when firing.
    pub recoil: f32, // todo
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
    /// How bouncy the projectile is, where 0 is not bouncy at all, and 1 is perfect elasticity.
    pub projectile_elasticity: f32, // todo possibly replace with an 'extra component' for a physics layer

    /// Size of each projectile.
    pub projectile_size: f32,
    // projectile_sprite
    pub projectile_color: Color,
    /// Where the projectile spawns, where the gun barrel ends, or around a character centered on its center
    pub projectile_spawn_point: ProjectileSpawnPoint,

    /// Damage each projectile deals to the body it hits.
    pub projectile_damage: f32,
    /// Does the gun deal damage to the bodies it hits that share the team with the shooter?
    pub friendly_fire: bool,
    // special behavior with certain objects (termite should destroy walls, portals, rail gun doesn't doesn't collide but leaves collision events)
    // closure projectile_special_collision_behavior (collision layer hit)?
    // closure projectile_special_flight_behavior (sparks from flying tesla coils)?
    // ooh! get some extra components on bullets
    //pub projectile_extra_components: Vec<Box<dyn Component<Storage = bevy::ecs::component::TableStorage>>>,

    // todo collision layers, time_to_live? or min_velocity_to_live, have everything despawn when colliding with arena borders
}

impl GunPersistentStats {
    pub(crate) const fn regular() -> Self {
        Self {
            gun_width: REGULAR_GUN_WIDTH,
            gun_length: REGULAR_GUN_LENGTH,
            gun_neutral_color: Color::Rgba {
                red: 0.25,
                green: 0.25,
                blue: 0.25,
                alpha: GUN_TRANSPARENCY,
            }, // Color::DARK_GRAY
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
            // Projectiles will slide along. That means it will also not change its velocity until then.
            projectile_elasticity: 0.51,
            projectile_size: BULLET_SIZE,
            projectile_color: Color::ALICE_BLUE,
            projectile_spawn_point: ProjectileSpawnPoint::Gunpoint,
            projectile_damage: BULLET_DAMAGE,
            friendly_fire: false,
            //projectile_extra_components: Vec::new(),//vec![]
        }
    }

    /// Get the standard transform of a gun.
    pub fn get_transform(&self) -> Transform {
        // todo non-standard guns -- in the future.
        Transform::from_translation(Vec3::new(self.gun_center_x, self.gun_center_y, GUN_Z_LAYER))
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
        //.with_rigidbody_type(heron::RigidBody::KinematicVelocityBased)
    }

    /// Get a round of projectiles that comes out of a gun when a trigger is pressed.
    /// These still have to be spawned. The gun will change its state.
    // possibly return here additional components to place on the spawned bundle
    pub(crate) fn produce_projectiles(
        &self,
        gun_transform: &GlobalTransform,
        gun_type: &GunPreset,
        gun: &mut Gun,
        team: &Team,
    ) -> Vec<BulletBundle> {
        let (gun_scale, _, gun_translation) = gun_transform.to_scale_rotation_translation();
        let bullet_spawn_distance = self.get_bullet_spawn_offset(gun_scale);

        // "Perimeter" does not have a set spawn point, so it will have to have another pass later.
        let bullet_spawn_point = match self.projectile_spawn_point {
            ProjectileSpawnPoint::Gunpoint => {
                gun_translation + bullet_spawn_distance * gun_transform.up()
            }
            ProjectileSpawnPoint::Perimeter => {
                gun_translation - self.gun_center_y * gun_transform.up()
            }
        };

        let bullet_transform = gun_transform
            .compute_transform()
            .with_translation(bullet_spawn_point)
            .with_scale(Vec3::ONE);

        let mut bullets = vec![];

        for _ in 0..self.projectiles_per_shot {
            let facing_direction = self.get_spread_direction(gun) * gun_transform.up();

            let bullet_transform = match self.projectile_spawn_point {
                ProjectileSpawnPoint::Gunpoint => bullet_transform,
                ProjectileSpawnPoint::Perimeter => bullet_transform.with_translation(
                    bullet_transform.translation + bullet_spawn_distance * facing_direction,
                ),
            };

            bullets.push(BulletBundle::new(
                gun_type,
                team.0,
                bullet_transform,
                facing_direction * self.projectile_speed,
            ));
        }

        bullets
    }

    /// Calculate a possibly random vector of flight direction of a projectile.
    pub fn get_spread_direction(&self, gun: &mut Gun) -> Quat {
        if self.projectile_spread_angle == 0.0 {
            Quat::IDENTITY
        } else {
            Quat::from_axis_angle(
                -Vec3::Z,
                (gun.random_state.gen::<f32>() - 0.5) * self.projectile_spread_angle,
            )
        }
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
