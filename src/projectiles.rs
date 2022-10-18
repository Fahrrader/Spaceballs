use crate::health::HitPoints;
use crate::physics::{CollisionLayer, KinematicsBundle, PopularCollisionShape};
use crate::teams::{Team, TeamNumber};
use crate::{WINDOW_HEIGHT, WINDOW_WIDTH};
use bevy::math::Vec3;
use bevy::prelude::{
    Bundle, Color, Commands, Component, Entity, Query, Sprite, SpriteBundle, Transform, With,
};
use bevy::utils::default;

pub const BULLET_SIZE: f32 = 5.0;
pub const BULLET_SPEED: f32 = 300.0;
pub const BULLET_DAMAGE: HitPoints = 5.0;

#[derive(Bundle)]
pub struct BulletBundle {
    bullet: Bullet,
    team: Team,
    #[bundle]
    kinematics: KinematicsBundle,
    #[bundle]
    sprite_bundle: SpriteBundle,
}

impl BulletBundle {
    pub fn new(team: TeamNumber, transform: Transform, velocity: Vec3) -> Self {
        let bullet_transform = transform.with_scale(Vec3::ONE * BULLET_SIZE);
        Self {
            bullet: Bullet,
            team: Team(team),
            kinematics: KinematicsBundle::new(
                PopularCollisionShape::get(PopularCollisionShape::Disc(BULLET_SIZE), Vec3::ONE),
                CollisionLayer::Projectile,
                &[
                    CollisionLayer::Character,
                    CollisionLayer::Projectile, // todo remove (but leave on for showcase)
                    CollisionLayer::Obstacle,
                ],
            )
            .with_linear_velocity(velocity),
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: Color::ALICE_BLUE,
                    ..default()
                },
                transform: bullet_transform,
                ..default()
            },
        }
    }
}

#[derive(Component)]
pub struct Bullet;

impl Bullet {
    // adjust for guns
    pub fn get_damage(&self) -> HitPoints {
        BULLET_DAMAGE
    }
}

// fallback if anyone gets out of the arena?
pub fn handle_bullets_out_of_bounds(
    mut commands: Commands,
    mut query_bullets: Query<(&Transform, Entity), With<Bullet>>,
) {
    for (transform, entity) in query_bullets.iter_mut() {
        if transform.translation.x < WINDOW_WIDTH * -0.5
            || transform.translation.x > WINDOW_WIDTH * 0.5
            || transform.translation.y < WINDOW_HEIGHT * -0.5
            || transform.translation.y > WINDOW_HEIGHT * 0.5
        {
            bevy::log::warn!("An entity {} got out of bounds!", entity.id());
            commands.entity(entity).despawn();
        }
    }
}
