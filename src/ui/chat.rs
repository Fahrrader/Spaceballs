use crate::network::peers::PeerMessage;
use crate::network::players::PlayerRegistry;
use crate::network::PlayerHandle;
use crate::ui::focus::Focus;
use crate::ui::input_consumption::{ActiveInputConsumerLayers, InputConsumerPriority};
use crate::ui::text_input::TextInput;
use crate::ui::{fonts, menu_builder};
use crate::GameState;
use bevy::prelude::*;
use std::time::Duration;

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

    pub fn by(mut self, whom: String) -> Self {
        self.message = format!("{}: {}", whom, self.message);
        self
    }

    pub fn by_player(mut self, handle: PlayerHandle) -> Self {
        self.player_handles = vec![handle];
        self.by("{0}".into())
    }

    pub fn separate_handles(&self) -> Vec<String> {
        let mut parts: Vec<String> = Vec::new();

        let pieces: Vec<&str> = self.message.split(|c| c == '{' || c == '}').collect();
        for (idx, piece) in pieces.iter().enumerate() {
            // Every even index is outside of curly braces
            if idx % 2 == 0 {
                parts.push(piece.to_string());
            } else if *piece == "You" {
                parts.push(piece.to_string());
            } else if let Ok(idx) = piece.parse::<usize>() {
                if let Some(player_handle) = self.player_handles.get(idx) {
                    parts.push(player_handle.to_string());
                }
            } else {
                parts.push(format!("{{{}}}", piece));
            }
        }

        parts
    }
}

#[derive(Component)]
pub struct ChatDisplay;

#[derive(Component)]
pub struct ChatMessagesDisplay;

#[derive(Component)]
pub struct ChatTypingDisplay;

#[derive(Component)]
pub struct ChatDisplayBackground;

pub const CHAT_INPUT_LAYER: InputConsumerPriority = InputConsumerPriority::new(7);
pub const CHAT_OPEN_INPUT_LAYER: InputConsumerPriority = InputConsumerPriority::new(6);

// this shit will stay here at least until bevy 0.11
const CHAT_WIDTH: f32 = 0.35 * 800.;

fn setup_chat_display(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
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
                    justify_content: JustifyContent::Start,
                    ..default()
                },
                ..default()
            },
            ChatDisplay,
        ))
        .with_children(|parent| {
            parent.spawn((
                NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(85.0)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Start,
                        justify_content: JustifyContent::End,
                        overflow: Overflow::Hidden,
                        ..default()
                    },
                    ..default()
                },
                ChatMessagesDisplay,
            ));

            let input_style = TextStyle {
                font: fonts::load(&asset_server, fonts::ULTRAGONIC),
                font_size: 14.0,
                color: Color::WHITE.with_a(0.8),
            };

            parent.spawn((
                TextBundle {
                    style: Style {
                        size: Size::new(Val::Px(CHAT_WIDTH), Val::Auto),
                        align_items: AlignItems::Start,
                        justify_content: JustifyContent::End,
                        ..default()
                    },
                    background_color: Color::DARK_GRAY.with_a(0.4).into(),
                    visibility: Visibility::Hidden,
                    text: Text::from_section("Chat: ", input_style.clone()),
                    ..default()
                },
                TextInput::new("".into(), input_style, Some("Send them a message".into())),
                Focus::<TextInput>::None,
                CHAT_INPUT_LAYER,
                ChatTypingDisplay,
            ));

            parent.spawn((
                NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(85.0)),
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    background_color: Color::DARK_GRAY.with_a(0.2).into(),
                    visibility: Visibility::Hidden,
                    ..default()
                },
                ChatDisplayBackground,
            ));
        });
}

fn handle_new_chat_messages(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut new_messages: EventReader<ChatMessage>,
    players: Res<PlayerRegistry>,
    chat_display_query: Query<(Entity, Option<&Children>), With<ChatMessagesDisplay>>,
) {
    if new_messages.is_empty() {
        return;
    }

    const MAX_CHAT_MESSAGES: usize = 16;
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

    let you_style = TextStyle {
        font: fonts::load(&asset_server, fonts::ULTRAGONIC),
        font_size: CHAT_FONT_SIZE,
        color: menu_builder::DEFAULT_TEXT_COLOR.with_a(0.8),
    };

    let name_style = you_style.clone();

    for message in new_messages.iter() {
        // Parse the chat message in case it contains any players handles, in which case we want to prettify them
        let texts = message
            .separate_handles()
            .iter()
            .enumerate()
            .map(|(i, piece)| {
                // `ChatMessage`'s `separate_handles` leaves player handles at odd indices
                if i % 2 == 1 {
                    match piece.as_str() {
                        "You" => TextSection::new("You", you_style.clone()),
                        _ => {
                            let player_name = piece
                                .parse::<usize>()
                                .ok()
                                .and_then(|id| players.get(id))
                                .map(|data| data.name.clone())
                                .unwrap_or("[unknown]".to_string());
                            TextSection::new(player_name, name_style.clone())
                        }
                    }
                } else {
                    TextSection::new(piece, text_style.clone())
                }
            })
            .collect::<Vec<_>>();
        let text = Text::from_sections(texts);

        if chat_children
            .map(|children| children.len() < MAX_CHAT_MESSAGES)
            .unwrap_or(true)
        {
            // Create a new chat message entity until we've reached the limit
            commands
                .entity(chat_display_entity)
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle {
                            text,
                            style: Style {
                                size: Size::new(Val::Px(CHAT_WIDTH), Val::Auto),
                                margin: UiRect::all(Val::Px(2.5)),
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
                .insert(ChatFadeout::new())
                .insert(Visibility::Visible);
        }
    }
}

#[derive(Resource)]
pub struct ChatIsFading {
    pub is_fading: bool,
}

impl ChatIsFading {
    pub fn set_to_fade(&mut self) {
        self.is_fading = true;
    }

    pub fn set_to_not_fade(&mut self) {
        self.is_fading = false;
    }
}

#[derive(Component)]
pub struct ChatFadeout {
    pub timer: Timer,
}

impl ChatFadeout {
    pub const DURATION_BEFORE_FADEOUT: Duration = Duration::from_millis(3000);

    pub fn new() -> Self {
        Self {
            timer: Timer::new(Self::DURATION_BEFORE_FADEOUT, TimerMode::Once),
        }
    }
}

fn handle_chat_opening(
    keyboard: Res<Input<KeyCode>>,
    input_consumers: Res<ActiveInputConsumerLayers>,
    mut chat_typing_query: Query<&mut Focus<TextInput>, With<ChatTypingDisplay>>,
    mut chat_visibility_query: Query<
        &mut Visibility,
        (
            Or<(With<ChatDisplayBackground>, With<ChatTypingDisplay>)>,
            Without<ChatFadeout>,
        ),
    >,
    mut message_query: Query<(&mut Text, &mut ChatFadeout, &mut Visibility)>,
    mut chat_fade: ResMut<ChatIsFading>,
) {
    if input_consumers.is_input_blocked_for_layer(&CHAT_OPEN_INPUT_LAYER) {
        return;
    }

    if keyboard.any_just_pressed([KeyCode::T, KeyCode::Return]) {
        for mut focus in chat_typing_query.iter_mut() {
            *focus = Focus::Focused(None);
        }

        for mut visibility in chat_visibility_query.iter_mut() {
            *visibility = Visibility::default();
        }

        chat_fade.set_to_not_fade();

        for (mut text, mut fadeout, mut visibility) in message_query.iter_mut() {
            fadeout.timer.unpause();
            fadeout.timer.reset();
            for section in text.sections.iter_mut() {
                section.style.color.set_a(1.0);
            }
            *visibility = Visibility::default();
        }
    }
}

fn handle_chat_sending(
    keyboard: Res<Input<KeyCode>>,
    input_consumers: Res<ActiveInputConsumerLayers>,
    mut chat_typing_query: Query<
        (&Text, &mut TextInput, &mut Focus<TextInput>),
        With<ChatTypingDisplay>,
    >,
    mut chat_visibility_query: Query<
        &mut Visibility,
        Or<(With<ChatDisplayBackground>, With<ChatTypingDisplay>)>,
    >,
    mut chat_fade: ResMut<ChatIsFading>,
    mut messenger: EventWriter<ChatMessage>,
    mut broadcaster: EventWriter<PeerMessage>,
) {
    if input_consumers.is_layer_active(&CHAT_INPUT_LAYER) {
        let return_pressed = keyboard.just_pressed(KeyCode::Return);
        let escape_pressed = keyboard.just_pressed(KeyCode::Escape);

        if return_pressed || escape_pressed {
            for (text, mut input, mut focus) in chat_typing_query.iter_mut() {
                if focus.is_none() {
                    continue;
                }

                if return_pressed {
                    if let Some(message) = text.sections.last() {
                        if !message.value.is_empty() {
                            let message = message.value.trim_end().to_string();
                            messenger
                                .send(ChatMessage::message(message.clone()).by("{You}".into()));
                            broadcaster.send(PeerMessage::Chat { message });
                            input.reset_text();
                        }
                    }
                }

                *focus = Focus::None;
            }

            chat_visibility_query.for_each_mut(|mut vis| *vis = Visibility::Hidden);
            chat_fade.set_to_fade();
        }
    }
}

fn handle_chat_message_fadeout(
    time: Res<Time>,
    chat_fade: Res<ChatIsFading>,
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

        if fadeout.timer.finished() && chat_fade.is_fading {
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

pub struct ChatPlugin;
impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ChatMessage>()
            .insert_resource(ChatIsFading { is_fading: true })
            .add_system(setup_chat_display.in_schedule(OnExit(GameState::MainMenu)))
            .add_system(handle_new_chat_messages.run_if(not(in_state(GameState::MainMenu))))
            .add_system(handle_chat_opening.run_if(not(in_state(GameState::MainMenu))))
            .add_system(handle_chat_sending.run_if(not(in_state(GameState::MainMenu))))
            .add_system(handle_chat_message_fadeout.run_if(not(in_state(GameState::MainMenu))));
    }
}
