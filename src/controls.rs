use crate::characters::PlayerControlled;
use crate::network::{GGRSConfig, GGRSInput};
use crate::ui::input_consumption::{
    ActiveInputConsumerLayers, GAME_INPUT_LAYER, PAUSE_INPUT_LAYER,
};
use crate::GamePauseEvent;
use bevy::ecs::schedule::SystemSet;
use bevy::input::{
    gamepad::{
        Gamepad, GamepadAxis, GamepadAxisType, GamepadButton, GamepadButtonType, GamepadConnection,
        GamepadConnectionEvent,
    },
    Axis, Input,
};
use bevy::prelude::{
    Commands, Component, EventReader, EventWriter, In, KeyCode, Local, Query, Res, Resource,
};
use bevy::reflect::{FromReflect, Reflect};
use bevy_ggrs::{ggrs, PlayerInputs};

/// Set of systems for input handling for better organisation in the schedule.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum InputHandlingSet {
    /// Keyboard, gamepad, AI -- the stage of recording inputs from all sources.
    InputReading,
    // OnlineRollback,
    /// Stage of acting on the previously recorded inputs.
    ResponseProcessing,
}

/// Inputs for characters to act on during the next frame.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Reflect, FromReflect)]
pub struct CharacterActionInput {
    /// Forward movement. Clamp to 1.0!
    pub up: f32,
    /// Right-hand movement. Clamp to 1.0!
    pub right: f32,
    /// To shoot or not to shoot? That is the question.
    pub fire: bool,
    /// Whether a reload must be triggered this frame.
    pub reload: bool,

    /// Whether an environmental interactive action must be triggered this frame,
    /// such as picking guns up from the ground.
    pub interact_1: bool,
    /// Whether an auxiliary environmental interactive action must be triggered this frame,
    /// such as throwing equipped guns away.
    pub interact_2: bool,
}

impl CharacterActionInput {
    /// Bring every input to the default state.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Get direction of linear speed.
    pub fn speed(&self) -> f32 {
        self.up.clamp(-1.0, 1.0)
    }

    /// Get direction of rotational speed.
    pub fn angular_speed(&self) -> f32 {
        self.right.clamp(-1.0, 1.0)
    }
}

/// System to record the players' online inputs (local and received) to the input struct used by the actuator systems.
pub fn handle_online_player_input(
    inputs: Res<PlayerInputs<GGRSConfig>>,
    mut query: Query<(&mut CharacterActionInput, &PlayerControlled)>,
    mut index_oob_timeout: Local<Vec<usize>>,
) {
    for (mut player_inputs, player) in query.iter_mut() {
        match inputs.get(player.handle) {
            Some(&(input, _)) => *player_inputs = input.into(),
            None => {
                // Report that the index is not in bounds (i.e. player generation fucked up)
                if !index_oob_timeout.contains(&player.handle) {
                    bevy::log::error!(
                        "Player handle {} out of bounds when reading online inputs! Expected player count and the length of the received player inputs: {}.",
                        player.handle,
                        inputs.len(),
                    );
                    index_oob_timeout.push(player.handle);
                }
            }
        };
    }
}

/// GGRS input system to record and convert local player input to the GGRS input structure.
pub fn process_input(
    _: In<ggrs::PlayerHandle>,
    keyboard: Res<Input<KeyCode>>,
    input_consumers: Res<ActiveInputConsumerLayers>,
    connected_gamepad: Option<Res<GamepadWrapper>>,
    gamepad_axes: Res<Axis<GamepadAxis>>,
    gamepad_buttons: Res<Input<GamepadButton>>,
    //mut query: Query<&mut CharacterActionInput, With<PlayerControlled>>,
) -> GGRSInput {
    let mut player_actions = CharacterActionInput::default();

    if input_consumers.is_input_allowed_for_layer(&GAME_INPUT_LAYER) {
        process_keyboard_input(&mut player_actions, &keyboard);
        #[cfg(target_arch = "wasm32")]
        process_js_joysticks_input(&mut player_actions);
        process_gamepad_input(
            &mut player_actions,
            &connected_gamepad,
            &gamepad_axes,
            &gamepad_buttons,
        );
    }

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
    set_flag_if_keys_changed(&mut actions.interact_1, vec![KeyCode::F]);
    set_flag_if_keys_changed(&mut actions.interact_2, vec![KeyCode::C]);
}

#[cfg(target_arch = "wasm32")]
fn process_js_joysticks_input(actions: &mut CharacterActionInput) {
    let js_input = crate::js_interop::get_sticks_positions_from_js();
    actions.up += js_input[1];
    actions.right += js_input[0];
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

    macro_rules! set_flag_if_pressed {
        ($action:expr, [$($e:expr),*]) => {{
            $(
                let button = GamepadButton {
                    gamepad,
                    button_type: $e,
                };
                $action |= buttons.pressed(button);
            )*
        }
    }}

    set_flag_if_pressed!(
        actions.fire,
        [GamepadButtonType::RightTrigger2, GamepadButtonType::South]
    );
    set_flag_if_pressed!(
        actions.reload,
        [GamepadButtonType::RightTrigger, GamepadButtonType::West]
    );
    set_flag_if_pressed!(actions.interact_1, [GamepadButtonType::North]);
    set_flag_if_pressed!(actions.interact_2, [GamepadButtonType::East]);
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
                bevy::log::info!(
                    "New gamepad connected with ID: {:?}, name: {:?}",
                    id,
                    info.name
                );

                // if we don't have any gamepad yet, use this one. Fix it for local multiplayer.
                if connected_gamepad.is_none() {
                    commands.insert_resource(GamepadWrapper(id));
                }
            }
            GamepadConnection::Disconnected => {
                bevy::log::info!("Lost gamepad connection with ID: {:?}", id);

                if let Some(GamepadWrapper(old_id)) = connected_gamepad.as_deref() {
                    if *old_id == id {
                        commands.remove_resource::<GamepadWrapper>();
                    }
                }
            }
        }
    }
}

/// System that listens for pause inputs either from a keyboard or a connected gamepad,
/// sending a `GamePauseEvent::Toggle` event.
pub fn handle_pause_input(
    keyboard: Res<Input<KeyCode>>,
    input_consumers: Res<ActiveInputConsumerLayers>,
    connected_gamepad: Option<Res<GamepadWrapper>>,
    gamepad_buttons: Res<Input<GamepadButton>>,
    mut pause_events: EventWriter<GamePauseEvent>,
) {
    if input_consumers.is_input_blocked_for_layer(&PAUSE_INPUT_LAYER) {
        return;
    }

    if keyboard.just_pressed(KeyCode::Escape)
        || connected_gamepad
            .filter(|gp| {
                gamepad_buttons.pressed(GamepadButton {
                    gamepad: gp.0,
                    button_type: GamepadButtonType::Start,
                })
            })
            .is_some()
    {
        pause_events.send(GamePauseEvent::Toggle);
    }
}
