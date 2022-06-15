use crate::health::Health;
use crate::projectiles::Bullet;
use crate::teams::Team;
use crate::{EntityDamagedEvent, Transform};
use bevy::ecs::query::WorldQuery;
use bevy::math::Vec3;
use bevy::prelude::{
    default, Bundle, Color, Commands, Entity, EventReader, EventWriter, Query, Sprite, SpriteBundle,
};
use heron::prelude::*;

pub const OBSTACLE_STEP_SIZE: f32 = 50.0;
pub const DEFAULT_OBSTACLE_COLOR: Color = Color::WHITE;

#[derive(Bundle)]
pub struct KinematicsBundle {
    pub rigidbody: RigidBody,
    pub velocity: Velocity,
    pub collider: CollisionShape,
    pub collision_layers: CollisionLayers,
}

impl KinematicsBundle {
    pub fn new(
        shape: CollisionShape,
        collision_group: CollisionLayer,
        collided_groups: &[CollisionLayer],
    ) -> Self {
        Self {
            collider: shape,
            collision_layers: CollisionLayers::none()
                .with_group(collision_group)
                .with_masks(collided_groups),
            ..default()
        }
    }

    pub fn with_rigidbody_type(mut self, rigidbody_type: RigidBody) -> Self {
        self.rigidbody = rigidbody_type;
        self
    }

    pub fn with_linear_velocity(mut self, velocity: Vec3) -> Self {
        self.velocity.linear = velocity;
        self
    }

    pub fn with_angular_velocity(mut self, rad_velocity: AxisAngle) -> Self {
        self.velocity.angular = rad_velocity;
        self
    }

    pub fn with_angular_velocity_from_angle(mut self, axis: Vec3, angle: f32) -> Self {
        self.velocity.angular = AxisAngle::new(axis, angle);
        self
    }
}

impl Default for KinematicsBundle {
    fn default() -> Self {
        Self {
            rigidbody: RigidBody::Dynamic,
            velocity: Velocity::default(),
            collider: CollisionShape::default(),
            collision_layers: CollisionLayers::none(),
        }
    }
}

#[derive(Bundle)]
pub struct RectangularObstacleBundle {
    rigidbody: RigidBody,
    collider: CollisionShape,
    collision_layers: CollisionLayers,
    #[bundle]
    sprite_bundle: SpriteBundle,
}

impl Default for RectangularObstacleBundle {
    fn default() -> Self {
        Self {
            rigidbody: RigidBody::Static,
            collider: CollisionShape::Cuboid {
                half_extends: Vec3::ONE,
                border_radius: None,
            },
            collision_layers: CollisionLayers::all_masks::<CollisionLayer>()
                .with_group(CollisionLayer::Obstacle)
                .without_mask(CollisionLayer::Obstacle),
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: DEFAULT_OBSTACLE_COLOR,
                    ..default()
                },
                ..default()
            },
        }
    }
}

impl RectangularObstacleBundle {
    pub fn new(transform: Transform) -> Self {
        Self {
            collider: PopularCollisionShape::get(
                PopularCollisionShape::Cell(OBSTACLE_STEP_SIZE),
                transform.scale,
            ),
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: DEFAULT_OBSTACLE_COLOR,
                    ..default()
                },
                transform: transform.with_scale(transform.scale * OBSTACLE_STEP_SIZE),
                ..default()
            },
            ..default()
        }
    }
}

#[derive(PhysicsLayer)]
pub enum CollisionLayer {
    Character,
    Projectile,
    Obstacle,
}

impl CollisionLayer {
    pub fn all() -> &'static [Self] {
        &[
            CollisionLayer::Character,
            CollisionLayer::Projectile,
            CollisionLayer::Obstacle,
        ]
    }
}

pub enum PopularCollisionShape {
    Cell(f32),
    Disc(f32),
}

impl PopularCollisionShape {
    pub fn get(shape: Self, scale: Vec3) -> CollisionShape {
        match shape {
            Self::Cell(size) => CollisionShape::Cuboid {
                half_extends: size / 2.0 * scale,
                border_radius: None,
            },
            Self::Disc(size) => CollisionShape::Sphere {
                radius: size / 2.0 * scale.length(),
            },
        }
    }
}

fn try_get_components_from_entities<'a, ComponentA: WorldQuery, ComponentB: WorldQuery>(
    query_a: &'a Query<ComponentA>,
    query_b: &'a Query<ComponentB>,
    entity_a: Entity,
    entity_b: Entity,
) -> Option<(
    <ComponentA::ReadOnlyFetch as bevy::ecs::query::Fetch<'a, 'a>>::Item,
    Entity,
    <ComponentB::ReadOnlyFetch as bevy::ecs::query::Fetch<'a, 'a>>::Item,
    Entity,
)> {
    return if let (Ok(component_a), Ok(component_b)) =
        (query_a.get(entity_a), query_b.get(entity_b))
    {
        Some((component_a, entity_a, component_b, entity_b))
    } else if let (Ok(component_a), Ok(component_b)) =
        (query_a.get(entity_b), query_b.get(entity_a))
    {
        Some((component_a, entity_b, component_b, entity_a))
    } else {
        None
    };
}

pub fn handle_bullet_collision_events(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    query_bodies: Query<(&CollisionShape, Option<&Health>, Option<&Team>)>,
    query_bullets: Query<(&Bullet, &Team)>,
    mut ew_damage: EventWriter<EntityDamagedEvent>,
) {
    for event in collision_events.iter() {
        let (entity_a, entity_b) = event.rigid_body_entities();
        // perhaps send damage to bullets as well to handle multiple types / buffs?
        if let Some((
            (bullet, bullet_team),
            bullet_entity,
            (_, body_health, body_team),
            body_entity,
        )) = try_get_components_from_entities::<
            (&Bullet, &Team),
            (&CollisionShape, Option<&Health>, Option<&Team>),
        >(&query_bullets, &query_bodies, entity_a, entity_b)
        {
            // commands.entity(bullet_entity).despawn(); todo uncomment after display
            if let Some(body_team) = body_team {
                if bullet_team != body_team {
                    if body_health.is_some() {
                        ew_damage.send(EntityDamagedEvent {
                            entity: body_entity,
                            damage: bullet.get_damage(),
                        })
                    }
                } else {
                    // friendly fire!
                }
            }
        }
    }
}
