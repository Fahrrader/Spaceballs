use crate::network::PlayerCount;
use crate::ui::color_interaction::ColorInteractionMap;
use crate::ui::focus::{Focus, KeyToButtonBinding};
use crate::ui::input_consumption::{MATCH_END_INPUT_LAYER, PAUSE_INPUT_LAYER};
use crate::ui::lobby::PeerWaitingText;
use crate::ui::menu_builder::{
    DEFAULT_BUTTON_COLOR, DEFAULT_BUTTON_HOVERED_COLOR, DEFAULT_BUTTON_PRESSED_COLOR,
    DEFAULT_FONT_SIZE, DEFAULT_OUTLINE_THICKNESS, DEFAULT_TEXT_COLOR, DEFAULT_TEXT_INPUT_MARGIN,
};
use crate::ui::score::{TotalScoreDisplay, VictoryText};
use crate::ui::text_input::TextInput;
use crate::ui::user_settings::{transfer_setting_from_text_input, UserInputForm, UserSettings};
use crate::ui::{colors, despawn_node, fonts};
use crate::{build_menu_plugin, GamePauseEvent, GameState, SceneSelector};
#[cfg(not(target_arch = "wasm32"))]
use bevy::app::AppExit;
use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;

macro_rules! generate_menu_states {
    ($($state:ident),* $(,)?) => {
        /// State used for the current menu screen.
        #[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
        pub enum MenuState {
            $($state,)*
            #[default]
            Disabled,
        }

        mod menu_state {
            $(
                pub enum $state {}
            )*
        }
    }
}

generate_menu_states!(
    Main,
    SinglePlayer,
    MultiPlayer,
    MatchMaker,
    // MatchBrowser,
    // Test,
    Controls,
    // Tutorial,
    Settings,
    Pause,
    MatchEnd,
    MatchmakingLobby,
);

/// Tag component used to tag entities as children on a generic menu screen -- those that should also be despawned when the screen is exited.
#[derive(Component)]
struct OnMenu<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T> Default for OnMenu<T> {
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

/// Plugin that handles spawning and despawning of a single menu that has a component [`T`].
struct SingleMenuPlugin<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T> Default for SingleMenuPlugin<T> {
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

/// All actions that can be triggered from a button click.
#[derive(Component)]
pub(crate) enum MenuButtonAction {
    SinglePlayer,
    MultiPlayer,
    // Quickmatch?
    // Match Browser?
    JoinGame,
    // HostGame,
    SelectScene(SceneSelector),
    StartGame,
    Resume,
    Rematch,
    Controls,
    Settings,
    BackToMenu,
    QuitToTitle,
    #[cfg(not(target_arch = "wasm32"))]
    Quit,
}

build_menu_plugin!(
    (setup_main_menu, Main),
    once align_items = AlignItems::Start.into(),
    once layout_height = Val::Percent(42.5).into(),
    Column {
        text_font_size = 60.0,
        once margin = UiRect::all(Val::Px(0.)).into(),
        Text [
            "Cosmic\n",
            once text_font_size = 72.0,
            once text_color = colors::LEMON,
            once font = fonts::SPACERUNNER,
            "Spaceball\n",
            "Tactical Action Arena",
        ] + (
            ColorInteractionMap::from([
                (Interaction::None, Some(DEFAULT_TEXT_COLOR.with_a(0.99))),
                // (Interaction::Hovered, Some(DEFAULT_TEXT_COLOR.with_a(0.99))),
                (Interaction::Clicked, Some(colors::LEMON)),
            ]),
            Interaction::None,
            crate::easter::EasterAnnouncerActivator,
        ),
    },
    Bottom {
        Column {
            Buttons [
                (MenuButtonAction::SinglePlayer, "Singleplayer"),
                (MenuButtonAction::MultiPlayer, "Multiplayer"),
            ],
            {
                button_color = colors::NEON_PINK.into(),
                button_text_hovered_color = Some(colors::LEMON.into()),
                button_font_size = 24.0,
                Buttons [
                    (MenuButtonAction::Controls, "Controls"),
                    (MenuButtonAction::Settings, "Settings"),
                ],
            },
            #[cfg(not(target_arch = "wasm32"))]
            {
                Buttons [
                    (MenuButtonAction::Quit, "Quit") + (
                        KeyToButtonBinding(KeyCode::Escape)
                    ),
                ],
            },
        },
    },
);

build_menu_plugin!(
    (setup_pause_menu, Pause, PAUSE_INPUT_LAYER),
    once layout_height = Val::Percent(100.).into(),
    once layout_width = Val::Percent(50.).into(),
    justify_content = JustifyContent::Start.into(),
    once align_items = AlignItems::Center.into(),
    button_width = Val::Px(380.),
    button_font_size = 30.,
    button_margin = UiRect::all(Val::Px(2.)),
    outline_width = Val::Px(0.),
    Left {
        once node_color = Color::DARK_GRAY.with_a(0.2),
        once layout_height = Val::Px(420.).into(),
        once layout_width = Val::Px(400.).into(),
        once justify_content = JustifyContent::Center.into(),
        Node {
            once align_items = AlignItems::Start.into(),
            once justify_content = JustifyContent::Center.into(),
            once margin = UiRect::all(Val::Px(10.)).into(),
            Column {
                once text_color = colors::ORCHID.into(),
                Text [
                    "Menu",
                ],
                Buttons [
                    (MenuButtonAction::Resume, "Resume"),
                    (MenuButtonAction::Controls, "Controls"),
                    (MenuButtonAction::QuitToTitle, "Quit to Main Menu"),
                ],
                #[cfg(not(target_arch = "wasm32"))]
                {
                    Buttons [
                        (MenuButtonAction::Quit, "Quit"),
                    ],
                },
            },
        },
    },
);

fn setup_match_end_menu(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut total_score_display_query: Query<
        (&mut Style, &mut BackgroundColor),
        With<TotalScoreDisplay>,
    >,
) {
    // aligning with the top block
    const MATCH_END_SCREEN_TOP: f32 = 21.875;
    const MATCH_END_SCREEN_RIGHT: f32 = (100. - MATCH_END_SCREEN_WIDTH) / 2.;
    const MATCH_END_SCREEN_HEIGHT: f32 = 70.0;
    const MATCH_END_SCREEN_WIDTH: f32 = 47.5;
    const VICTOR_NAMEPLATE_HEIGHT: f32 = 20.0;
    // which is 50% with [`MATCH_END_SCREEN_HEIGHT`] == 70.0
    const SCORE_PANEL_HEIGHT: f32 = 35.0 / MATCH_END_SCREEN_HEIGHT * 100.0;
    const BUTTON_PANEL_HEIGHT: f32 = 30.0;

    if let Ok((mut score_display, mut score_display_color)) =
        total_score_display_query.get_single_mut()
    {
        score_display.size.width = Val::Percent(MATCH_END_SCREEN_WIDTH);
        score_display.position.right = Val::Percent(MATCH_END_SCREEN_RIGHT);
        score_display.position.top = Val::Percent(
            MATCH_END_SCREEN_TOP + MATCH_END_SCREEN_HEIGHT / 100. * VICTOR_NAMEPLATE_HEIGHT,
        );
        *score_display_color = Color::NONE.into();
    }

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(
                        Val::Percent(MATCH_END_SCREEN_WIDTH),
                        Val::Percent(MATCH_END_SCREEN_HEIGHT),
                    ),
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        right: Val::Percent(MATCH_END_SCREEN_RIGHT),
                        top: Val::Percent(MATCH_END_SCREEN_TOP),
                        ..default()
                    },
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    align_self: AlignSelf::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                // complementary to [`DEFAULT_TEXT_COLOR`]
                background_color: Color::rgb(0.212, 0., 0.102).with_a(0.5).into(),
                ..default()
            },
            MATCH_END_INPUT_LAYER,
        ))
        .with_children(|parent| {
            let victor_name_style = TextStyle {
                font: fonts::load(&asset_server, fonts::SPACERUNNER),
                font_size: 45.,
                color: DEFAULT_TEXT_COLOR,
            };
            let victor_verb_style = TextStyle {
                font: fonts::load(&asset_server, fonts::SPACERUNNER),
                font_size: 38.,
                color: DEFAULT_TEXT_COLOR,
            };

            // winner's name space
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(VICTOR_NAMEPLATE_HEIGHT)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle {
                            text: Text::from_sections([
                                TextSection {
                                    value: "You".to_string(),
                                    style: victor_name_style.clone(),
                                },
                                TextSection {
                                    value: "\nwin!".to_string(),
                                    style: victor_verb_style,
                                },
                            ])
                            .with_alignment(TextAlignment::Center),
                            ..default()
                        },
                        VictoryText,
                    ));
                });

            // empty column that is space to house score display
            parent.spawn(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(SCORE_PANEL_HEIGHT)),
                    ..default()
                },
                ..default()
            });

            // button column
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(BUTTON_PANEL_HEIGHT)),
                        flex_direction: FlexDirection::ColumnReverse,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    let button_bundle = ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Percent(100.0), Val::Px(50.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::NONE.into(),
                        ..default()
                    };
                    let button_text_style = TextStyle {
                        font_size: 30.,
                        color: DEFAULT_BUTTON_COLOR,
                        font: fonts::load(&asset_server, fonts::ULTRAGONIC),
                    };
                    let color_interaction_map = ColorInteractionMap::from([
                        (Interaction::None, Some(DEFAULT_BUTTON_COLOR)),
                        (Interaction::Hovered, Some(DEFAULT_BUTTON_HOVERED_COLOR)),
                        (Interaction::Clicked, Some(DEFAULT_BUTTON_PRESSED_COLOR)),
                    ]);
                    let empty_color_interaction_map = ColorInteractionMap::from([]);

                    parent
                        .spawn((
                            button_bundle.clone(),
                            Focus::<Interaction>::None,
                            empty_color_interaction_map.clone(),
                            MenuButtonAction::Quit,
                        ))
                        .with_children(|button| {
                            button.spawn((
                                TextBundle {
                                    text: Text::from_section("Quit", button_text_style.clone()),
                                    ..default()
                                },
                                color_interaction_map.clone(),
                            ));
                        });

                    parent
                        .spawn((
                            button_bundle.clone(),
                            Focus::<Interaction>::None,
                            empty_color_interaction_map.clone(),
                            MenuButtonAction::QuitToTitle,
                            KeyToButtonBinding(KeyCode::Escape),
                        ))
                        .with_children(|button| {
                            button.spawn((
                                TextBundle {
                                    text: Text::from_section(
                                        "Quit to Main Menu",
                                        button_text_style.clone(),
                                    ),
                                    ..default()
                                },
                                color_interaction_map.clone(),
                            ));
                        });

                    parent
                        .spawn((
                            button_bundle,
                            Focus::<Interaction>::None,
                            empty_color_interaction_map,
                            MenuButtonAction::Rematch,
                        ))
                        .with_children(|button| {
                            button.spawn((
                                TextBundle {
                                    text: Text::from_section("Rematch", button_text_style),
                                    ..default()
                                },
                                color_interaction_map,
                            ));
                        });
                });
        });
}

impl Plugin for SingleMenuPlugin<menu_state::MatchEnd> {
    fn build(&self, app: &mut App) {
        app.add_system(setup_match_end_menu.in_schedule(OnEnter(MenuState::MatchEnd)))
            .add_system(
                despawn_node::<OnMenu<menu_state::MatchEnd>>
                    .in_schedule(OnExit(MenuState::MatchEnd)),
            );
    }
}

build_menu_plugin!(
    (setup_singleplayer_menu, SinglePlayer),
    once align_self = AlignSelf::Start.into(),
    once layout_height = Val::Percent(50.).into(),
    Column {
        Column {
            Text [
                "Singleplayer",
            ],
        },
        once align_self = AlignSelf::Start.into(),
        once layout_height = Val::Percent(90.).into(),
        Column {
            Node {
                button_width = Val::Px(330.0),
                button_height = Val::Px(165.0),
                Buttons [
                    (MenuButtonAction::SelectScene(SceneSelector::Main), "Scene\nMain"),
                    (MenuButtonAction::SelectScene(SceneSelector::Experimental), "Scene\nExperimental"),
                ],
            },
        },
    },
    Bottom {
        Column {
            Buttons [
                (MenuButtonAction::StartGame, "Start Game"),
                (MenuButtonAction::BackToMenu, "Back") + (
                    KeyToButtonBinding(KeyCode::Escape)
                ),
            ],
        },
    },
);

build_menu_plugin!(
    (setup_multiplayer_menu(user_settings: Res<UserSettings>), MultiPlayer),
    Top {
        Column {
            Text [ "Multiplayer", ],
        },
    },
    once layout_height = Val::Percent(50.).into(),
    once layout_width = Val::Percent(75.).into(),
    Column {
        once layout_height = Val::Percent(90.).into(),
        once layout_width = Val::Percent(100.).into(),
        once margin = UiRect::top(Val::Percent(3.)).into(),
        Column {
            margin = UiRect::all(Val::Px(7.5)).into(),
            layout_width = Val::Percent(100.).into(),
            Column {
                margin = UiRect::all(Val::Px(DEFAULT_OUTLINE_THICKNESS * 0.5)).into(),
                Text [ "Player name", ],
                button_height = Val::Px(DEFAULT_FONT_SIZE + DEFAULT_TEXT_INPUT_MARGIN * 2.),
                // font = fonts::FIRA_SANS,
                TextInput [
                    max_symbols: 24,
                    placeholder: "Anata no namae wa..?",
                    user_settings.player_name.clone(),
                ] + (
                    UserInputForm::PlayerName,
                    Focus::<TextInput>::Focused(None),
                ),
            },
            Column {
                margin = UiRect::all(Val::Px(DEFAULT_OUTLINE_THICKNESS * 0.5)).into(),
                Text [ "Server URL", ],
                once node_color = Color::TOMATO.with_a(0.3),
                // stupid fucking text doesn't wrap around properly if not specified in pixels
                button_width = Val::Percent(100.),
                button_height = Val::Px(3. * DEFAULT_FONT_SIZE + DEFAULT_TEXT_INPUT_MARGIN * 2.),
                // font = fonts::FIRA_SANS,
                TextInput [
                    placeholder: "URL of the connecting server",
                    user_settings.server_url.clone(),
                ] + (
                    UserInputForm::ServerUrl,
                ),
            },
            Column {
                margin = UiRect::all(Val::Px(DEFAULT_OUTLINE_THICKNESS * 0.5)).into(),
                Text [ "Room name", ],
                button_height = Val::Px(DEFAULT_FONT_SIZE + DEFAULT_TEXT_INPUT_MARGIN * 2.),
                // font = fonts::FIRA_SANS,
                TextInput [
                    placeholder: "",
                    max_symbols: 24,
                    user_settings.room_name.clone(),
                ] + (
                    UserInputForm::RoomName,
                ),
            },
        },
    },
    Bottom {
        Column {
            Buttons [
                (MenuButtonAction::JoinGame, "Continue"),
                (MenuButtonAction::BackToMenu, "Back") + (
                    KeyToButtonBinding(KeyCode::Escape)
                ),
            ],
        },
    },
);

build_menu_plugin!(
    (setup_multiplayer_creation_menu, MatchMaker),
    once align_self = AlignSelf::Start.into(),
    once layout_height = Val::Percent(50.).into(),
    Column {
        Column {
            Text [
                "Matchmaking",
            ],
        },
        once align_self = AlignSelf::Start.into(),
        once layout_height = Val::Percent(90.).into(),
        Column {
            Node {
                button_width = Val::Px(330.0),
                button_height = Val::Px(165.0),
                Buttons [
                    (MenuButtonAction::SelectScene(SceneSelector::Main), "Scene\nMain"),
                    (MenuButtonAction::SelectScene(SceneSelector::Experimental), "Scene\nExperimental"),
                ],
            },
        },
    },
    Bottom {
        Column {
            Buttons [
                (MenuButtonAction::StartGame, "Start Game"),
                (MenuButtonAction::MultiPlayer, "Back") + (
                    KeyToButtonBinding(KeyCode::Escape)
                ),
            ],
        },
    },
);

build_menu_plugin!(
    (setup_settings_menu, Settings),
    Column {
        Text [
            "Haha you expected settings, but it was me, ",
            once text_color = colors::LEMON,
            "Dio",
            "!\n",
            text_color = colors::LAVENDER,
            "Go back to the playroom, stud.\n",
        ],
    },
    Bottom {
        Buttons [
            (MenuButtonAction::BackToMenu, "Back") + (
                KeyToButtonBinding(KeyCode::Escape)
            ),
        ],
    },
);

build_menu_plugin!(
    (setup_controls_menu, Controls),
    Bottom {
        Buttons [
            (MenuButtonAction::BackToMenu, "Back") + (
                KeyToButtonBinding(KeyCode::Escape)
            ),
        ],
    },
);

build_menu_plugin!(
    (setup_matchmaking_lobby_menu, MatchmakingLobby),
    Column {
        Text [ "Waiting for ", "", " more peer", "s", " to join...", ] + (
            PeerWaitingText { number_section_idx: 1, plurality_section_idx: 3 },
        ),
    },
    Bottom {
        Buttons [
            (MenuButtonAction::QuitToTitle, "Quit to Main Menu") + (
                KeyToButtonBinding(KeyCode::Escape)
            )
        ],
    },
);

/// Systems to handle the menu screens setup and despawning and more, if desired.
struct MenuSetupPlugins;

impl PluginGroup for MenuSetupPlugins {
    fn build(self) -> PluginGroupBuilder {
        use menu_state::*;

        PluginGroupBuilder::start::<Self>()
            .add(SingleMenuPlugin::<Main>::default())
            .add(SingleMenuPlugin::<Pause>::default())
            .add(SingleMenuPlugin::<MatchEnd>::default())
            .add(SingleMenuPlugin::<MatchmakingLobby>::default())
            .add(SingleMenuPlugin::<SinglePlayer>::default())
            .add(SingleMenuPlugin::<MultiPlayer>::default())
            .add(SingleMenuPlugin::<MatchMaker>::default())
            //.add(SingleMenuPlugin::<MatchBrowser>::default())
            .add(SingleMenuPlugin::<Controls>::default())
            .add(SingleMenuPlugin::<Settings>::default())
    }
}

/// System to initialize the default Main Menu state.
fn set_main_menu_state(mut menu_state: ResMut<NextState<MenuState>>) {
    menu_state.set(MenuState::Main);
}

/// System to read and apply game pause events to set the new menu state.
fn pause_menu(
    mut pause_events: EventReader<GamePauseEvent>,
    mut menu_state: ResMut<NextState<MenuState>>,
    player_count: Res<PlayerCount>,
) {
    if !pause_events
        .iter()
        .any(|event| matches!(event, GamePauseEvent::Pause | GamePauseEvent::Toggle))
    {
        return;
    }

    if player_count.0 <= 1 {
        // todo pause
    }
    menu_state.set(MenuState::Pause);
}

/// System to read and apply game unpause events to set the new menu state.
fn unpause_menu(
    mut pause_events: EventReader<GamePauseEvent>,
    mut menu_state: ResMut<NextState<MenuState>>,
    player_count: Res<PlayerCount>,
) {
    if !pause_events
        .iter()
        .any(|event| matches!(event, GamePauseEvent::Unpause | GamePauseEvent::Toggle))
    {
        return;
    }

    if player_count.0 <= 1 {
        // unpause
    }
    menu_state.set(MenuState::Disabled);
}

/// Handle button press interactions.
pub(crate) fn handle_menu_actions(
    mut commands: Commands,
    interaction_query: Query<
        (
            &Interaction,
            &MenuButtonAction,
            /*Option<&Focus>,*/ Entity,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    // focus_query: Query<&Focus>,
    mut scene_focus_query: Query<&mut Focus<SceneSelector>>,
    mut pause_events: EventWriter<GamePauseEvent>,
    #[cfg(not(target_arch = "wasm32"))] mut app_exit_events: EventWriter<AppExit>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut game_state: ResMut<NextState<GameState>>,
    current_game_state: Res<State<GameState>>,
) {
    for (interaction, menu_button_action, entity) in &interaction_query {
        if *interaction == Interaction::Clicked {
            match menu_button_action {
                #[cfg(not(target_arch = "wasm32"))]
                MenuButtonAction::Quit => app_exit_events.send(AppExit),
                MenuButtonAction::SinglePlayer => {
                    commands.insert_resource(PlayerCount(1));
                    menu_state.set(MenuState::SinglePlayer);
                }
                MenuButtonAction::MultiPlayer => {
                    commands.insert_resource(PlayerCount(2));
                    menu_state.set(MenuState::MultiPlayer)
                }
                MenuButtonAction::JoinGame => menu_state.set(MenuState::MatchMaker),
                MenuButtonAction::SelectScene(scene) => {
                    for mut focus in scene_focus_query.iter_mut() {
                        if let Focus::Focused(_) = *focus {
                            *focus = Focus::None;
                        }
                    }
                    commands
                        .entity(entity)
                        .insert(Focus::<SceneSelector>::focused(*scene));
                }
                MenuButtonAction::StartGame => {
                    let scene_arg = scene_focus_query
                        .iter()
                        .find_map(|focus| focus.extract_context());

                    match scene_arg {
                        Some(context) => {
                            commands.insert_resource(context);

                            game_state.set(GameState::Matchmaking);
                            menu_state.set(MenuState::MatchmakingLobby);
                        }
                        None => {
                            // notify the player?
                        }
                    }
                }
                MenuButtonAction::Resume => pause_events.send(GamePauseEvent::Unpause),
                MenuButtonAction::Rematch => pause_events.send(GamePauseEvent::Unpause), // todo
                MenuButtonAction::Controls => menu_state.set(MenuState::Controls),
                MenuButtonAction::Settings => menu_state.set(MenuState::Settings),
                MenuButtonAction::BackToMenu => {
                    if current_game_state.0 == GameState::InGame {
                        menu_state.set(MenuState::Pause)
                    } else {
                        menu_state.set(MenuState::Main)
                    }
                }
                MenuButtonAction::QuitToTitle => {
                    pause_events.send(GamePauseEvent::Unpause);
                    game_state.set(GameState::MainMenu);
                    menu_state.set(MenuState::Main);
                }
            }
        }
    }
}

/// Plugin handling all menu interactions, spawnings and despawnings.
pub struct MenuPlugin;

// todo I want them blooms on the UI. Figure out how to do blooms!
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<MenuState>()
            // Plugins responsible for spawning and despawning the menus
            .add_plugins(MenuSetupPlugins)
            .add_system(set_main_menu_state.in_schedule(OnEnter(GameState::MainMenu)))
            .add_system(
                pause_menu
                    .run_if(in_state(GameState::InGame).and_then(in_state(MenuState::Disabled))),
            )
            .add_system(
                unpause_menu.run_if(
                    in_state(GameState::InGame).and_then(not(in_state(MenuState::Disabled))),
                ),
            )
            .add_system(
                handle_menu_actions
                    .run_if(not(in_state(MenuState::Disabled)))
                    .in_base_set(CoreSet::Update),
            )
            .add_system(transfer_setting_from_text_input.in_schedule(OnExit(MenuState::Settings)))
            .add_system(
                transfer_setting_from_text_input.in_schedule(OnExit(MenuState::MultiPlayer)),
            );
        // Systems to handle the display settings screen
        /*.add_systems(
            OnEnter(MenuState::SettingsDisplay),
            display_settings_menu_setup,
        )
        .add_systems(
            CoreSet::Update,
            (
                setting_button::<DisplayQuality>
                    .run_if(in_state(MenuState::SettingsDisplay)),
            ),
        )
        .add_systems(
            OnExit(MenuState::SettingsDisplay),
            despawn_screen::<OnDisplaySettingsMenuScreen>,
        )
        // Systems to handle the sound settings screen
        .add_systems(OnEnter(MenuState::SettingsSound), sound_settings_menu_setup)
        .add_systems(
            CoreSet::Update,
            setting_button::<Volume>.run_if(in_state(MenuState::SettingsSound)),
        )
        .add_systems(
            OnExit(MenuState::SettingsSound),
            despawn_screen::<OnSoundSettingsMenuScreen>,
        )
        // Common systems to all screens that handles buttons behavior
        .add_systems(
            CoreSet::Update,
            (menu_action, button_system).run_if(in_state(GameState::Menu)),
        );*/
    }
}
