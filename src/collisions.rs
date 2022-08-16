use crate::characters::CHARACTER_SIZE;
use crate::projectiles::BULLET_SIZE;
use bevy::math::Vec2;
use bevy::prelude::{Component, Entity, EventWriter, Query, Transform, With};

#[derive(Component)]
pub struct Collider;

/*pub fn collide() {
    todo!()
}*/

pub struct CollisionEvent {
    pub entity_a: Entity,
    pub entity_b: Entity,
    //pub velocity: Vec2,
}

// todo possibly optimize with layers -- or just use rapier's
pub fn handle_collision(
    query: Query<(&Transform, Entity), With<Collider>>, // Velocity
    mut ew_collision: EventWriter<CollisionEvent>,
) {
    for (transform_a, entity_a) in query.iter() {
        for (transform_b, entity_b) in query.iter() {
            if entity_a == entity_b {
                continue;
            }
            let collision = bevy::sprite::collide_aabb::collide(
                transform_a.translation,
                Vec2::new(BULLET_SIZE, BULLET_SIZE), // * bullet_transform.scale.truncate(),
                transform_b.translation,
                Vec2::new(CHARACTER_SIZE, CHARACTER_SIZE), // * character_transform.scale.truncate(),
            );

            if collision.is_some() {
                ew_collision.send(CollisionEvent { entity_a, entity_b });
            }
        }
    }
}
