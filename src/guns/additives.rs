use crate::characters::{CHARACTER_MAX_HEALTH, CHARACTER_SIZE};
use crate::guns::presets::RAILGUN;
use crate::health::HitPoints;
use crate::physics::OngoingCollisions;
use crate::projectiles::Projectile;
use crate::teams::Team;
use crate::Health;
use bevy::prelude::{Commands, Component, Query, Res, Time, With};
use bevy::reflect::{FromReflect, Reflect};

pub mod railgun {
    use super::*;

    /// Due to the rail gun's penetrative properties, damage is applied per second of travel inside a body.
    /// Correlates heavily with projectile speed. Used by a special system.
    pub const PENETRATION_DAMAGE_PER_SECOND: HitPoints = CHARACTER_MAX_HEALTH / 5.0 // under a normal angle and fully crossing the body, should kill in [5] hits
        * RAILGUN.projectile_speed
        / CHARACTER_SIZE;

    /// Marker component for a rail gun projectile.
    #[derive(Component, Debug, Default, Reflect, FromReflect)]
    pub struct RailGunThing;

    /// System to continually deal damage to bodies that rail gun slugs travel through.
    pub fn handle_railgun_penetration_damage(
        mut commands: Commands,
        time: Res<Time>,
        query_bullets: Query<(&OngoingCollisions, &Projectile, &Team), With<RailGunThing>>,
        mut query_bodies: Query<(&mut Health, Option<&Team>)>,
    ) {
        for (collisions, bullet, bullet_team) in query_bullets.iter() {
            let gun_stats = bullet.gun_type.stats();
            for body_entity in collisions.iter() {
                if let Ok(mut body) = query_bodies.get_mut(*body_entity) {
                    Projectile::do_damage(
                        &mut commands,
                        (gun_stats, bullet_team),
                        (*body_entity, &mut body.0, body.1),
                        Some(PENETRATION_DAMAGE_PER_SECOND * time.delta_seconds()),
                    );
                }
            }
        }
    }
}

pub mod systems {
    pub use super::railgun::handle_railgun_penetration_damage;
}
