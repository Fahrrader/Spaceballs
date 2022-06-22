use crate::actions::CharacterActionInput;
use crate::characters::PlayerControlled;
use crate::Without;
use bevy::prelude::Query;
use rand::prelude::random;

pub fn handle_ai_input(mut query: Query<&mut CharacterActionInput, Without<PlayerControlled>>) {
    // I heard spinning is a good trick
    for mut actions in query.iter_mut() {
        actions.right = true;
        actions.up = true;
        actions.fire = random::<f32>() < 0.25;
    }
}
