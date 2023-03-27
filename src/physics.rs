use crate::{error, SCREEN_SPAN};
use bevy::ecs::query::{ReadOnlyWorldQuery, WorldQuery};
use bevy::math::Vec3;
use bevy::prelude::{
    default, App, Bundle, Color, Commands, Component, CoreStage, DespawnRecursiveExt, Entity,
    EventReader, Plugin, Query, Reflect, RemovedComponents, Sprite, SpriteBundle, Transform, With,
};
use bevy::utils::HashSet;
use bevy_rapier2d::prelude::*;

pub use bevy_rapier2d::prelude::{
    ActiveEvents, Ccd as ContinuousCollisionDetection, CollisionEvent, CollisionGroups, RigidBody,
    Sensor, Velocity,
};

/// The size of a standard world cell chunk. Useful to keep about the same as a character's body size to configure the terrain easier.
pub const CHUNK_SIZE: f32 = 50.0;
pub const DEFAULT_OBSTACLE_COLOR: Color = Color::WHITE;

/// Collection of components desired for physics and collision simulation.
#[derive(Bundle, Default)]
pub struct KinematicsBundle {
    pub rigidbody: RigidBody,
    pub velocity: Velocity,
    pub damping: Damping,
    pub restitution: Restitution,
    // pub friction: Friction,
    pub mass_properties: ColliderMassProperties,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
}

impl KinematicsBundle {
    /// Create a new kinematic bundle.
    ///
    /// [collider] is a collider mesh, an invisible shape used to calculate collisions; it doesn't need to be the exact shape of the object.
    ///
    /// [collision_group] is a [`CollisionLayer`] the entity would belong to, i.e. a bullet would be a [`Projectile`](crate::CollisionLayer::Projectile).
    /// It makes up the mask of heron's [`CollisionLayers`] struct.
    ///
    /// [collided_groups] is an array of [`CollisionLayer`] enums the entity would collide with and fire events in such a case.
    /// It makes up the groups of heron's [`CollisionLayers`] struct.
    pub fn new(
        collider: Collider,
        collision_groups: &[CollisionLayer],
        collided_groups: &[CollisionLayer],
    ) -> Self {
        fn compute_group(layers: &[CollisionLayer]) -> Group {
            let group_bits = layers.iter().map(|&x| x as u32).sum();
            let maybe_group = Group::from_bits(group_bits);
            maybe_group.unwrap_or_else(|| {
                error!("Tried to create a collision group outside of normal bounds with bits = {:?}. Returning default group.", group_bits);
                Group::default()
            })
        }

        Self {
            collider,
            collision_groups: CollisionGroups::new(
                compute_group(collision_groups),
                compute_group(collided_groups),
            ),
            ..default()
        }
    }

    pub fn with_rigidbody_type(mut self, rigidbody_type: RigidBody) -> Self {
        self.rigidbody = rigidbody_type;
        self
    }

    pub fn with_linear_velocity(mut self, velocity: Vec3) -> Self {
        if velocity.z != 0. {
            panic!("Tried to assign a linear velocity vector with a meaningful Z-axis! Why?");
        }
        // no projection onto 2d, so buyer beware
        self.velocity.linvel = velocity.truncate();
        self
    }

    pub fn with_angular_velocity(mut self, rad_velocity: f32) -> Self {
        self.velocity.angvel = rad_velocity;
        self
    }

    pub fn with_linear_damping(mut self, damping: f32) -> Self {
        self.damping.linear_damping = damping;
        self
    }

    pub fn with_angular_damping(mut self, damping: f32) -> Self {
        self.damping.angular_damping = damping;
        self
    }

    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.restitution = Restitution::new(restitution);
        self
    }

    pub fn with_density(mut self, density: f32) -> Self {
        self.mass_properties = ColliderMassProperties::Density(density);
        self
    }
}

/// Standard rectangular obstacle, stopping characters and bullets alike.
/// Uses [`CHUNK_SIZE`] to determine its dimensions in addition to the provided scale.
#[derive(Bundle)]
pub struct RectangularObstacleBundle {
    pub rigidbody: RigidBody,
    pub collider: Collider,
    #[bundle]
    pub sprite_bundle: SpriteBundle,
}

impl Default for RectangularObstacleBundle {
    fn default() -> Self {
        Self {
            rigidbody: RigidBody::Fixed,
            collider: Collider::cuboid(1., 1.),
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
    /// Make a thing that stuff can't pass through. Warning: calculates its size based on the scale given and the normal obstacle size.
    pub fn new(transform: Transform) -> Self {
        Self {
            collider: popular_collider::square(1.0),
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: DEFAULT_OBSTACLE_COLOR,
                    ..default()
                },
                transform: transform.with_scale(transform.scale * CHUNK_SIZE),
                ..default()
            },
            ..default()
        }
    }
}

/// Collection of shortcuts to commonly used collision shapes.
pub mod popular_collider {
    use crate::physics::Collider;

    pub fn square(side: f32) -> Collider {
        Collider::cuboid(side / 2.0, side / 2.0)
    }

    pub fn rect(x: f32, y: f32) -> Collider {
        Collider::cuboid(x / 2.0, y / 2.0)
    }

    pub fn disc(radius: f32) -> Collider {
        Collider::ball(radius)
    }
}

/// All various layers of collision used in the game, used by the CollisionLayers component to check if a collision should happen or not.
#[derive(Clone, Copy)]
pub enum CollisionLayer {
    Character = 1 << 0,
    Gear = 1 << 1,
    Projectile = 1 << 2,
    Obstacle = 1 << 3,
}

impl Into<u32> for CollisionLayer {
    fn into(self) -> u32 {
        self as u32
    }
}

impl CollisionLayer {
    /// Fetch all possible collision layers.
    #[inline]
    pub const fn all() -> &'static [Self] {
        &[
            CollisionLayer::Character,
            CollisionLayer::Gear,
            CollisionLayer::Projectile,
            CollisionLayer::Obstacle,
        ]
    }
}

/// Try to find two entities in two queries without knowing which one entity exists in which query.
pub(crate) fn try_get_components_from_entities<
    'a,
    ComponentA: WorldQuery,
    ComponentB: WorldQuery,
    FilterA: ReadOnlyWorldQuery,
    FilterB: ReadOnlyWorldQuery,
>(
    query_a: &'a Query<ComponentA, FilterA>,
    query_b: &'a Query<ComponentB, FilterB>,
    entity_a: Entity,
    entity_b: Entity,
) -> Option<(Entity, Entity)> {
    if query_a.contains(entity_a) && query_b.contains(entity_b) {
        Some((entity_a, entity_b))
    } else if query_a.contains(entity_b) && query_b.contains(entity_a) {
        Some((entity_b, entity_a))
    } else {
        None
    }
}

/// Component which will be filled (if present) with a list of entities with which the current entity is currently in contact.
///
/// NOTE: will only be updated if ['ActiveEvents::COLLISION_EVENTS'] is also present on the entity.
#[derive(Component, Default, Reflect)]
pub struct OngoingCollisions(HashSet<Entity>);

impl OngoingCollisions {
    /// Returns the number of colliding entities.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if there is no colliding entities.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns `true` if the collisions contains the specified entity.
    #[must_use]
    pub fn contains(&self, entity: &Entity) -> bool {
        self.0.contains(entity)
    }

    /// An iterator visiting all colliding entities in arbitrary order.
    pub fn iter(&self) -> impl Iterator<Item = &Entity> + '_ {
        self.0.iter()
    }
}

/// Adds entity to [`OngoingCollisions`] on starting collision and removes from it
/// when the collision stops.
pub(super) fn update_ongoing_collisions(
    mut collision_events: EventReader<CollisionEvent>,
    mut collisions: Query<&mut OngoingCollisions>,
) {
    for event in collision_events.iter() {
        match event {
            CollisionEvent::Started(entity_a, entity_b, _) => {
                if let Ok(mut entities) = collisions.get_mut(*entity_a) {
                    entities.0.insert(*entity_b);
                }
                if let Ok(mut entities) = collisions.get_mut(*entity_b) {
                    entities.0.insert(*entity_a);
                }
            }
            CollisionEvent::Stopped(entity_a, entity_b, _) => {
                if let Ok(mut entities) = collisions.get_mut(*entity_a) {
                    entities.0.remove(entity_b);
                }
                if let Ok(mut entities) = collisions.get_mut(*entity_b) {
                    entities.0.remove(entity_a);
                }
            }
        };
    }
}

/// Removes deleted entities from [`OngoingCollisions`] component because
/// entity deletion doesn't emit [`CollisionEvent::Stopped`].
///
/// It's an intentional [issue](https://github.com/dimforge/rapier/issues/299) with Rapier.
pub(super) fn cleanup_ongoing_collisions(
    removed_rigid_bodies: RemovedComponents<RigidBody>,
    mut collisions: Query<&mut OngoingCollisions>,
) {
    for rigid_body in removed_rigid_bodies.iter() {
        for mut colliding_entities in collisions.iter_mut() {
            colliding_entities.0.remove(&rigid_body);
        }
    }
}

/// System to despawn entities (bullets, in particular) that get out of bounds.
/// Temporary fallback measurement, possibly, since normally it shouldn't happen.
pub fn handle_entities_out_of_bounds(
    mut commands: Commands,
    mut query_bodies: Query<(&Transform, Entity), With<Velocity>>,
) {
    const HALF_SCREEN_SPAN: f32 = SCREEN_SPAN * 0.5;
    // todo projectile velocity dampening
    for (transform, entity) in query_bodies.iter_mut() {
        if transform.translation.x.abs() > HALF_SCREEN_SPAN
            || transform.translation.y.abs() > HALF_SCREEN_SPAN
        {
            bevy::log::warn!("An entity {} got out of bounds!", entity.index());
            commands.entity(entity).despawn_recursive();
            // todo kill procedure first, probably
        }
    }
}

pub struct SpaceballsPhysicsPlugin;

impl Plugin for SpaceballsPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<OngoingCollisions>()
            .add_system(update_ongoing_collisions)
            .add_system_to_stage(CoreStage::PostUpdate, cleanup_ongoing_collisions);
    }
}
