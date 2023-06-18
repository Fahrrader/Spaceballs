use crate::ui::{remove_focus_from_non_focused_entities, Focus, FocusSwitchedEvent};
use crate::MenuState;
use bevy::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use clipboard::{ClipboardContext, ClipboardProvider};
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

    pub fn insert(&mut self, ch: char) -> bool {
        if self.text.len() < self.max_symbols {
            // Could be unsafe due to some characters being more than 1 byte long
            self.text.insert(self.cursor_position, ch);
            self.cursor_position += 1;
            true
        } else {
            false
        }
    }

    pub fn insert_string<S: AsRef<str>>(&mut self, s: S) -> bool {
        let s = s.as_ref();
        let remaining_capacity = self.max_symbols.saturating_sub(self.text.chars().count());
        let insertion = s.chars().take(remaining_capacity).collect::<String>();
        let inserted_len = insertion.chars().count();

        if inserted_len == 0 {
            return false;
        }

        self.text.insert_str(self.cursor_position, &insertion);
        self.cursor_position += inserted_len;

        true
    }

    pub fn delete(&mut self, steps: isize) -> String {
        if self.cursor_position != 0 || steps < 0 {
            let mut chars: Vec<char> = self.text.chars().collect();
            let (start, end) = if steps > 0 {
                (
                    self.cursor_position - steps.min(self.cursor_position as isize) as usize,
                    self.cursor_position,
                )
            } else {
                (
                    self.cursor_position,
                    (self.cursor_position as isize - steps).min(chars.len() as isize) as usize,
                )
            };

            let deleted_range: String = chars.drain(start..end).collect();
            self.text = chars.into_iter().collect();
            self.cursor_position = start;
            deleted_range
        } else {
            String::new()
        }
    }

    pub fn reset_text(&mut self) -> String {
        self.cursor_position = 0;
        std::mem::take(&mut self.text)
    }

    pub fn shift_cursor_left(&mut self, steps: usize) {
        self.cursor_position = self.cursor_position.saturating_sub(steps);
    }

    pub fn shift_cursor_right(&mut self, steps: usize) {
        self.cursor_position = (self.cursor_position + steps).min(self.text.len());
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn copy_to_clipboard(contents: String) {
    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
    ctx.set_contents(contents).unwrap();
}

#[cfg(not(target_arch = "wasm32"))]
fn paste_from_clipboard() -> String {
    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
    match ctx.get_contents() {
        Ok(contents) => contents,
        Err(_) => String::new(), // Handle error (empty string in this case)
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
            _ => {}
        }
    }
}

#[derive(Default)]
struct KeyHandler {
    timer: Timer,
    handled: bool,
}

impl KeyHandler {
    const KEY_PRESS_INITIAL_TIMEOUT: f32 = 0.5;
    const KEY_PRESS_TIMEOUT: f32 = 0.05;

    pub fn press_with_timeout(
        &mut self,
        key: KeyCode,
        action: &mut dyn FnMut(),
        (keys, time): (&Input<KeyCode>, &Time),
    ) {
        let mut new_duration = None;
        if keys.just_pressed(key) {
            action();
            new_duration = Some(Duration::from_secs_f32(Self::KEY_PRESS_INITIAL_TIMEOUT));
        } else if keys.pressed(key) && self.timer.tick(time.delta()).just_finished() {
            action();
            new_duration = Some(Duration::from_secs_f32(Self::KEY_PRESS_TIMEOUT));
        }

        if let Some(duration) = new_duration {
            self.timer.set_duration(duration);
            self.timer.reset();
            self.handled = true;
        } else if keys.just_released(key) {
            self.timer.reset();
            self.handled = true;
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn handle_text_input(
    mut text_query: Query<(&mut TextInput, &Focus<TextInput>)>,
    mut characters_evs: EventReader<ReceivedCharacter>,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut key_handler: Local<KeyHandler>,
) {
    for (mut input, focus) in text_query.iter_mut() {
        if focus.is_none() {
            continue;
        }

        key_handler.handled = false;

        key_handler.press_with_timeout(
            KeyCode::Return,
            &mut || {
                input.insert('\n');
            },
            (&keys, &time),
        );

        key_handler.press_with_timeout(
            KeyCode::Left,
            &mut || input.shift_cursor_left(1),
            (&keys, &time),
        );

        key_handler.press_with_timeout(
            KeyCode::Right,
            &mut || input.shift_cursor_right(1),
            (&keys, &time),
        );

        key_handler.press_with_timeout(
            KeyCode::Back,
            &mut || {
                input.delete(1);
            },
            (&keys, &time),
        );

        key_handler.press_with_timeout(
            KeyCode::Delete,
            &mut || {
                input.delete(-1);
            },
            (&keys, &time),
        );

        if keys.pressed(KeyCode::LControl) || keys.pressed(KeyCode::RControl) {
            if keys.just_pressed(KeyCode::C) {
                copy_to_clipboard(input.text.clone());
                key_handler.handled = true;
            }

            if keys.just_pressed(KeyCode::X) {
                copy_to_clipboard(input.reset_text());
                key_handler.handled = true;
            }

            key_handler.press_with_timeout(
                KeyCode::V,
                &mut || {
                    input.insert_string(paste_from_clipboard());
                },
                (&keys, &time),
            );
        }

        if !key_handler.handled {
            for ev in characters_evs.iter() {
                if ev.char.is_ascii_graphic() || ev.char.is_ascii_whitespace() {
                    input.insert(ev.char);
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn handle_text_input() {}

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
