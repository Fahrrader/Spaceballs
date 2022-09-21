use bevy::prelude::{Commands, Component, DespawnRecursiveExt, Entity, Query, With};

/// Floating point number signifying an entity's last arbitrary currency it pays to stay in this world.
pub type HitPoints = f32;

/// Holder component of an entity's hit points.
#[derive(Component)]
pub struct Health {
    pub hp: HitPoints,
    // armor? max?
}

impl Default for Health {
    fn default() -> Self {
        Self { hp: 1.0 }
    }
}

impl Health {
    pub fn new(max_health: HitPoints) -> Self {
        Self { hp: max_health }
    }

    /// Take off (or add, if negative) some hit points.
    pub(crate) fn damage(&mut self, damage: HitPoints) -> bool {
        self.hp -= damage;
        self.is_dead()
    }

    /// Check if the character is dead.
    pub fn is_dead(&self) -> bool {
        self.hp <= 0.0
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Dying;

/// System to sift through events of taking damage and apply it to entities' health.
pub fn handle_death(
    mut commands: Commands,
    // todo joke on Gogol's Dead Souls
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
