use crate::ui::{colors, despawn_node, ColorInteractionMap};
use crate::{build_menu_plugin, GameState};
use bevy::app::{AppExit, PluginGroupBuilder};
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
    SelectScene(usize),
    JoinGame,
    HostGame,
    Controls,
    Settings,
    QuitToMenu,
    // todo:web no show if in browser -- there is no place to escape!
    Quit,
}

/// Tag component used to mark which setting is currently selected.
#[derive(Component)]
struct SelectedOption;

/// Handle changing all buttons' colors based on mouse interaction.
fn handle_button_style_change(
    interaction_query: Query<
        (
            &Interaction,
            Option<&SelectedOption>,
            &ColorInteractionMap,
            Entity,
        ),
        Changed<Interaction>,
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
        node_colors: Option<&ColorInteractionMap>,
        default_colors: &ColorInteractionMap,
        present_color: Color,
    ) -> (Color, bool) {
        match node_colors.and_then(|map| map.get(interaction)) {
            Some(color) => (*color, node_colors.unwrap().has_color(present_color)),
            None => match default_colors.get(interaction) {
                Some(color) => (*color, default_colors.has_color(present_color)),
                None => (present_color, false),
            },
        }
    }

    fn paint_nodes(
        interaction: Interaction,
        default_colors: &ColorInteractionMap,
        children: &Vec<Entity>,
        text_children_query: &mut Query<(&mut Text, Option<&ColorInteractionMap>)>,
        node_children_query: &mut Query<(
            &mut BackgroundColor,
            Option<&ColorInteractionMap>,
            Option<&Children>,
        )>,
    ) {
        for &child in children.iter() {
            if let Ok((mut text, color_interaction_map)) = text_children_query.get_mut(child) {
                let (color, _) = distill_color(
                    interaction,
                    color_interaction_map,
                    default_colors,
                    Color::WHITE,
                );
                for mut section in text.sections.iter_mut() {
                    section.style.color = color;
                }
            }

            if let Ok((mut background, color_interaction_map, more_children)) =
                node_children_query.get_mut(child)
            {
                let (new_color, should_paint) = distill_color(
                    interaction,
                    color_interaction_map,
                    default_colors,
                    background.0,
                );
                if should_paint {
                    *background = new_color.into();
                }

                if let Some(more_children) = more_children {
                    let children_cloned = more_children.iter().cloned().collect();
                    paint_nodes(
                        interaction,
                        default_colors,
                        &children_cloned,
                        text_children_query,
                        node_children_query,
                    );
                }
            }
        }
    }

    for (interaction, selected, color_interaction_map, entity) in interaction_query.iter() {
        let interaction = match (*interaction, selected) {
            (Interaction::None, Some(_)) => Interaction::Hovered,
            _ => *interaction,
        };

        paint_nodes(
            interaction,
            color_interaction_map,
            &vec![entity],
            &mut text_children_query,
            &mut node_children_query,
        );
    }
}

fn set_main_menu_state(mut menu_state: ResMut<NextState<MenuState>>) {
    menu_state.set(MenuState::Main);
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
    Column {
        Title,
        //node_background_color = colors::PEACH.with_a(0.3).into(),
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
        Buttons [
            (MenuButtonAction::Quit, "Quit"),
        ],
    },
);

build_menu_plugin!(
    (setup_settings_menu, Settings),
    Column {
        Text [
            "Haha you expected settings, but it was me, ",
            {
                text_color = colors::LEMON,
                "Dio",
            },
            "!\n",
            text_color = colors::LAVENDER,
            "Go back to the playroom, stud.\n",
        ],
        Buttons [
            (MenuButtonAction::QuitToMenu, "Back"),
        ],
    },
);

build_menu_plugin!(
    (setup_controls_menu, Controls),
    Column {
        Buttons [
            (MenuButtonAction::QuitToMenu, "Back"),
        ],
    },
);

build_menu_plugin!(
    (setup_multiplayer_menu, MultiPlayer),
    Column {
        Text [
            "Multiplayer",
        ],
        Buttons [
            (MenuButtonAction::JoinGame, "Join Game"),
            (MenuButtonAction::HostGame, "Host Game"),
            (MenuButtonAction::QuitToMenu, "Back"),
        ],
    },
);

/// Systems to handle the menu screens setup and despawning
struct MenuSetupPlugins;

impl PluginGroup for MenuSetupPlugins {
    fn build(self) -> PluginGroupBuilder {
        use menu_state::*;

        PluginGroupBuilder::start::<Self>()
            .add(SingleMenuPlugin::<Main>::default())
            //.add(SingleMenuPlugin::<SinglePlayer>::default())
            .add(SingleMenuPlugin::<MultiPlayer>::default())
            //.add(SingleMenuPlugin::<MatchMaker>::default())
            //.add(SingleMenuPlugin::<MatchBrowser>::default())
            .add(SingleMenuPlugin::<Controls>::default())
            .add(SingleMenuPlugin::<Settings>::default())
    }
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
                MenuButtonAction::MultiPlayer => menu_state.set(MenuState::MultiPlayer),
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
        app
            // At start, the menu is not enabled. This will be changed in `menu_setup` when
            // entering the `GameState::Menu` state.
            // Current screen in the menu is handled by an independent state from `GameState`
            .add_state::<MenuState>()
            .add_plugins(MenuSetupPlugins)
            .add_system(set_main_menu_state.in_schedule(OnEnter(GameState::MainMenu)))
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
