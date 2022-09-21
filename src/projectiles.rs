use crate::health::{Dying, Health};
use crate::physics::{
    try_get_components_from_entities, CollisionLayer, KinematicsBundle, PopularCollisionShape,
};
use crate::teams::{Team, TeamNumber};
use crate::{GunPreset, WINDOW_HEIGHT, WINDOW_WIDTH};
use bevy::math::Vec3;
use bevy::prelude::{
    Bundle, Commands, Component, Entity, EventReader, Query, Sprite, SpriteBundle, Transform, With,
};
use bevy::utils::default;

/// Collection of components making up a projectile entity.
#[derive(Bundle)]
pub struct BulletBundle {
    bullet: Bullet,
    gun_type: GunPreset,
    team: Team,
    #[bundle]
    kinematics: KinematicsBundle,
    #[bundle]
    sprite_bundle: SpriteBundle,
}

impl BulletBundle {
    pub fn new(
        gun_type: &GunPreset,
        team: TeamNumber,
        transform: Transform,
        velocity: Vec3,
    ) -> Self {
        let bullet_transform = transform.with_scale(Vec3::ONE * gun_type.stats().projectile_size);
        Self {
            bullet: Bullet,
            gun_type: gun_type.clone(),
            team: Team(team),
            kinematics: KinematicsBundle::new(
                PopularCollisionShape::get(
                    PopularCollisionShape::Disc(gun_type.stats().projectile_size),
                    Vec3::ONE,
                ),
                CollisionLayer::Projectile,
                &[
                    CollisionLayer::Character,
                    //CollisionLayer::Projectile, // todo remove (but leave on for showcase)
                    CollisionLayer::Obstacle,
                ],
            )
            .with_linear_velocity(velocity),
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: gun_type.stats().projectile_color,
                    ..default()
                },
                transform: bullet_transform,
                ..default()
            },
        }
    }
}

/// Marker component signifying that this is indeed a bullet / projectile.
#[derive(Component)]
pub struct Bullet;

/// System to read collision events and apply their effects to the respective bodies.
/// In particular, damage.
pub fn handle_bullet_collision_events(
    mut commands: Commands,
    mut collision_events: EventReader<heron::CollisionEvent>,
    mut query_bodies: Query<(Option<&mut Health>, Option<&Team>)>,
    query_bullets: Query<(&GunPreset, &Team), With<Bullet>>,
) {
    for event in collision_events.iter() {
        let (entity_a, entity_b) = event.rigid_body_entities();
        if let Some((bullet_entity, body_entity)) =
            try_get_components_from_entities(&query_bullets, &query_bodies, entity_a, entity_b)
        {
            let (gun_type, bullet_team) = query_bullets.get(bullet_entity).unwrap();
            let (body_health, body_team) = query_bodies.get_mut(body_entity).unwrap();
            // commands.entity(bullet_entity).despawn(); todo uncomment after display
            if let Some(body_team) = body_team {
                if gun_type.stats().friendly_fire || bullet_team != body_team {
                    if let Some(mut life) = body_health {
                        if life.damage(gun_type.stats().projectile_damage) {
                            commands.entity(body_entity).insert(Dying);
                        }
                    }
                }
            }
        }
    }
}

/// System to despawn entities (bullets, in particular) that get out of bounds.
/// Temporary fallback measurement, possibly, since normally it shouldn't happen.
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
