use crate::actions::CharacterActionInput;
use crate::characters::{PlayerControlled, CHARACTER_RAD_SPEED};
use crate::Without;
use bevy::prelude::{Query, Res, Time};
use bevy::utils::default;
// use rand::prelude::random;
use std::f64::consts::PI;

const TIME_STEP: f64 = 2.0 * PI / (CHARACTER_RAD_SPEED as f64);

// todo possibly split AI calculation between participating machines, depending on some runtime performance metrics?
/// System to give AI characters something to do this frame. Uses a function of time to calculate the set of actions performed.
pub fn handle_ai_input(
    // todo:mp time should be synced from the moment the guys join, and the game starts
    time: Res<Time>,
    mut query: Query<&mut CharacterActionInput, Without<PlayerControlled>>,
) {
    // I heard spinning is a good trick
    for mut action_input in query.iter_mut() {
        *action_input = advanced_action_routine(
            ((time.elapsed_seconds_f64() % (TIME_STEP * 3.0)) / TIME_STEP).floor() as u8,
        );
    }
}

/// Some bullshit things to cycle the AI behavior through for now.
fn advanced_action_routine(step: u8) -> CharacterActionInput {
    match step {
        0 => CharacterActionInput {
            up: 1.0,
            ..default()
        },
        1 => CharacterActionInput {
            up: -1.0,
            ..default()
        },
        _ => CharacterActionInput {
            right: -1.0,
            fire: true,
            ..default()
        },
    }
}
