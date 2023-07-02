use crate::network::players::PlayerRegistry;
use crate::network::PlayerHandle;
use crate::ui::{fonts, menu_builder};
use crate::GameState;
use bevy::prelude::*;
use std::time::Duration;

pub const MAX_CHAT_MESSAGES: usize = 32;

#[derive(Component, Clone, Debug)]
pub struct ChatMessage {
    pub message: String,
    pub player_handles: Vec<PlayerHandle>,
    // timestamp?
}

impl ChatMessage {
    pub fn message(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            player_handles: vec![],
        }
    }

    pub fn separate_handles(&self) -> Vec<String> {
        let mut parts: Vec<String> = Vec::new();

        let mut start = 0;
        while let Some(begin) = self.message[start..].find('{') {
            if let Some(end) = self.message[start + begin..].find('}') {
                if let Ok(idx) =
                    self.message[start + begin + 1..start + begin + end].parse::<usize>()
                {
                    if let Some(player_handle) = self.player_handles.get(idx) {
                        parts.push(self.message[start..start + begin].to_string());
                        parts.push(player_handle.to_string());
                        start += begin + end + 1;
                        continue;
                    }
                }
            }
            break;
        }

        if start < self.message.len() {
            parts.push(self.message[start..].to_string());
        }

        parts
    }
}

#[derive(Component)]
pub struct ChatDisplay;

fn setup_chat_display(mut commands: Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(35.0), Val::Percent(20.0)),
                position_type: PositionType::Absolute,
                position: UiRect {
                    right: Val::Px(20.0),
                    top: Val::Px(20.0),
                    ..default()
                },
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                justify_content: JustifyContent::End,
                overflow: Overflow::Hidden,
                ..default()
            },
            ..default()
        },
        ChatDisplay,
    ));
}

fn handle_new_chat_messages(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut new_messages: EventReader<ChatMessage>,
    players: Option<Res<PlayerRegistry>>,
    chat_display_query: Query<(Entity, Option<&Children>), With<ChatDisplay>>,
) {
    if new_messages.is_empty() {
        return;
    }

    const CHAT_FONT_SIZE: f32 = 14.0;

    let text_style = TextStyle {
        font: fonts::load(&asset_server, fonts::ULTRAGONIC),
        font_size: CHAT_FONT_SIZE,
        color: Color::WHITE.with_a(0.8),
    };

    let (chat_display_entity, chat_children) = chat_display_query
        .get_single()
        .expect("Failed to fetch singular chat display");
    let mut children_pushed = 0;

    for message in new_messages.iter() {
        let name_style = TextStyle {
            font: fonts::load(&asset_server, fonts::ULTRAGONIC),
            font_size: CHAT_FONT_SIZE,
            color: menu_builder::DEFAULT_TEXT_COLOR.with_a(0.8),
        };

        // Parse the chat message in case it contains any players handles, in which case we want to prettify them
        let texts = message
            .separate_handles()
            .iter()
            .enumerate()
            .map(|(i, piece)| {
                // `ChatMessage`'s `separate_handles` leaves player handles at odd indices
                if i % 2 == 1 {
                    let player_name = players
                        .as_ref()
                        .and_then(|registry| registry.get(piece.parse::<usize>().ok()?))
                        .map(|data| data.name.clone())
                        .unwrap_or("{player}".into());
                    TextSection::new(player_name, name_style.clone())
                } else {
                    TextSection::new(piece, text_style.clone())
                }
            })
            .collect::<Vec<_>>();
        let text = Text::from_sections(texts);

        if chat_children.map(|children| children.len()).unwrap_or(0) < MAX_CHAT_MESSAGES {
            // Create a new chat message entity until we've reached the limit
            commands
                .entity(chat_display_entity)
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle {
                            text,
                            style: Style {
                                // this shit will stay here at least until bevy 0.11
                                size: Size::new(Val::Px(0.35 * 800.), Val::Auto),
                                margin: UiRect::top(Val::Px(5.)),
                                ..default()
                            },
                            ..default()
                        },
                        message.clone(),
                        ChatFadeout::new(),
                    ));
                });
        } else {
            // Reuse oldest message entity, push from the front to the end
            let old_message_entity = chat_children.expect("Expected message limit to be above zero -- and to have children to encounter that limit")[children_pushed];
            commands
                .entity(chat_display_entity)
                .add_child(old_message_entity);
            children_pushed += 1;

            commands
                .entity(old_message_entity)
                .insert(text)
                .insert(message.clone())
                .insert(ChatFadeout::new());
        }
    }
}

#[derive(Component)]
pub struct ChatFadeout {
    pub timer: Timer,
    pub is_fading: bool,
}

impl ChatFadeout {
    pub const DURATION_BEFORE_FADEOUT: Duration = Duration::from_millis(3000);

    pub fn new() -> Self {
        Self {
            timer: Timer::new(Self::DURATION_BEFORE_FADEOUT, TimerMode::Once),
            is_fading: true,
        }
    }
}

fn handle_chat_message_fadeout(
    time: Res<Time>,
    mut query: Query<(&mut Text, &mut Visibility, &mut ChatFadeout)>,
) {
    // about 90 frames
    const FADEOUT_PER_FRAME: f32 = 0.95;
    const FADEOUT_THRESHOLD: f32 = 0.01;

    for (mut text, mut visibility, mut fadeout) in query.iter_mut() {
        fadeout.timer.tick(time.delta());

        if matches!(*visibility, Visibility::Hidden) {
            continue;
        }

        if fadeout.timer.finished() && fadeout.is_fading {
            let mut has_faded_out = true;

            for section in text.sections.iter_mut() {
                let a = section.style.color.a();
                section.style.color.set_a(a * FADEOUT_PER_FRAME);
                has_faded_out &= section.style.color.a() <= FADEOUT_THRESHOLD;
            }

            if has_faded_out {
                fadeout.timer.pause();
                *visibility = Visibility::Hidden;
            }
        }
    }
}

fn reset_fadeout_on_chat_open(
    // replace ChatDisplay with bigger picture
    chat_display_query: Query<&Visibility, (Changed<Visibility>, With<ChatDisplay>)>,
    mut query: Query<(&mut Text, &mut ChatFadeout)>,
) {
    if chat_display_query
        .iter()
        .any(|visibility| matches!(*visibility, Visibility::Visible))
    {
        for (mut text, mut fadeout) in query.iter_mut() {
            fadeout.timer.unpause();
            fadeout.timer.reset();
            fadeout.is_fading = false;
            for section in text.sections.iter_mut() {
                section.style.color.set_a(1.0);
            }
        }
    }
}

fn mock_message_sending(
    mut messenger: EventWriter<ChatMessage>,
    keyboard: Res<Input<KeyCode>>,
    messages: Query<With<ChatMessage>>,
    mut n_messages: Local<usize>,
) {
    if keyboard.just_pressed(KeyCode::M) {
        *n_messages += 2;
        messenger.send(ChatMessage::message(messages.iter().len().to_string()));
        messenger.send(ChatMessage::message(format!(
            "{}: I'm a businessman with a business plan",
            *n_messages - 1
        )));
        messenger.send(ChatMessage::message(format!(
            "{}: I'm gonna make you money in business land",
            *n_messages
        )));
    }
}

pub struct ChatPlugin;
impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ChatMessage>()
            .add_system(setup_chat_display.in_schedule(OnExit(GameState::MainMenu)))
            .add_system(handle_new_chat_messages.run_if(not(in_state(GameState::MainMenu))))
            .add_system(handle_chat_message_fadeout.run_if(not(in_state(GameState::MainMenu))))
            .add_system(reset_fadeout_on_chat_open.run_if(not(in_state(GameState::MainMenu))))
            .add_system(mock_message_sending.run_if(not(in_state(GameState::MainMenu))));
    }
}
