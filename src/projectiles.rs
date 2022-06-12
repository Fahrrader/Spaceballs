use crate::characters::{Character, CHARACTER_SIZE};
use crate::collisions::Collider;
use crate::health::EntityDamagedEvent;
use crate::health::HitPoints;
use crate::teams::Team;
use bevy::core::Time;
use bevy::math::Vec2;
use bevy::prelude::{
    Bundle, Camera, Color, Commands, Component, DespawnRecursiveExt, Entity, EventWriter, Query,
    Res, Sprite, SpriteBundle, Transform, With,
};
use bevy::render::primitives::{Frustum, Sphere};
use bevy::sprite::collide_aabb::collide;
use bevy::utils::default;

pub const BULLET_SIZE: f32 = 5.0;
pub const BULLET_SPEED: f32 = 300.0;
pub const BULLET_DAMAGE: HitPoints = 5.0;

#[derive(Bundle)]
pub struct BulletBundle {
    bullet: Bullet,
    #[bundle]
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

impl BulletBundle {
    pub fn new(team: Team, transform: Transform, velocity: Vec2) -> Self {
        Self {
            bullet: Bullet { team, velocity },
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: Color::ALICE_BLUE,
                    custom_size: Some(Vec2::new(BULLET_SIZE, BULLET_SIZE)),
                    ..default()
                },
                transform: transform.clone(),
                ..default()
            },
            collider: Collider,
        }
    }
}

#[derive(Component)]
pub struct Bullet {
    pub team: Team,
    pub velocity: Vec2,
}

impl Bullet {
    pub fn stop(&mut self) {
        self.velocity = Vec2::default();
    }
}

// todo displace to movement, there will be no more need for frustum -- despawn on collide with arena bounds
pub fn handle_bullet_flight(
    mut commands: Commands,
    time: Res<Time>,
    mut query_bullets: Query<(&Bullet, &mut Transform, Entity)>,
    query_frustum: Query<&Frustum, With<Camera>>,
) {
    let dt = time.delta_seconds();

    let frustum = query_frustum.single();

    for (bullet, mut transform, entity) in query_bullets.iter_mut() {
        transform.translation += bullet.velocity.extend(0.0) * dt;

        let model_sphere = Sphere {
            center: transform.translation.into(),
            radius: BULLET_SIZE,
        };

        if !frustum.intersects_sphere(&model_sphere, false) {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// todo make a general collision event, parse event data here
pub fn handle_bullet_collision(
    mut commands: Commands,
    mut query_bullets: Query<(&Bullet, &Transform, Entity), With<Collider>>,
    mut query_characters: Query<(&Character, &Transform, Entity), With<Collider>>,
    mut ew_damage: EventWriter<EntityDamagedEvent>,
) {
    for (bullet, bullet_transform, bullet_entity) in query_bullets.iter_mut() {
        for (character, character_transform, character_entity) in query_characters.iter_mut() {
            let collision = collide(
                bullet_transform.translation,
                Vec2::new(BULLET_SIZE, BULLET_SIZE) * bullet_transform.scale.truncate(),
                character_transform.translation,
                Vec2::new(CHARACTER_SIZE, CHARACTER_SIZE) * character_transform.scale.truncate(),
            );

            if collision.is_some() {
                // perhaps send damage to bullets as well to handle multiple types / buffs?
                commands.entity(bullet_entity).despawn_recursive();
                if bullet.team != character.team {
                    ew_damage.send(EntityDamagedEvent {
                        entity: character_entity,
                        damage: BULLET_DAMAGE,
                    })
                } else {
                    // friendly fire!
                }
            }
        }
    }
}
