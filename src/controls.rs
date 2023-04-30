use crate::actions::CharacterActionInput;
use crate::characters::PlayerControlled;
use crate::GgrsConfig;
use bevy::ecs::schedule::SystemSet;
use bevy::input::{
    gamepad::{
        Gamepad, GamepadAxis, GamepadAxisType, GamepadButton, GamepadButtonType, GamepadConnection,
        GamepadConnectionEvent,
    },
    Axis, Input,
};
use bevy::prelude::{Commands, EventReader, In, KeyCode, Query, Res, Resource};
use bevy_ggrs::{ggrs, PlayerInputs};

/// Players' input data structure, used and encoded by GGRS and exchanged over the internet.
pub type GgrsInput = u8;

/// Set of systems for input handling for better organisation in the schedule.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum InputHandlingSet {
    /// Keyboard, gamepad, AI -- the stage of recording inputs from all sources.
    InputReading,
    // OnlineRollback,
    /// Stage of acting on the previously recorded inputs.
    ResponseProcessing,
}

/// System to record the players' online inputs (local and received) to the input struct used by the actuator systems.
pub fn handle_online_player_input(
    inputs: Res<PlayerInputs<GgrsConfig>>,
    mut query: Query<(&mut CharacterActionInput, &PlayerControlled)>,
) {
    for (mut player_inputs, player) in query.iter_mut() {
        // todo:mp check that the index is in bounds (i.e. player generation did not fuck up)
        let (input, _) = inputs[player.handle];
        *player_inputs = input.into();
    }
}

/// GGRS input system to record and convert local player input to the GGRS input structure.
pub fn process_input(
    _: In<ggrs::PlayerHandle>,
    keyboard: Res<Input<KeyCode>>,
    connected_gamepad: Option<Res<GamepadWrapper>>,
    gamepad_axes: Res<Axis<GamepadAxis>>,
    gamepad_buttons: Res<Input<GamepadButton>>,
    //mut query: Query<&mut CharacterActionInput, With<PlayerControlled>>,
) -> GgrsInput {
    let mut player_actions = CharacterActionInput::default();

    process_keyboard_input(&mut player_actions, &keyboard);
    process_gamepad_input(
        &mut player_actions,
        &connected_gamepad,
        &gamepad_axes,
        &gamepad_buttons,
    );

    player_actions.into()
}

/// Map external keyboard input to a player's character action input.
fn process_keyboard_input(actions: &mut CharacterActionInput, keyboard: &Input<KeyCode>) {
    let set_flag_if_keys_changed = |action: &mut bool, action_keys: Vec<KeyCode>| {
        let any_key_pressed = keyboard.any_pressed(action_keys);
        if any_key_pressed {
            *action |= any_key_pressed;
        }
    };

    let set_axis_if_keys_changed =
        |action: &mut f32, pos_action_keys: Vec<KeyCode>, neg_action_keys: Vec<KeyCode>| {
            let any_pos_key_pressed = keyboard.any_pressed(pos_action_keys);
            let any_neg_key_pressed = keyboard.any_pressed(neg_action_keys);

            if any_pos_key_pressed || any_neg_key_pressed {
                *action += any_pos_key_pressed as i32 as f32 - any_neg_key_pressed as i32 as f32;
            }
        };

    set_axis_if_keys_changed(
        &mut actions.up,
        vec![KeyCode::W, KeyCode::Up],
        vec![KeyCode::S, KeyCode::Down],
    );
    set_axis_if_keys_changed(
        &mut actions.right,
        vec![KeyCode::D, KeyCode::Right],
        vec![KeyCode::A, KeyCode::Left],
    );
    set_flag_if_keys_changed(&mut actions.fire, vec![KeyCode::Space]);
    set_flag_if_keys_changed(&mut actions.reload, vec![KeyCode::R]);
    set_flag_if_keys_changed(&mut actions.use_environment_1, vec![KeyCode::F]);
    set_flag_if_keys_changed(&mut actions.use_environment_2, vec![KeyCode::C]);
}

/*
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
 */
/// To-be resource holding the connected gamepad ID.
#[derive(Resource)]
pub struct GamepadWrapper(Gamepad);

/// Map gamepad input to a player's character action input.
pub fn process_gamepad_input(
    actions: &mut CharacterActionInput,
    connected_gamepad: &Option<Res<GamepadWrapper>>,
    axes: &Axis<GamepadAxis>,
    buttons: &Input<GamepadButton>,
) {
    let gamepad = if let Some(gp) = connected_gamepad {
        gp.0
    } else {
        // no gamepad is connected
        return;
    };

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
            actions.up += y;
        }
        if x.abs() > DEAD_ZONE {
            actions.right += x;
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

    actions.fire |= buttons_pressed!(GamepadButtonType::RightTrigger2, GamepadButtonType::South);
    actions.reload |= buttons_pressed!(GamepadButtonType::RightTrigger, GamepadButtonType::West);
    actions.use_environment_1 |= buttons_pressed!(GamepadButtonType::North);
    actions.use_environment_2 |= buttons_pressed!(GamepadButtonType::East);
}

/// System to track gamepad connections and disconnections.
pub fn handle_gamepad_connections(
    mut commands: Commands,
    connected_gamepad: Option<Res<GamepadWrapper>>,
    mut gamepad_events: EventReader<GamepadConnectionEvent>,
) {
    for ev in gamepad_events.iter() {
        let id = ev.gamepad;
        match &ev.connection {
            // name is skipped. maybe there will use for this later
            GamepadConnection::Connected(info) => {
                println!(
                    "New gamepad connected with ID: {:?}, name: {:?}",
                    id, info.name
                );

                // if we don't have any gamepad yet, use this one. Fix it for local multiplayer.
                if connected_gamepad.is_none() {
                    commands.insert_resource(GamepadWrapper(id));
                }
            }
            GamepadConnection::Disconnected => {
                println!("Lost gamepad connection with ID: {:?}", id);

                if let Some(GamepadWrapper(old_id)) = connected_gamepad.as_deref() {
                    if *old_id == id {
                        commands.remove_resource::<GamepadWrapper>();
                    }
                }
            }
        }
    }
}
