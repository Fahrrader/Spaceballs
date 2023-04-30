use crate::actions::CharacterActionInput;
use crate::characters::{AiControlled, CHARACTER_RAD_SPEED};
use bevy::prelude::{Component, Query, Res, Time, With};
use bevy::utils::default;
// use rand::prelude::random;
use std::f32::consts::PI;

/// One possible AI controller component, deciding an AI's input. Contains the current time tracker.
/// For now, it performs incredible maneuvers.
#[derive(Component, Default)]
pub struct AiActionRoutine(pub f32);

impl AiActionRoutine {
    /// Seconds it takes to proceed to the next stage in the routine.
    const STAGE_LENGTH: f32 = /*2.0 * */ PI / CHARACTER_RAD_SPEED;
    // const TIME_STEP: f32 = Self::STAGE_LENGTH / 120.0;

    /// Some bullshit things to cycle the AI behavior through for now.
    fn action_routine(stage: u8) -> CharacterActionInput {
        // I heard spinning is a good trick
        match stage {
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

    /// Increase the AI's time tracker and evaluate its routine at that point.
    pub fn increment_routine_step(&mut self, delta_seconds: f32) -> CharacterActionInput {
        // locked step - guarantees completion and the order of inputs, doesn't give two fucks about time sanity / synchronicity
        // time (this) - guarantees that the stage will shift at the same time, reinforced by the response logic, fails when time skipping
        self.0 += delta_seconds;
        let stage = ((self.0 % (Self::STAGE_LENGTH * 3.0)) / Self::STAGE_LENGTH).floor() as u8;
        Self::action_routine(stage)
    }
}

// todo possibly split AI calculation between participating machines, depending on some runtime performance metrics?
/// System to give AI characters something to do this frame. Uses a function of time to calculate the set of actions performed.
pub fn handle_ai_input(
    time: Res<Time>,
    mut query: Query<(&mut CharacterActionInput, &mut AiActionRoutine), With<AiControlled>>,
) {
    for (mut action_input, mut action_routine) in query.iter_mut() {
        *action_input = action_routine.increment_routine_step(time.delta_seconds());
    }
}
