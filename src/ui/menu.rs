use crate::ui::menu_builder::outline_parent;
use crate::ui::{colors, despawn_node, ColorInteractionMap};
use crate::{build_menu_system, GameState};
use bevy::app::AppExit;
use bevy::prelude::*;

macro_rules! generate_menu_states {
    ($($state:ident),* $(,)?) => {
        /// State used for the current menu screen.
        #[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
        enum MenuState {
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

/// All actions that can be triggered from a button click.
#[derive(Component)]
pub(crate) enum MenuButtonAction {
    SinglePlayer,
    MultiPlayer,
    // Quickmatch?
    // Match Browser?
    SelectScene(usize),
    JoinGame,
    HostGame,
    Controls,
    Settings,
    QuitToMenu,
    // todo:web no show if in browser -- there is no place to escape!
    Quit,
}

// todo useless, delete
/// Tag component used to mark which setting is currently selected.
#[derive(Component)]
struct SelectedOption;

pub(crate) const TEXT_COLOR: Color = colors::AERO_BLUE;
pub(crate) const DEFAULT_BUTTON_COLOR: Color = Color::YELLOW_GREEN;
pub(crate) const HOVERED_BUTTON_COLOR: Color = TEXT_COLOR;
//const HOVERED_PRESSED_BUTTON: Color = colors::AERO_BLUE;
pub(crate) const PRESSED_BUTTON_COLOR: Color = Color::CYAN;

/// Handle changing all buttons' colors based on mouse interaction.
fn handle_button_style_change(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&ColorInteractionMap>,
            Option<&Children>,
            Option<&SelectedOption>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_children_query: Query<(&mut Text, Option<&ColorInteractionMap>)>,
    mut node_children_query: Query<
        (
            &mut BackgroundColor,
            Option<&ColorInteractionMap>,
            Option<&Children>,
        ),
        (Without<Button>, Without<Text>),
    >,
) {
    fn distill_color(
        interaction: Interaction,
        color_interaction_map: Option<&ColorInteractionMap>,
    ) -> (Color, bool) {
        match color_interaction_map.and_then(|map| map.get(interaction)) {
            Some(color) => (*color, true),
            None => {
                let default_color = match interaction {
                    Interaction::Clicked => PRESSED_BUTTON_COLOR,
                    Interaction::Hovered => HOVERED_BUTTON_COLOR,
                    Interaction::None => DEFAULT_BUTTON_COLOR,
                };
                (default_color, false)
            }
        }
    }

    fn paint_background(
        background: &mut BackgroundColor,
        new_color: Color,
        should_ignore_transparency: bool,
    ) {
        if should_ignore_transparency || background.0 != Color::NONE {
            *background = new_color.into();
        }
    }

    fn paint_children(
        interaction: Interaction,
        children: &Vec<Entity>,
        text_children_query: &mut Query<(&mut Text, Option<&ColorInteractionMap>)>,
        node_children_query: &mut Query<
            (
                &mut BackgroundColor,
                Option<&ColorInteractionMap>,
                Option<&Children>,
            ),
            (Without<Button>, Without<Text>),
        >,
    ) {
        for &child in children.iter() {
            if let Ok((mut text, color_interaction_map)) = text_children_query.get_mut(child) {
                let (color, _) = distill_color(interaction, color_interaction_map);
                for mut section in text.sections.iter_mut() {
                    section.style.color = color;
                }
            }

            if let Ok((mut bg_color, color_interaction_map, more_children)) =
                node_children_query.get_mut(child)
            {
                let (color, should_ignore_transparency) =
                    distill_color(interaction, color_interaction_map);
                if color_interaction_map.is_some() {
                    paint_background(&mut bg_color, color, should_ignore_transparency);
                }

                if let Some(more_children) = more_children {
                    let children_cloned = more_children.iter().cloned().collect();
                    paint_children(
                        interaction,
                        &children_cloned,
                        text_children_query,
                        node_children_query,
                    );
                }
            }
        }
    }

    for (interaction, mut color, color_interaction_map, children, selected) in
        interaction_query.iter_mut()
    {
        let interaction = match (*interaction, selected) {
            (Interaction::None, Some(_)) => Interaction::Clicked,
            _ => *interaction,
        };
        let (new_color, should_ignore_transparency) =
            distill_color(interaction, color_interaction_map);

        if color_interaction_map.is_some() {
            paint_background(&mut color, new_color, should_ignore_transparency);
        }

        if children.is_none() {
            continue;
        }
        let children_cloned = children.unwrap().iter().cloned().collect();
        paint_children(
            interaction,
            &children_cloned,
            &mut text_children_query,
            &mut node_children_query,
        );
    }
}

fn set_main_menu_state(mut menu_state: ResMut<NextState<MenuState>>) {
    menu_state.set(MenuState::Main);
}

build_menu_system!(
    setup_main_menu,
    Main {
        Column {
            Title,
            node_background_color = colors::PEACH.with_a(0.3).into(),
            Buttons [
                (MenuButtonAction::SinglePlayer, "Singleplayer"),
                (MenuButtonAction::MultiPlayer, "Multiplayer"),
            ],
            Column {
                button_color = colors::NEON_PINK.into(),
                button_text_hovered_color = colors::LEMON.into(),
                button_font_size = 16.0,
                Buttons [
                    (MenuButtonAction::Controls, "Controls"),
                    (MenuButtonAction::Settings, "Settings"),
                ],
            },
            Buttons [
                (MenuButtonAction::Quit, "Quit"),
            ],
        },
    },
);
/*
fn setup_main_menu(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    build_menu!(
        Main {
            Column {
                //Title,
                Buttons [
                    (MenuButtonAction::SinglePlayer, "Singleplayer"),
                    (MenuButtonAction::MultiPlayer, "Multiplayer"),
                    (MenuButtonAction::Controls, "Controls"),
                    (MenuButtonAction::Settings, "Settings"),
                    (MenuButtonAction::Quit, "Quit"),
                ],
            }
        }
    );

    let title_text_style = TextStyle {
        font: font.clone(),
        font_size: 60.0,
        color: TEXT_COLOR,
    };

    let button_text_style = TextStyle {
        font_size: 40.0,
        color: DEFAULT_BUTTON_COLOR,
        font: font.clone(),
    };

    // Common style for all buttons on the screen
    let button_style = Style {
        size: Size::new(Val::Px(300.0), Val::Px(65.0)),
        margin: UiRect::all(Val::Px(4.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            OnMenu::<menu_state::Main>::default(),
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    // background_color: Color::CRIMSON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    // Display the game name
                    parent.spawn(
                        TextBundle::from_section(
                            "Cosmic\nSpaceball\nTactical Action Arena",
                            title_text_style.clone(),
                        )
                            .with_style(Style {
                                margin: UiRect::all(Val::Px(50.0)),
                                ..default()
                            })
                            .with_text_alignment(TextAlignment::Center),
                    );

                    // todo unite under one node bundle, keep it constrained so that title text doesn't shift around
                    // Display three buttons for each action available from the main menu:
                    // - play, multiplayer
                    // - controls
                    // - settings
                    // - quit
                    for (action, text) in [
                        (MenuButtonAction::SinglePlayer, "Play"),
                        (MenuButtonAction::Controls, "Controls"),
                        (MenuButtonAction::Settings, "Settings"),
                        // don't display if on web!
                        (MenuButtonAction::Quit, "Quit"),
                    ] {
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style.clone(),
                                    background_color: Color::NONE.into(),
                                    ..default()
                                },
                                action,
                            ))
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section(
                                    text,
                                    button_text_style.clone(),
                                ));

                                outline_parent(parent, Val::Px(4.), DEFAULT_BUTTON_COLOR);
                            });
                    }
                });
        });
}*/

/*fn setup_singleplayer_menu(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    let font = asset_server.load("fonts/Spacerunner.otf");

    /* layout:
    * text { Haha you expected settings, but it was me, Dio!\nGo back to the playroom, stud.}
    * button { Back }
     */

    build_menu!(
        OnSinglePlayerMenu {
            Column {
                Text(
                    (Haha you expected settings, but it was me, Dio!),
                    (Go back to the playroom, stud.)
                ),
                Buttons [
                    (MenuButtonAction::BackToMainMenu, "Back"),
                ],
            }
        }
    );
}*/

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

fn setup_settings_menu(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    let font = asset_server.load("fonts/Spacerunner.otf");

    /* layout:
     * text { Haha you expected settings, but it was me, Dio!\nGo back to the playroom, stud.}
     * button { Back }
     */

    /*build_menu!(
        OnSettingsMenuScreen {
            Column {
                Text(
                    (Haha you expected settings, but it was me, Dio!),
                    (Go back to the playroom, stud.)
                ),
                Buttons [
                    (MenuButtonAction::BackToMainMenu, "Back"),
                ],
            }
        }
    );*/

    let button_style = Style {
        size: Size::new(Val::Px(250.0), Val::Px(65.0)),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_style = TextStyle {
        font_size: 40.0,
        color: TEXT_COLOR,
        font: font.clone(),
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::width(Val::Percent(100.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            OnMenu::<menu_state::Settings>::default(),
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    //background_color: Color::CRIMSON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    for (action, text) in [
                        //(MenuButtonAction::SettingsDisplay, "Display"),
                        (MenuButtonAction::Quit, "Sound"),
                        (MenuButtonAction::QuitToMenu, "Back"),
                    ] {
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style.clone(),
                                    background_color: Color::NONE.into(),
                                    ..default()
                                },
                                action,
                            ))
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section(
                                    text,
                                    button_text_style.clone(),
                                ));

                                outline_parent(parent, Val::Px(4.), DEFAULT_BUTTON_COLOR, None);
                            });
                    }
                });
        });
}

fn handle_menu_actions(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_events: EventWriter<AppExit>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Clicked {
            match menu_button_action {
                MenuButtonAction::Quit => app_exit_events.send(AppExit),
                MenuButtonAction::SinglePlayer => {
                    game_state.set(GameState::Matchmaking);
                    menu_state.set(MenuState::Disabled);
                }
                MenuButtonAction::MultiPlayer => {}
                MenuButtonAction::SelectScene(_) => {}
                MenuButtonAction::HostGame => {}
                MenuButtonAction::JoinGame => {}
                MenuButtonAction::Controls => menu_state.set(MenuState::Controls),
                MenuButtonAction::Settings => menu_state.set(MenuState::Settings),
                /*MenuButtonAction::SettingsDisplay => {
                    menu_state.set(MenuState::SettingsDisplay);
                }
                MenuButtonAction::SettingsSound => {
                    menu_state.set(MenuState::SettingsSound);
                }
                MenuButtonAction::BackToMainMenu => ,
                MenuButtonAction::BackToSettings => {
                    menu_state.set(MenuState::Settings);
                }*/
                // todo also set game state to main menu?
                // add despawning system to the game that would trigger on exiting InGame state
                MenuButtonAction::QuitToMenu => menu_state.set(MenuState::Main),
            }
        }
    }
}

pub struct MenuPlugin;

// todo I want them blooms on the UI. Figure out how to do blooms!
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        use menu_state::*;

        app
            // At start, the menu is not enabled. This will be changed in `menu_setup` when
            // entering the `GameState::Menu` state.
            // Current screen in the menu is handled by an independent state from `GameState`
            .add_state::<MenuState>()
            .add_system(set_main_menu_state.in_schedule(OnEnter(GameState::MainMenu)))
            // Systems to handle the main menu screen
            .add_system(setup_main_menu.in_schedule(OnEnter(MenuState::Main)))
            .add_system(despawn_node::<OnMenu<Main>>.in_schedule(OnExit(MenuState::Main)))
            .add_systems(
                (
                    handle_menu_actions
                        // eh, no, what about in-game menu
                        .run_if(in_state(GameState::MainMenu)),
                    handle_button_style_change
                        // eh, no, what about in-game menu
                        .run_if(in_state(GameState::MainMenu)),
                )
                    .in_base_set(CoreSet::Update),
            )
            // Systems to handle the settings menu screen
            .add_system(setup_settings_menu.in_schedule(OnEnter(MenuState::Settings)))
            .add_system(despawn_node::<OnMenu<Settings>>.in_schedule(OnExit(MenuState::Settings)));
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
