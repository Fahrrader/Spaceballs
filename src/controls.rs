use crate::actions::CharacterActionInput;
use crate::characters::PlayerControlled;
use bevy::input::Input;
use bevy::prelude::{KeyCode, Query, Res, With};

pub fn handle_player_input(
    keys: Res<Input<KeyCode>>,
    mut query: Query<&mut CharacterActionInput, With<PlayerControlled>>,
) {
    for mut player_actions in query.iter_mut() {
        player_actions.up = keys.pressed(KeyCode::W) || keys.pressed(KeyCode::Up);
        player_actions.down = keys.pressed(KeyCode::S) || keys.pressed(KeyCode::Down);
        player_actions.left = keys.pressed(KeyCode::A) || keys.pressed(KeyCode::Left);
        player_actions.right = keys.pressed(KeyCode::D) || keys.pressed(KeyCode::Right);
        player_actions.fire = keys.pressed(KeyCode::Space);
    }
}
