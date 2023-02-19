use crate::guns::{GunPersistentStats, GunPreset};
use crate::health::{Dying, Health, HitPoints};
use crate::physics::{
    try_get_components_from_entities, //CollisionLayer, KinematicsBundle, PopularCollisionShape,
};
use crate::teams::{Team, TeamNumber};
use bevy::math::Vec3;
use bevy::prelude::{
    Bundle, Commands, Component, Entity, EventReader, Query, Res, Sprite, SpriteBundle, Time,
    Transform, With,
};
use bevy::utils::default;

/// Collection of components making up a projectile entity.
#[derive(Bundle)]
pub struct BulletBundle {
    pub bullet: Bullet,
    pub team: Team,
    /*#[bundle]
    pub kinematics: KinematicsBundle,*/
    #[bundle]
    pub sprite_bundle: SpriteBundle,
}

impl BulletBundle {
    pub fn new(
        gun_type: GunPreset,
        team: TeamNumber,
        transform: Transform,
        velocity: Vec3,
    ) -> Self {
        let gun_stats = gun_type.stats();
        let bullet_transform = transform.with_scale(Vec3::ONE * gun_stats.projectile_size);
        Self {
            bullet: Bullet { gun_type },
            team: Team(team),
            /*kinematics: KinematicsBundle::new(
                PopularCollisionShape::Disc(gun_stats.projectile_size).get(Vec3::ONE),
                CollisionLayer::Projectile,
                &[CollisionLayer::Character, CollisionLayer::Obstacle],
            )
            .with_linear_velocity(velocity)
            .with_restitution(gun_stats.projectile_elasticity)
            .with_density(gun_stats.projectile_density),*/
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: gun_stats.projectile_color.0,
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
pub struct Bullet {
    gun_type: GunPreset,
}

/// Marker component for a rail gun projectile.
#[derive(Component)]
pub struct RailGunThing;

/// System to handle projectiles shot from a rail gun.
pub fn handle_railgun_things(
    time: Res<Time>,
    mut query_bullets: Query<(&Bullet, &mut Transform), With<RailGunThing>>,
) {
    for (bullet, mut transform) in query_bullets.iter_mut() {
        let up = transform.up();
        transform.translation +=
            up * bullet.gun_type.stats().projectile_speed * time.delta_seconds();
    }
}

/// Apply damage to a body affected by a projectile. If the remaining health happens to be below 0, marks it Dying.
fn do_projectile_damage(
    commands: &mut Commands,
    projectile: (&GunPersistentStats, &Team),
    body: (Entity, &mut Health, Option<&Team>),
    damage_substitute: Option<HitPoints>,
) {
    if body.1.is_dead() {
        // uncouth, but since we still don't have healing, return to this later when panicking is solved
        return;
    }
    let mut should_be_damaged = true;
    if let Some(body_team) = body.2 {
        should_be_damaged = projectile.0.friendly_fire || projectile.1 != body_team;
    }
    if should_be_damaged
        && body
            .1
            .damage(damage_substitute.unwrap_or(projectile.0.projectile_damage))
    {
        // todo panics if an entity is already despawned. issues on bevy are still open.
        commands.entity(body.0).insert(Dying);
    }
}
/*
/// System to continually deal damage to bodies that rail gun slugs travel through.
pub fn handle_damage_from_railgun_things(
    mut commands: Commands,
    time: Res<Time>,
    query_bullets: Query<(&heron::Collisions, &Bullet, &Team), With<RailGunThing>>,
    mut query_bodies: Query<(&mut Health, Option<&Team>)>,
) {
    for (collisions, bullet, bullet_team) in query_bullets.iter() {
        let gun_stats = bullet.gun_type.stats();
        for body_entity in collisions.entities() {
            if let Ok(mut body) = query_bodies.get_mut(body_entity) {
                do_projectile_damage(
                    &mut commands,
                    (gun_stats, bullet_team),
                    (body_entity, &mut body.0, body.1),
                    Some(crate::guns::RAIL_GUN_DAMAGE_PER_SECOND * time.delta_seconds()),
                );
            }
        }
    }
}*/

/*
/// System to read collision events from bullets and apply their effects to the respective bodies.
/// In particular, damage.
pub fn handle_bullet_collision_events(
    mut commands: Commands,
    mut collision_events: EventReader<heron::CollisionEvent>,
    // todo change this function to only damage if projectile dampening is done, health would be a given
    mut query_bodies: Query<(Option<&mut Health>, Option<&Team>)>,
    query_bullets: Query<(&Bullet, &Team, &heron::Velocity)>,
) {
    for event in collision_events.iter() {
        let (entity_a, entity_b) = event.rigid_body_entities();
        if let Some((bullet_entity, body_entity)) =
            try_get_components_from_entities(&query_bullets, &query_bodies, entity_a, entity_b)
        {
            let (body_health, body_team) = query_bodies.get_mut(body_entity).unwrap();
            let (gun_stats, bullet_team, bullet_velocity) = query_bullets
                .get(bullet_entity)
                .map(|(bullet, team, velocity)| (bullet.gun_type.stats(), team, velocity))
                .unwrap();
            // todo deal damage proportionate to the momentum transferred, armor changes restitution of the body - deal less damage if a bullet is deflected
            // There'd be double damage if we don't pick a type of events.
            // Most bullets do not register collision Stopping immediately due to perfect inelasticity.
            if event.is_started() {
                if let Some(mut life) = body_health {
                    do_projectile_damage(
                        &mut commands,
                        (&gun_stats, bullet_team),
                        (body_entity, &mut life, body_team),
                        None,
                    );
                }
            }
            if gun_stats.is_projectile_busted(bullet_velocity.linear.length()) {
                commands.entity(bullet_entity).despawn();
            }
        }
    }
}*/
