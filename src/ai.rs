use crate::actions::CharacterActionInput;
use crate::characters::{PlayerControlled, CHARACTER_RAD_SPEED};
use crate::Without;
use bevy::core::Time;
use bevy::log::info;
use bevy::prelude::{Query, Res};
use bevy::utils::default;
use rand::prelude::random;
use std::f64::consts::PI;

const TIME_STEP: f64 = 2.0 * PI / (CHARACTER_RAD_SPEED as f64);

pub fn handle_ai_input(
    time: Res<Time>,
    mut query: Query<&mut CharacterActionInput, Without<PlayerControlled>>,
) {
    // I heard spinning is a good trick
    for mut action_input in query.iter_mut() {
        action_input.replace_from(&advanced_action_routine(
            ((time.seconds_since_startup() % (TIME_STEP * 3.0)) / TIME_STEP).floor() as u8,
        ));
    }
}

fn advanced_action_routine(step: u8) -> CharacterActionInput {
    match step {
        0 => CharacterActionInput {
            up: true,
            ..default()
        },
        1 => CharacterActionInput {
            down: true,
            ..default()
        },
        _ => CharacterActionInput {
            left: true,
            fire: true,
            ..default()
        },
    }
}
