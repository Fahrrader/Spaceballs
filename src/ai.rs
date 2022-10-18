use crate::actions::CharacterActionInput;
use crate::characters::PlayerControlled;
use crate::teams::TeamNumber;
use crate::Without;
use bevy::prelude::Query;
use rand::prelude::random;

pub const AI_DEFAULT_TEAM: TeamNumber = 8;

pub fn handle_ai_input(mut query: Query<&mut CharacterActionInput, Without<PlayerControlled>>) {
    // I heard spinning is a good trick
    for mut actions in query.iter_mut() {
        actions.right = 1.0;
        actions.up = 1.0;
        actions.fire = random::<f32>() < 0.25;
    }
}
