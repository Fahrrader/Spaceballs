use crate::ui::{remove_focus_from_non_focused_entities, Focus, FocusSwitchedEvent};
use crate::MenuState;
use bevy::prelude::*;
use std::time::Duration;

/// Text input component, meant to be working together with [`Text`] and [`Focus<TextInput>`].
///
/// `Focus<TextInput>` controls whether the `TextInput` component should currently accept user input,
/// and `Text` displays the actual text.
#[derive(Component, Debug)]
pub struct TextInput {
    pub text: String,
    pub placeholder: String,
    pub text_style: TextStyle,
    pub cursor_position: usize,
    pub max_symbols: usize,
    // regex: ...
    // line_breaks_allowed: ...
}

impl Default for TextInput {
    fn default() -> Self {
        Self {
            text: "".into(),
            placeholder: "".into(),
            text_style: TextStyle::default(),
            cursor_position: 0,
            max_symbols: usize::MAX,
        }
    }
}

impl TextInput {
    pub fn new(initial_value: String, placeholder: Option<String>, text_style: TextStyle) -> Self {
        Self {
            text: initial_value.clone(),
            placeholder: placeholder.unwrap_or("".into()),
            text_style,
            cursor_position: initial_value.len(),
            max_symbols: usize::MAX,
        }
    }
}

fn handle_text_input_addition(mut text_query: Query<(&mut Text, &TextInput), Added<TextInput>>) {
    for (mut text, input) in text_query.iter_mut() {
        let mut placeholder_style = input.text_style.clone();
        placeholder_style.color = placeholder_style
            .color
            .with_a(placeholder_style.color.a() * 0.33);
        text.sections.push(TextSection::new(
            input.placeholder.clone(),
            placeholder_style,
        ));

        text.sections.push(TextSection::new(
            input.text.clone(),
            input.text_style.clone(),
        ));
    }
}

fn handle_text_input_new_focus(
    mut focus_query: Query<(Entity, &Interaction, &mut Focus<TextInput>), Changed<Interaction>>,
    mut focus_switch_events: EventWriter<FocusSwitchedEvent<TextInput>>,
) {
    for (entity, interaction, mut focus_input) in focus_query.iter_mut() {
        match interaction {
            Interaction::Clicked => {
                *focus_input = Focus::Focused(TextInput::default());
                focus_switch_events.send(FocusSwitchedEvent::new(Some(entity)));
            }
            _ => {} // Focus::None,
        }
    }
}

fn handle_text_input(
    mut text_query: Query<(&mut TextInput, &Focus<TextInput>)>,
    mut characters_evs: EventReader<ReceivedCharacter>,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut key_press_timer: Local<Timer>,
) {
    const KEY_PRESS_INITIAL_TIMEOUT: f32 = 0.5;
    const KEY_PRESS_TIMEOUT: f32 = 0.05;

    let mut handle_key_press_with_timeout = |key: KeyCode, action: &mut dyn FnMut()| {
        if keys.just_pressed(key) {
            action();

            key_press_timer.set_duration(Duration::from_secs_f32(KEY_PRESS_INITIAL_TIMEOUT));
            key_press_timer.reset();
        } else if keys.pressed(key) && key_press_timer.tick(time.delta()).just_finished() {
            action();

            key_press_timer.set_duration(Duration::from_secs_f32(KEY_PRESS_TIMEOUT));
            key_press_timer.reset();
        } else if keys.just_released(key) {
            key_press_timer.reset();
        }
    };

    for (mut input, focus) in text_query.iter_mut() {
        if focus.is_none() {
            continue;
        }

        for ev in characters_evs.iter() {
            if ev.char.is_ascii_graphic() || ev.char.is_ascii_whitespace() {
                let idx = input.cursor_position;
                // Could be unsafe due to some characters being more than 1 byte long
                input.text.insert(idx, ev.char);
                input.cursor_position += 1;
            }
        }

        handle_key_press_with_timeout(KeyCode::Return, &mut || {
            let idx = input.cursor_position;
            input.text.insert(idx, '\n');
            input.cursor_position += 1;
        });

        handle_key_press_with_timeout(KeyCode::Left, &mut || {
            input.cursor_position = input.cursor_position.saturating_sub(1);
        });

        handle_key_press_with_timeout(KeyCode::Right, &mut || {
            input.cursor_position = (input.cursor_position + 1).min(input.text.len());
        });

        handle_key_press_with_timeout(KeyCode::Back, &mut || {
            if input.cursor_position != 0 {
                let mut chars: Vec<char> = input.text.chars().collect();
                chars.remove(input.cursor_position - 1);
                input.text = chars.into_iter().collect();
                input.cursor_position -= 1;
            }
        });

        handle_key_press_with_timeout(KeyCode::Delete, &mut || {
            if input.cursor_position < input.text.len() {
                let mut chars: Vec<char> = input.text.chars().collect();
                chars.remove(input.cursor_position);
                input.text = chars.into_iter().collect();
            }
        });
    }
}

fn transfer_text_input(mut text_query: Query<(&mut Text, &TextInput), Changed<TextInput>>) {
    for (mut text, input) in text_query.iter_mut() {
        if let Some(section) = text.sections.last_mut() {
            section.value = input.text.clone();
        }
    }
}

fn handle_input_field_placeholder(
    mut text_query: Query<(&mut Text, &TextInput), Changed<TextInput>>,
) {
    for (mut text, input) in text_query.iter_mut() {
        let idx = text.sections.len() - 2;
        if let Some(mut placeholder_section) = text.sections.get_mut(idx) {
            if input.text.is_empty() {
                placeholder_section.value = input.placeholder.clone();
            } else {
                placeholder_section.value = "".into();
            }
        }
    }
}

pub struct TextInputPlugin;

impl Plugin for TextInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FocusSwitchedEvent<TextInput>>()
            .add_system(handle_text_input_addition.run_if(not(in_state(MenuState::Disabled))))
            .add_systems(
                (
                    handle_text_input.run_if(not(in_state(MenuState::Disabled))),
                    transfer_text_input.run_if(not(in_state(MenuState::Disabled))),
                    handle_input_field_placeholder.run_if(not(in_state(MenuState::Disabled))),
                    handle_text_input_new_focus.run_if(not(in_state(MenuState::Disabled))),
                    remove_focus_from_non_focused_entities::<TextInput>
                        .run_if(not(in_state(MenuState::Disabled))),
                )
                    .after(handle_text_input_addition),
            );
    }
}
