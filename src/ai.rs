use crate::actions::ActionInput;
use crate::characters::PlayerControlled;
use crate::teams::Team;
use crate::Without;
use bevy::prelude::Query;
use rand::prelude::random;

pub const AI_DEFAULT_TEAM: Team = 8;

pub fn handle_ai_input(mut query: Query<&mut ActionInput, Without<PlayerControlled>>) {
    // I heard spinning is a good trick
    for mut actions in query.iter_mut() {
        actions.right = true;
        actions.fire = random::<f32>() < 0.25;
    }
}
