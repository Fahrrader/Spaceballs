use crate::ui::menu_builder::DEFAULT_TEXT_COLOR;
use crate::ui::{colors, despawn_node, ColorInteractionMap, Focus};
use crate::{build_menu_plugin, GamePauseEvent, GameState, PlayerCount, SceneSelector};
use bevy::app::{AppExit, PluginGroupBuilder};
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
    MatchBrowser,
    // Test,
    Controls,
    // Tutorial,
    Settings,
    Pause,
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
    HostGame,
    SelectScene(SceneSelector),
    StartSinglePlayerGame,
    StartMultiPlayerGame,
    Resume,
    Controls,
    Settings,
    BackToMenu,
    QuitToTitle,
    Quit,
}

/// Handle changing all buttons' colors based on mouse interaction.
fn handle_button_style_change(
    interaction_query: Query<
        (&Interaction, Option<&Focus<SceneSelector>>, Entity),
        (
            With<ColorInteractionMap>,
            Or<(Changed<Interaction>, Changed<Focus<SceneSelector>>)>,
        ),
    >,
    mut text_children_query: Query<(&mut Text, Option<&ColorInteractionMap>)>,
    mut node_children_query: Query<(
        &mut BackgroundColor,
        Option<&ColorInteractionMap>,
        Option<&Children>,
    )>,
) {
    fn distill_color(
        interaction: Interaction,
        node_colors: &ColorInteractionMap,
        present_color: Color,
        // node_entity: Entity,
    ) -> Option<Color> {
        node_colors
            .get(interaction)
            .copied()
            .filter(|_| node_colors.has_color(present_color))
            .or_else(|| {
                // warn!("UI entity {} has a color interaction map but possesses a color outside of it. Didn't paint over.", node_entity.index());
                None
            })
    }

    fn extract_color(
        interaction: Interaction,
        color: Color,
        color_interaction_map: Option<&ColorInteractionMap>,
    ) -> Color {
        color_interaction_map
            .and_then(|map| distill_color(interaction, map, color))
            .unwrap_or(color)
    }

    fn paint_nodes(
        interaction: Interaction,
        children: &Vec<Entity>,
        text_children_query: &mut Query<(&mut Text, Option<&ColorInteractionMap>)>,
        node_children_query: &mut Query<(
            &mut BackgroundColor,
            Option<&ColorInteractionMap>,
            Option<&Children>,
        )>,
    ) {
        for &child in children {
            if let Ok((mut text, color_interaction_map)) = text_children_query.get_mut(child) {
                text.sections.iter_mut().for_each(|section| {
                    section.style.color =
                        extract_color(interaction, section.style.color, color_interaction_map);
                });
            }

            if let Ok((mut background, color_interaction_map, more_children)) =
                node_children_query.get_mut(child)
            {
                *background =
                    extract_color(interaction, background.0, color_interaction_map).into();

                if let Some(more_children) = more_children {
                    let children_cloned = more_children.iter().cloned().collect();
                    paint_nodes(
                        interaction,
                        &children_cloned,
                        text_children_query,
                        node_children_query,
                    );
                }
            }
        }
    }

    for (interaction, focus, entity) in interaction_query.iter() {
        let interaction = match (*interaction, focus) {
            (Interaction::None, Some(&Focus::Focused(_))) => Interaction::Hovered,
            _ => *interaction,
        };

        paint_nodes(
            interaction,
            &vec![entity],
            &mut text_children_query,
            &mut node_children_query,
        );
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
        // todo unpause
    }
    menu_state.set(MenuState::Disabled);
}

/* Main menu UI structure
* singleplayer {
    * scene select {  } (grid of available levels)
    * start button
    * back button
}
* multiplayer {
    * host {
        * scene select {  }
        * start button
        * back button
    }
    * join {
        (maybe possibly list of peers/rooms)
        * text input { direct connection: room URL }
        * join button
        * back button
    }
    * back button
}
* controls {
    * describe_controls ( on the left - wasd/left thumbstick, space, r, f, c, etc.; on the right - descriptions )
    * back button
}
* settings {
    * text { Haha you expected settings, but it was me, Dio!\nGo back to the playground, stud.}
    * button { Back }
}
* quit button
 */

/* In-game menu UI structure
blurry top node
* title?
* text (must be selectable and copyable (text input, but non-editable?)) { [Room URL] }
* continue
* controls
* settings
* quit to main menu
* quit
 */

build_menu_plugin!(
    (setup_main_menu, Main),
    once align_items = AlignItems::Start.into(),
    once layout_height = Val::Percent(42.5).into(),
    Column {
        text_font_size = 60.0,
        Text [
            "Cosmic\n",
            once text_font_size = 72.0,
            once text_color = colors::LEMON,
            "Spaceball\n",
            "Tactical Action Arena",
        ] + (
            ColorInteractionMap::from([
                (Interaction::None, Some(DEFAULT_TEXT_COLOR.with_a(0.99))),
                (Interaction::Hovered, Some(DEFAULT_TEXT_COLOR.with_a(0.99))),
                (Interaction::Clicked, Some(colors::LEMON)),
            ]),
            Interaction::None,
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
                    (MenuButtonAction::Quit, "Quit"),
                ],
            },
        },
    },
);

build_menu_plugin!(
    (setup_pause_menu, Pause),
    //once node_color = Color::TURQUOISE.with_a(0.05),
    once layout_height = Val::Percent(100.).into(),
    once layout_width = Val::Percent(50.).into(),
    justify_content = JustifyContent::Start.into(),
    once align_items = AlignItems::Center.into(),
    button_width = Val::Px(280.),
    button_font_size = 30.,
    button_margin = UiRect::all(Val::Px(2.)),
    outline_width = Val::Px(0.),
    Left {
        once align_items = AlignItems::Start.into(),
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
);
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
                    (MenuButtonAction::SelectScene(SceneSelector::Lite), "Scene\nLite"),
                    (MenuButtonAction::SelectScene(SceneSelector::Experimental), "Scene\nExperimental"),
                ],
            },
        },
    },
    Bottom {
        Column {
            Buttons [
                (MenuButtonAction::StartSinglePlayerGame, "Start Game"),
                (MenuButtonAction::BackToMenu, "Back"),
            ],
        },
    },
);

build_menu_plugin!(
    (setup_multiplayer_menu, MultiPlayer),
    once align_items = AlignItems::Start.into(),
    once align_self = AlignSelf::Start.into(),
    Column {
        Text [
            "Multiplayer",
        ],
    },
    Bottom {
        Column {
            Buttons [
                (MenuButtonAction::JoinGame, "Join Game"),
                (MenuButtonAction::HostGame, "Host Game"),
                (MenuButtonAction::BackToMenu, "Back"),
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
            (MenuButtonAction::BackToMenu, "Back"),
        ],
    },
);

build_menu_plugin!(
    (setup_controls_menu, Controls),
    Bottom {
        Buttons [
            (MenuButtonAction::BackToMenu, "Back"),
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
            .add(SingleMenuPlugin::<SinglePlayer>::default())
            .add(SingleMenuPlugin::<MultiPlayer>::default())
            //.add(SingleMenuPlugin::<MatchMaker>::default())
            //.add(SingleMenuPlugin::<MatchBrowser>::default())
            .add(SingleMenuPlugin::<Controls>::default())
            .add(SingleMenuPlugin::<Settings>::default())
    }
}

/// Handle button press interactions.
fn handle_menu_actions(
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
    mut app_exit_events: EventWriter<AppExit>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut game_state: ResMut<NextState<GameState>>,
    current_game_state: Res<State<GameState>>,
) {
    for (interaction, menu_button_action, entity) in &interaction_query {
        if *interaction == Interaction::Clicked {
            match menu_button_action {
                MenuButtonAction::Quit => app_exit_events.send(AppExit),
                MenuButtonAction::SinglePlayer => menu_state.set(MenuState::SinglePlayer),
                MenuButtonAction::MultiPlayer => menu_state.set(MenuState::MultiPlayer),
                MenuButtonAction::HostGame => menu_state.set(MenuState::MatchMaker),
                MenuButtonAction::JoinGame => menu_state.set(MenuState::MatchBrowser),
                MenuButtonAction::SelectScene(scene) => {
                    for mut focus in scene_focus_query.iter_mut() {
                        if let Focus::Focused(_) = *focus {
                            *focus = Focus::None;
                        }
                    }
                    commands
                        .entity(entity)
                        .insert(Focus::<SceneSelector>::Focused(*scene));
                }
                MenuButtonAction::StartSinglePlayerGame => {
                    let scene_arg = scene_focus_query
                        .iter()
                        .find_map(|focus| focus.extract_context());

                    match scene_arg {
                        Some(context) => {
                            commands.insert_resource(context);
                            commands.insert_resource(PlayerCount(1));

                            game_state.set(GameState::Matchmaking);
                            menu_state.set(MenuState::Disabled);
                        }
                        None => {
                            // notify the player?
                        }
                    }
                }
                MenuButtonAction::StartMultiPlayerGame => {
                    let scene_arg = scene_focus_query
                        .iter()
                        .find_map(|focus| focus.extract_context());

                    match scene_arg {
                        Some(context) => {
                            commands.insert_resource(context);
                            commands.insert_resource(PlayerCount(2));

                            game_state.set(GameState::Matchmaking);
                            menu_state.set(MenuState::Disabled);
                        }
                        None => {
                            // notify the player?
                        }
                    }
                }
                MenuButtonAction::Resume => pause_events.send(GamePauseEvent::Unpause),
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
            .add_systems(
                (
                    handle_menu_actions.run_if(not(in_state(MenuState::Disabled))),
                    handle_button_style_change.run_if(not(in_state(MenuState::Disabled))),
                )
                    .in_base_set(CoreSet::Update),
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
