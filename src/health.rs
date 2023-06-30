use bevy::prelude::{Commands, Component, DespawnRecursiveExt, Entity, Query, With};
use bevy::reflect::{FromReflect, Reflect};

/// Floating point number signifying an entity's last arbitrary currency it pays to stay in this world.
pub type HitPoints = f32;

/// Holder component of an entity's hit points.
#[derive(Component, Debug, Default, PartialEq, Reflect, FromReflect)]
pub struct Health {
    hp: HitPoints,
    // armor? max?
}

impl Health {
    pub fn new(max_health: HitPoints) -> Self {
        Self { hp: max_health }
    }

    /// Get current hit points.
    pub const fn hp(&self) -> HitPoints {
        self.hp
    }

    /// Take off (or add, if negative) some hit points.
    pub fn damage(&mut self, damage: HitPoints) -> bool {
        self.hp -= damage;
        self.is_dead()
    }

    /// Check if the character is dead.
    pub fn is_dead(&self) -> bool {
        self.hp <= 0.0
    }
}

impl From<f32> for Health {
    fn from(hp: HitPoints) -> Self {
        Self { hp }
    }
}

/// Marker component indicating that the entity has reached zero hit points and is about to be despawned.
#[derive(Component, Debug, Default, Reflect, FromReflect)]
#[component(storage = "SparseSet")]
pub struct Dying;

/// System to sift through events of taking damage and apply it to entities' health.
pub fn handle_death(
    mut commands: Commands,
    // joke on Gogol's Dead Souls
    mut query_lives: Query<(&Health, Entity), With<Dying>>,
) {
    for (life, entity) in query_lives.iter_mut() {
        if life.is_dead() {
            commands.entity(entity).despawn_recursive();
        } else {
            commands.entity(entity).remove::<Dying>();
        }
    }
}
