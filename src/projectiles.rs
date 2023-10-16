use crate::guns::{GunPersistentStats, GunPreset};
use crate::health::{Dying, Health, HitPoints};
use crate::network::PlayerHandle;
use crate::physics::{
    popular_collider, try_get_components_from_entities, ActiveEvents, CollisionEvent,
    CollisionLayer, KinematicsBundle, Velocity,
};
use crate::teams::{Team, TeamNumber};
use bevy::math::Vec3;
use bevy::prelude::{
    Bundle, Commands, Component, Entity, EventReader, FromReflect, Query, Reflect, Sprite,
    SpriteBundle, Transform,
};
use bevy::utils::default;

/// Collection of components making up a projectile entity.
#[derive(Bundle)]
pub struct ProjectileBundle {
    pub projectile: Projectile,
    pub team: Team,
    #[bundle]
    pub kinematics: KinematicsBundle,
    pub active_events: ActiveEvents,
    #[bundle]
    pub sprite_bundle: SpriteBundle,
}

impl ProjectileBundle {
    pub fn new(
        gun_type: GunPreset,
        shooter_handle: Option<PlayerHandle>,
        team: TeamNumber,
        transform: Transform,
        velocity: Vec3,
    ) -> Self {
        let gun_stats = gun_type.stats();
        let bullet_transform = transform.with_scale(Vec3::ONE * gun_stats.projectile_size);
        Self {
            projectile: Projectile {
                gun_type,
                shooter_handle,
            },
            team: Team(team),
            kinematics: KinematicsBundle::new(
                // radius + bit of an oomph to the collider, no need to be so accurate
                popular_collider::disc(0.5 + 0.25),
                &[CollisionLayer::Projectile],
                &[CollisionLayer::Character, CollisionLayer::Obstacle],
            )
            .with_linear_velocity(velocity)
            .with_restitution(gun_stats.projectile_elasticity)
            .with_density(gun_stats.projectile_density),
            active_events: ActiveEvents::COLLISION_EVENTS,
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
#[derive(Component, Debug, Default, Reflect, FromReflect)]
pub struct Projectile {
    pub gun_type: GunPreset,
    pub shooter_handle: Option<PlayerHandle>,
}

impl Projectile {
    /// Apply damage to a body affected by a projectile. If the remaining health happens to be below 0, marks it Dying.
    pub fn do_damage(
        commands: &mut Commands,
        projectile: (&GunPersistentStats, Option<PlayerHandle>, &Team),
        body: (Entity, &mut Health, Option<&Team>),
        damage_substitute: Option<HitPoints>,
    ) {
        if body.1.is_dead() {
            // uncouth, but since we still don't have healing, return to this later when panicking is solved
            return;
        }
        let mut should_be_damaged = true;
        if let Some(body_team) = body.2 {
            should_be_damaged = projectile.0.friendly_fire || projectile.2 != body_team;
        }
        if should_be_damaged
            && body
                .1
                .damage(damage_substitute.unwrap_or(projectile.0.projectile_damage))
        {
            // todo panics if an entity is already despawned. issues on bevy are still open.
            commands.entity(body.0).insert(Dying {
                by_shooter: projectile.1,
            });
            bevy::log::error!("Damaged an idiot, he's dying");
        }
    }
}

/// System to read collision events from bullets and apply their effects to the respective bodies.
/// In particular, damage.
pub fn handle_bullet_collision_events(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    // todo change this function to only damage if projectile dampening is done, health would be a given
    mut query_bodies: Query<(Option<&mut Health>, Option<&Team>)>,
    query_bullets: Query<(&Projectile, &Team, &Velocity)>,
) {
    for event in collision_events.iter() {
        let (entity_a, entity_b) = match event {
            CollisionEvent::Started(a, b, _) => (*a, *b),
            CollisionEvent::Stopped(a, b, _) => (*a, *b),
        };
        if let Some((bullet_entity, body_entity)) =
            try_get_components_from_entities(&query_bullets, &query_bodies, entity_a, entity_b)
        {
            let (body_health, body_team) = query_bodies.get_mut(body_entity).unwrap();
            let (gun_stats, maybe_shooter, bullet_team, bullet_velocity) = query_bullets
                .get(bullet_entity)
                .map(|(bullet, team, velocity)| {
                    (
                        bullet.gun_type.stats(),
                        bullet.shooter_handle,
                        team,
                        velocity,
                    )
                })
                .unwrap();
            // todo deal damage proportionate to the momentum transferred, armor changes restitution of the body - deal less damage if a bullet is deflected
            // There'd be double damage if we don't pick a type of events.
            // Most bullets do not register collision Stopping immediately due to perfect inelasticity.
            if matches!(event, CollisionEvent::Started(..)) {
                if let Some(mut life) = body_health {
                    Projectile::do_damage(
                        &mut commands,
                        (&gun_stats, maybe_shooter, bullet_team),
                        (body_entity, &mut life, body_team),
                        None,
                    );
                }
            }
            if gun_stats.is_projectile_busted(bullet_velocity.linvel.length()) {
                commands.entity(bullet_entity).despawn();
            }
        }
    }
}
