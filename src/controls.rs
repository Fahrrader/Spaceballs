use crate::actions::CharacterActionInput;
use crate::characters::PlayerControlled;
use bevy::input::{Axis, Input};
use bevy::prelude::{
    Commands, EventReader, Gamepad, GamepadAxis, GamepadAxisType, GamepadButton, GamepadButtonType,
    GamepadEvent, GamepadEventType, KeyCode, Query, Res, With,
};

// uncouth, probably refactor later -- may have to forbid multiple simultaneous inputs
/// Reset every input, preferably at the start of the game.
pub fn reset_input(mut query: Query<&mut CharacterActionInput>) {
    for mut character_inputs in query.iter_mut() {
        character_inputs.reset();
    }
}

/// System to convert external keyboard input to a player's character action input.
pub fn handle_keyboard_input(
    keys: Res<Input<KeyCode>>,
    mut query: Query<&mut CharacterActionInput, With<PlayerControlled>>,
) {
    let set_flag_if_keys_changed = |action: &mut bool, action_keys: Vec<KeyCode>| {
        let any_key_pressed = keys.any_pressed(action_keys);
        if any_key_pressed {
            *action |= any_key_pressed;
        }
    };

    let set_axis_if_keys_changed =
        |action: &mut f32, pos_action_keys: Vec<KeyCode>, neg_action_keys: Vec<KeyCode>| {
            let any_pos_key_pressed = keys.any_pressed(pos_action_keys);
            let any_neg_key_pressed = keys.any_pressed(neg_action_keys);

            if any_pos_key_pressed || any_neg_key_pressed {
                *action += any_pos_key_pressed as i32 as f32 - any_neg_key_pressed as i32 as f32;
            }
        };

    for mut player_actions in query.iter_mut() {
        set_axis_if_keys_changed(
            &mut player_actions.up,
            vec![KeyCode::W, KeyCode::Up],
            vec![KeyCode::S, KeyCode::Down],
        );
        set_axis_if_keys_changed(
            &mut player_actions.right,
            vec![KeyCode::D, KeyCode::Right],
            vec![KeyCode::A, KeyCode::Left],
        );
        set_flag_if_keys_changed(&mut player_actions.fire, vec![KeyCode::Space]);
        set_flag_if_keys_changed(&mut player_actions.reload, vec![KeyCode::R]);
        set_flag_if_keys_changed(&mut player_actions.use_environment_1, vec![KeyCode::F]);
        set_flag_if_keys_changed(&mut player_actions.use_environment_2, vec![KeyCode::C]);
    }
}

/// To-be resource holding the connected gamepad ID.
pub struct GamepadWrapper(Gamepad);

/// System to convert gamepad input to a player's character action input.
pub fn handle_gamepad_input(
    axes: Res<Axis<GamepadAxis>>,
    buttons: Res<Input<GamepadButton>>,
    connected_gamepad: Option<Res<GamepadWrapper>>,
    mut query: Query<&mut CharacterActionInput, With<PlayerControlled>>,
) {
    let gamepad = if let Some(gp) = connected_gamepad {
        gp.0
    } else {
        // no gamepad is connected
        return;
    };

    for mut player_actions in query.iter_mut() {
        let axis_lx = GamepadAxis {
            gamepad,
            axis_type: GamepadAxisType::LeftStickX,
        };
        let axis_ly = GamepadAxis {
            gamepad,
            axis_type: GamepadAxisType::LeftStickY,
        };

        if let (Some(x), Some(y)) = (axes.get(axis_lx), axes.get(axis_ly)) {
            const DEAD_ZONE: f32 = 0.25;
            if y.abs() > DEAD_ZONE {
                player_actions.up += y;
            }
            if x.abs() > DEAD_ZONE {
                player_actions.right += x;
            }
        }

        macro_rules! buttons_pressed {
            ($($e:expr),*) => {{
                let mut pressed = false;
                $(
                    let button = GamepadButton {
                        gamepad,
                        button_type: $e,
                    };
                    pressed |= buttons.pressed(button);
                )*
                pressed
            }
        }}

        player_actions.fire |=
            buttons_pressed!(GamepadButtonType::RightTrigger2, GamepadButtonType::South);
        player_actions.reload |=
            buttons_pressed!(GamepadButtonType::RightTrigger, GamepadButtonType::West);
        player_actions.use_environment_1 |= buttons_pressed!(GamepadButtonType::North);
        player_actions.use_environment_2 |= buttons_pressed!(GamepadButtonType::East);
    }
}

/// System to track gamepad connections and disconnections.
pub fn handle_gamepad_connections(
    mut commands: Commands,
    connected_gamepad: Option<Res<GamepadWrapper>>,
    mut gamepad_events: EventReader<GamepadEvent>,
) {
    for ev in gamepad_events.iter() {
        let id = ev.gamepad;
        match ev.event_type {
            GamepadEventType::Connected => {
                println!("New gamepad connected with ID: {:?}", id);

                // if we don't have any gamepad yet, use this one. Fix it for local multiplayer.
                if connected_gamepad.is_none() {
                    commands.insert_resource(GamepadWrapper(id));
                }
            }
            GamepadEventType::Disconnected => {
                println!("Lost gamepad connection with ID: {:?}", id);

                if let Some(GamepadWrapper(old_id)) = connected_gamepad.as_deref() {
                    if *old_id == id {
                        commands.remove_resource::<GamepadWrapper>();
                    }
                }
            }
            _ => {}
        }
    }
}
