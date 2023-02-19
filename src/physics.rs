use crate::SCREEN_SPAN;
use bevy::ecs::query::{ReadOnlyWorldQuery, WorldQuery};
use bevy::math::Vec3;
use bevy::prelude::{
    default, Bundle, Color, Commands, DespawnRecursiveExt, Entity, Query, Sprite, SpriteBundle,
    Transform, With,
};
//use heron::prelude::*;

/// The size of an obstacle chunk. Useful to keep about the same as a character's body size to configure the terrain easier.
pub const OBSTACLE_CHUNK_SIZE: f32 = 50.0;
pub const DEFAULT_OBSTACLE_COLOR: Color = Color::WHITE;
/*
/// Collection of components desired for physics and collision simulation.
#[derive(Bundle)]
pub struct KinematicsBundle {
    pub rigidbody: RigidBody,
    pub velocity: Velocity,
    pub damping: Damping,
    pub physic_material: PhysicMaterial,
    pub collider: CollisionShape,
    pub collision_layers: CollisionLayers,
}

impl Default for KinematicsBundle {
    fn default() -> Self {
        Self {
            rigidbody: RigidBody::Dynamic,
            velocity: Velocity::default(),
            damping: Damping::default(),
            physic_material: PhysicMaterial::default(),
            collider: CollisionShape::default(),
            collision_layers: CollisionLayers::none(),
        }
    }
}

// todo look into transitioning to rapier from heron for more precise control once more abilities and guns are added
impl KinematicsBundle {
    /// Create a new kinematic bundle.
    ///
    /// [shape] is akin to a collider mesh, an invisible shape used to calculate collisions; it doesn't need to be the exact shape of the object.
    ///
    /// [collision_group] is a [`CollisionLayer`] the entity would belong to, i.e. a bullet would be a [`Projectile`](crate::CollisionLayer::Projectile).
    /// It makes up the mask of heron's [`CollisionLayers`] struct.
    ///
    /// [collided_groups] is an array of [`CollisionLayer`] enums the entity would collide with and fire events in such a case.
    /// It makes up the groups of heron's [`CollisionLayers`] struct.
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

    pub fn with_angular_velocity_in_rads(mut self, axis: Vec3, radians: f32) -> Self {
        self.velocity.angular = AxisAngle::new(axis, radians);
        self
    }

    pub fn with_linear_damping(mut self, damping: f32) -> Self {
        self.damping.linear = damping;
        self
    }

    pub fn with_angular_damping(mut self, damping: f32) -> Self {
        self.damping.angular = damping;
        self
    }

    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.physic_material.restitution = restitution;
        self
    }

    pub fn with_density(mut self, density: f32) -> Self {
        self.physic_material.density = density;
        self
    }
}

/// Standard rectangular obstacle, stopping characters and bullets alike.
/// Uses [`OBSTACLE_CHUNK_SIZE`] to determine its dimensions in addition to the provided scale.
#[derive(Bundle)]
pub struct RectangularObstacleBundle {
    pub rigidbody: RigidBody,
    pub collider: CollisionShape,
    pub collision_layers: CollisionLayers,
    #[bundle]
    pub sprite_bundle: SpriteBundle,
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
    /// Make a thing that stuff can't pass through. Warning: calculates its size based on the scale given and the normal obstacle size.
    pub fn new(transform: Transform) -> Self {
        Self {
            collider: PopularCollisionShape::SquareCell(OBSTACLE_CHUNK_SIZE).get(transform.scale),
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: DEFAULT_OBSTACLE_COLOR,
                    ..default()
                },
                transform: transform.with_scale(transform.scale * OBSTACLE_CHUNK_SIZE),
                ..default()
            },
            ..default()
        }
    }
}*/

/// All various layers of collision used in the game, used by the CollisionLayers component to check if a collision should happen or not.
//#[derive(PhysicsLayer)]
pub enum CollisionLayer {
    Character,
    Gear,
    Projectile,
    Obstacle,
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
/*
/// Collection of shortcuts to commonly used collision shapes.
pub enum PopularCollisionShape {
    SquareCell(f32),
    RectangularCell(f32, f32),
    Disc(f32),
}

impl PopularCollisionShape {
    pub fn get(self, scale: Vec3) -> CollisionShape {
        match self {
            Self::SquareCell(size) => CollisionShape::Cuboid {
                half_extends: size / 2.0 * scale,
                border_radius: None,
            },
            Self::RectangularCell(size_x, size_y) => CollisionShape::Cuboid {
                half_extends: Vec3::new(scale.x * size_x / 2.0, scale.y * size_y / 2.0, scale.z),
                border_radius: None,
            },
            Self::Disc(size) => CollisionShape::Sphere {
                radius: size / 2.0 * scale.length(),
            },
        }
    }
}*/

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
/*
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
            bevy::log::warn!("An entity {} got out of bounds!", entity.id());
            commands.entity(entity).despawn_recursive();
            // todo kill procedure first, probably
        }
    }
}*/
