use crate::health::Health;
use crate::physics::{
    try_get_components_from_entities, CollisionLayer, KinematicsBundle, PopularCollisionShape,
};
use crate::teams::{Team, TeamNumber};
use crate::{EntityDamagedEvent, GunPreset, WINDOW_HEIGHT, WINDOW_WIDTH};
use bevy::math::Vec3;
use bevy::prelude::{
    Bundle, Color, Commands, Component, Entity, EventReader, EventWriter, Query, Sprite,
    SpriteBundle, Transform, With,
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

pub fn handle_bullet_collision_events(
    mut commands: Commands,
    mut collision_events: EventReader<heron::CollisionEvent>,
    query_bodies: Query<(&heron::CollisionShape, Option<&Health>, Option<&Team>)>,
    query_bullets: Query<(&GunPreset, &Team), With<Bullet>>,
    mut ew_damage: EventWriter<EntityDamagedEvent>,
) {
    for event in collision_events.iter() {
        let (entity_a, entity_b) = event.rigid_body_entities();
        if let Some((bullet_entity, body_entity)) =
            try_get_components_from_entities(&query_bullets, &query_bodies, entity_a, entity_b)
        {
            let (gun_type, bullet_team) = query_bullets.get(bullet_entity).unwrap();
            let (_, body_health, body_team) = query_bodies.get(body_entity).unwrap();
            // commands.entity(bullet_entity).despawn(); todo uncomment after display
            if let Some(body_team) = body_team {
                if gun_type.stats().friendly_fire || bullet_team != body_team {
                    if body_health.is_some() {
                        ew_damage.send(EntityDamagedEvent {
                            entity: body_entity,
                            damage: gun_type.stats().projectile_damage,
                        })
                    }
                }
            }
        }
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
