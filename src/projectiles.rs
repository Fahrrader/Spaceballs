use crate::characters::Character;
use crate::collisions::{Collider, CollisionEvent};
use crate::health::EntityDamagedEvent;
use crate::health::HitPoints;
use crate::movement::Velocity;
use crate::teams::Team;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{
    Bundle, Camera, Color, Commands, Component, DespawnRecursiveExt, Entity, EventReader,
    EventWriter, Query, Sprite, SpriteBundle, Transform, With,
};
use bevy::render::primitives::{Frustum, Sphere};
use bevy::utils::default;

pub const BULLET_SIZE: f32 = 5.0;
pub const BULLET_SPEED: f32 = 300.0;
pub const BULLET_DAMAGE: HitPoints = 5.0;

#[derive(Bundle)]
pub struct BulletBundle {
    bullet: Bullet,
    velocity: Velocity,
    #[bundle]
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

impl BulletBundle {
    pub fn new(team: Team, transform: Transform, velocity: Vec3) -> Self {
        Self {
            bullet: Bullet { team },
            velocity: Velocity {
                linear: velocity,
                ..default()
            },
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: Color::ALICE_BLUE,
                    custom_size: Some(Vec2::new(BULLET_SIZE, BULLET_SIZE)),
                    ..default()
                },
                transform,
                ..default()
            },
            collider: Collider,
        }
    }
}

#[derive(Component)]
pub struct Bullet {
    pub team: Team,
}

// todo remove soon, there will be no more need for frustum -- despawn on collide with arena bounds
pub fn handle_bullet_flight(
    mut commands: Commands,
    mut query_bullets: Query<(&Transform, Entity), With<Bullet>>,
    query_frustum: Query<&Frustum, With<Camera>>,
) {
    let frustum = query_frustum.single();

    for (transform, entity) in query_bullets.iter_mut() {
        let model_sphere = Sphere {
            center: transform.translation.into(),
            radius: BULLET_SIZE,
        };

        if !frustum.intersects_sphere(&model_sphere, false) {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn handle_bullet_collision_events(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    query_characters: Query<&Character>,
    query_bullets: Query<&Bullet>, // velocity?
    mut ew_damage: EventWriter<EntityDamagedEvent>,
) {
    for event in collision_events.iter() {
        let bullet = query_bullets.get(event.entity_a);
        let character = query_characters.get(event.entity_b);
        // perhaps send damage to bullets as well to handle multiple types / buffs?
        if let (Ok(bullet), Ok(character)) = (bullet, character) {
            commands.entity(event.entity_a).despawn_recursive();
            if bullet.team != character.team {
                ew_damage.send(EntityDamagedEvent {
                    entity: event.entity_b,
                    damage: BULLET_DAMAGE,
                })
            } else {
                // friendly fire!
            }
        }
    }
}
