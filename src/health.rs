use crate::collisions::Collider;
use bevy::prelude::{Commands, Component, DespawnRecursiveExt, Entity, EventReader, Query, With};

pub type HitPoints = f32;

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

    pub fn damage(&mut self, damage: HitPoints) -> bool {
        self.hp -= damage;
        self.is_dead()
    }

    pub fn is_dead(&self) -> bool {
        self.hp <= 0.0
    }
}

pub struct EntityDamagedEvent {
    pub entity: Entity,
    pub damage: HitPoints,
}

pub fn handle_damage(
    mut commands: Commands,
    mut damage_events: EventReader<EntityDamagedEvent>,
    mut query_lives: Query<&mut Health, With<Collider>>,
) {
    for event in damage_events.iter() {
        let life = query_lives.get_mut(event.entity);
        if let Ok(mut life) = life {
            life.damage(event.damage);
            if life.is_dead() {
                commands.entity(event.entity).despawn_recursive();
            }
        }
    }
}
