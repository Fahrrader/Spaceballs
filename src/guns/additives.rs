use crate::characters::{CHARACTER_MAX_HEALTH, CHARACTER_SIZE};
use crate::health::HitPoints;
use crate::physics::OngoingCollisions;
use crate::projectiles::Projectile;
use crate::teams::Team;
use crate::Health;
use bevy::prelude::*;
use bevy::reflect::{FromReflect, Reflect};

pub mod railgun {
    use super::*;

    const PENETRATIONS_TO_KILL: f32 = 4.0;
    /// Due to the rail gun's penetrative properties, damage is applied per second of travel inside a body.
    /// Correlates heavily with projectile speed. Used by a special system.
    // under a normal angle and fully crossing the body, should kill in [`PENETRATIONS_TO_KILL`]
    pub const PENETRATION_DAMAGE_PER_DISTANCE: HitPoints =
        CHARACTER_MAX_HEALTH / CHARACTER_SIZE / PENETRATIONS_TO_KILL;

    /// Marker component for a rail gun projectile.
    #[derive(Component, Debug, Default, Reflect, FromReflect)]
    pub struct RailGunThing {
        pub previous_position: Vec3,
    }

    /// System to continually deal damage to bodies that rail gun slugs travel through.
    pub fn handle_railgun_penetration_damage(
        mut commands: Commands,
        mut query_bullets: Query<(
            &OngoingCollisions,
            &Projectile,
            &Team,
            &Transform,
            &mut RailGunThing,
        )>,
        mut query_bodies: Query<(&mut Health, Option<&Team>)>,
    ) {
        for (collisions, bullet, bullet_team, bullet_transform, mut railgun_thing) in
            query_bullets.iter_mut()
        {
            let gun_stats = bullet.gun_type.stats();
            for body_entity in collisions.iter() {
                if let Ok((mut body_health, body_team)) = query_bodies.get_mut(*body_entity) {
                    let distance_travelled =
                        (bullet_transform.translation - railgun_thing.previous_position).length();
                    let damage = PENETRATION_DAMAGE_PER_DISTANCE * distance_travelled;
                    Projectile::do_damage(
                        &mut commands,
                        (gun_stats, bullet_team),
                        (*body_entity, &mut body_health, body_team),
                        Some(damage),
                    );
                }
            }
            railgun_thing.previous_position = bullet_transform.translation;
        }
    }

    /*pub fn handle_railgun_thing_addition(
        mut query_things: Query<(&mut RailGunThing, &Transform), Added<RailGunThing>>
    ) {
        query_things.for_each_mut(|(mut thing, transform)| {
            thing.previous_position = transform.translation;
        });
    }*/
}

pub mod systems {
    pub use super::railgun::handle_railgun_penetration_damage;
}
