use crate::characters::PlayerControlled;
use crate::network::PlayerHandle;
use crate::ui::chat::ChatMessage;
use crate::PlayerDied;
use bevy::prelude::{
    Commands, Component, DespawnRecursiveExt, Entity, EventReader, EventWriter, Query,
};
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
pub struct Dying {
    pub(crate) by_shooter: Option<PlayerHandle>,
}

/// System to sift through events of taking damage and apply it to entities' health.
pub fn handle_death(
    mut commands: Commands,
    mut query_lives: Query<(&Health, Entity, Option<&PlayerControlled>, &Dying)>,
    mut dead_teller: EventWriter<PlayerDied>,
) {
    for (life, entity, maybe_player, dying) in query_lives.iter_mut() {
        if life.is_dead() {
            commands.entity(entity).despawn_recursive();
            if let Some(player) = maybe_player {
                dead_teller.send(PlayerDied {
                    player_handle: player.handle,
                    killed_by: dying.by_shooter,
                });
            }
        } else {
            commands.entity(entity).remove::<Dying>();
        }
    }
}

pub fn handle_reporting_death(
    mut dead_reader: EventReader<PlayerDied>,
    mut postman: EventWriter<ChatMessage>,
) {
    for event in dead_reader.iter() {
        let message = if let Some(killer) = event.killed_by {
            ChatMessage {
                message: "{0} killed {1}!".to_string(),
                player_handles: vec![killer, event.player_handle],
            }
        } else {
            ChatMessage {
                message: "{0} died!".to_string(),
                player_handles: vec![event.player_handle],
            }
        };
        postman.send(message);
    }
}
