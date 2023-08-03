use crate::network::players::MatchTime;
use crate::network::{PlayerHandle, PlayerRegistry};
use crate::ui::fonts;
use crate::ui::input_consumption::{ActiveInputConsumerLayers, PLAYER_SCORE_VIEW_LAYER};
use crate::{GameState, MenuState};
use bevy::prelude::*;

#[derive(Component)]
struct TotalScoreDisplay;

#[derive(Component)]
struct PlayerScoreDisplay(PlayerHandle);

#[derive(Component)]
struct PlayerScoreStat;

#[derive(Component)]
struct MatchTimeDisplayPanel;

#[derive(Component)]
struct MatchTimeDisplay;

const STAT_FONT_SIZE: f32 = 17.;

const PLAYER_SCORE_NAME_IDX: usize = 0;
const PLAYER_SCORE_KILLS_IDX: usize = 1;
const PLAYER_SCORE_DEATHS_IDX: usize = 2;

fn util_create_individual_score_display_bundle(height_offset: f32) -> NodeBundle {
    NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Row,
            size: Size::new(Val::Percent(100.), Val::Px(STAT_FONT_SIZE + height_offset)),
            align_items: AlignItems::Start,
            justify_content: JustifyContent::Center,
            ..default()
        },
        ..default()
    }
}

fn util_setup_individual_score_display(
    parent: &mut ChildBuilder,
    name: impl Into<String>,
    kills: impl Into<String>,
    deaths: impl Into<String>,
    name_style: TextStyle,
    stat_style: TextStyle,
) {
    // Name section
    parent.spawn((
        TextBundle {
            text: Text::from_section(name, name_style).with_alignment(TextAlignment::Left),
            style: Style {
                size: Size::new(Val::Percent(50.), Val::Percent(100.)),
                ..default()
            },
            ..default()
        },
        PlayerScoreStat,
    ));

    // Kills section -- index must be corresponding to [`PLAYER_SCORE_KILLS_IDX`]
    parent.spawn((
        TextBundle {
            text: Text::from_section(kills, stat_style.clone()),
            style: Style {
                size: Size::new(Val::Percent(25.), Val::Percent(100.)),
                ..default()
            },
            ..default()
        },
        PlayerScoreStat,
    ));

    // Deaths section -- index must be corresponding to [`PLAYER_SCORE_DEATHS_IDX`]
    parent.spawn((
        TextBundle {
            text: Text::from_section(deaths, stat_style),
            style: Style {
                size: Size::new(Val::Percent(25.), Val::Percent(100.)),
                ..default()
            },
            ..default()
        },
        PlayerScoreStat,
    ));

    // Ping would be nice
}

fn should_show_score_display(keyboard: &Res<Input<KeyCode>>) -> bool {
    keyboard.pressed(KeyCode::Tab)
}

fn setup_score_display(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    match_time: Option<Res<MatchTime>>,
) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(35.0), Val::Percent(30.0)),
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        right: Val::Percent(2.5),
                        // assuming chat's end is at 2.5 + 20 percent, and pause menu is at 190 px (23.75%)
                        top: Val::Percent(23.75),
                        ..default()
                    },
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Start,
                    align_self: AlignSelf::Center,
                    justify_content: JustifyContent::Start,
                    padding: UiRect::all(Val::Px(5.)),
                    ..default()
                },
                background_color: Color::DARK_GRAY.with_a(0.4).into(),
                visibility: Visibility::Hidden,
                ..default()
            },
            TotalScoreDisplay,
        ))
        .with_children(|parent| {
            // Match time
            let mut match_time_display = util_create_individual_score_display_bundle(10.);
            if match_time.is_none() {
                match_time_display.visibility = Visibility::Hidden;
                match_time_display.style.display = Display::None;
            }
            parent
                .spawn((match_time_display, MatchTimeDisplayPanel))
                .with_children(|parent| {
                    let descriptor_style = TextStyle {
                        font: fonts::load(&asset_server, fonts::ULTRAGONIC),
                        font_size: STAT_FONT_SIZE,
                        color: Color::WHITE,
                    };

                    parent.spawn(TextBundle {
                        text: Text::from_section("Time left:", descriptor_style.clone())
                            .with_alignment(TextAlignment::Left),
                        style: Style {
                            size: Size::new(Val::Percent(50.), Val::Percent(100.)),
                            ..default()
                        },
                        ..default()
                    });

                    parent.spawn((
                        TextBundle {
                            text: Text::from_section("0:00", descriptor_style.clone()),
                            style: Style {
                                size: Size::new(Val::Percent(25.), Val::Percent(100.)),
                                ..default()
                            },
                            ..default()
                        },
                        MatchTimeDisplay,
                    ));
                });

            // Column descriptors
            parent
                .spawn(util_create_individual_score_display_bundle(10.))
                .with_children(|parent| {
                    let descriptor_style = TextStyle {
                        font: fonts::load(&asset_server, fonts::ULTRAGONIC),
                        font_size: STAT_FONT_SIZE,
                        color: Color::WHITE,
                    };

                    util_setup_individual_score_display(
                        parent,
                        "Name",
                        "Kills",
                        "Deaths",
                        descriptor_style.clone(),
                        descriptor_style,
                    );
                });
        });
}

fn populate_score_display(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    total_score_display_query: Query<(Entity, Option<&Children>), With<TotalScoreDisplay>>,
    individual_score_display_query: Query<(&PlayerScoreDisplay, &Children)>,
    mut player_score_stats_query: Query<&mut Text, With<PlayerScoreStat>>,
    players: Res<PlayerRegistry>,
) {
    if !players.is_changed() {
        return;
    }

    for (score_display_entity, display_children) in total_score_display_query.iter() {
        let mut scoreboard_size = 0;
        if let Some(display_children) = display_children {
            // yeah, super unsafe if there is more than 1 display. Until then...
            scoreboard_size = individual_score_display_query.iter().len()
                / total_score_display_query.iter().len(); // display_children.len() - 2;

            for display_entity in display_children {
                if let Ok((player_display, children)) =
                    individual_score_display_query.get(*display_entity)
                {
                    // if this expectancy ever triggers, should probably clean the displays instead of complain
                    // and if so, also scoreboard_size -= 1; but there's going to be more refactors to player registry anyway
                    let player_data = players.get(player_display.0).expect(&*format!(
                        "Player {} not found in the registry",
                        player_display.0
                    ));

                    // update the sacred texts; the books
                    let mut name_text = player_score_stats_query
                        .get_mut(children[PLAYER_SCORE_NAME_IDX])
                        .expect(&*format!(
                            "Failed to fetch child display for player {}'s name",
                            player_display.0
                        ));
                    name_text.sections[0].value = player_data.name.to_string();

                    // update the sacred texts; the books
                    let mut kills_text = player_score_stats_query
                        .get_mut(children[PLAYER_SCORE_KILLS_IDX])
                        .expect(&*format!(
                            "Failed to fetch child display for player {}'s kills",
                            player_display.0
                        ));
                    kills_text.sections[0].value = player_data.kills.to_string();

                    let mut deaths_text = player_score_stats_query
                        .get_mut(children[PLAYER_SCORE_DEATHS_IDX])
                        .expect(&*format!(
                            "Failed to fetch child display for player {}'s deaths",
                            player_display.0
                        ));
                    deaths_text.sections[0].value = player_data.deaths.to_string();
                }
            }
        }

        if scoreboard_size < players.len() {
            let stats_style = TextStyle {
                font: fonts::load(&asset_server, fonts::ULTRAGONIC),
                font_size: STAT_FONT_SIZE,
                color: Color::WHITE,
            };

            commands
                .entity(score_display_entity)
                .with_children(|parent| {
                    for i in scoreboard_size..players.len() {
                        let player_data = &players[i];
                        let name_style = TextStyle {
                            font: fonts::load(&asset_server, fonts::ULTRAGONIC),
                            font_size: STAT_FONT_SIZE,
                            color: player_data.team.color(),
                        };

                        parent
                            .spawn((
                                util_create_individual_score_display_bundle(5.),
                                PlayerScoreDisplay(i),
                            ))
                            .with_children(|parent| {
                                util_setup_individual_score_display(
                                    parent,
                                    player_data.name.clone(),
                                    player_data.kills.to_string(),
                                    player_data.deaths.to_string(),
                                    name_style,
                                    stats_style.clone(),
                                );
                            });
                    }
                });
        }
    }
}

fn handle_showing_score_display(
    keyboard: Res<Input<KeyCode>>,
    input_consumers: Res<ActiveInputConsumerLayers>,
    mut total_score_display_query: Query<&mut Visibility, With<TotalScoreDisplay>>,
    pause_state: Res<State<MenuState>>,
) {
    if input_consumers.is_input_blocked_for_layer(&PLAYER_SCORE_VIEW_LAYER) {
        return;
    }

    let should_show = should_show_score_display(&keyboard) || pause_state.0 == MenuState::Pause;

    for mut visibility in total_score_display_query.iter_mut() {
        match (*visibility, should_show) {
            (Visibility::Hidden, true) => *visibility = Visibility::Visible,
            (Visibility::Visible, false) => *visibility = Visibility::Hidden,
            _ => {}
        };
    }
}

fn handle_match_time_display(
    match_time: Option<Res<MatchTime>>,
    mut time_display_query: Query<&mut Text, With<MatchTimeDisplay>>,
    mut time_display_panel_query: Query<(&mut Visibility, &mut Style), With<MatchTimeDisplayPanel>>,
    mut match_time_removal_parsed: Local<bool>,
) {
    if time_display_query.is_empty() || match_time.is_none() && !*match_time_removal_parsed {
        return;
    }

    if let Some(match_time) = match_time {
        let remaining_secs = match_time.0.remaining_secs().floor() as i32;
        let minutes = remaining_secs / 60;
        let seconds = remaining_secs % 60;
        let string_of_time = format!("{}:{:02}", minutes, seconds);

        time_display_query.for_each_mut(|mut clock_face| {
            clock_face.sections[0].value = string_of_time.clone();
        });

        if match_time.is_added() {
            time_display_panel_query.for_each_mut(|(mut clock_veil, mut fashion)| {
                *clock_veil = Visibility::default();
                fashion.display = Display::default();
            });
        }
    } else {
        time_display_panel_query.for_each_mut(|(mut clock_veil, mut fashion)| {
            *clock_veil = Visibility::Hidden;
            fashion.display = Display::None;
        });

        *match_time_removal_parsed = true;
    }
}

pub struct PlayerScorePlugin;
impl Plugin for PlayerScorePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup_score_display.in_schedule(OnEnter(GameState::InGame)))
            // despawn_node::<TotalScoreDisplay> -- handled by despawn_everything
            .add_system(populate_score_display.run_if(in_state(GameState::InGame)))
            .add_system(handle_showing_score_display.run_if(in_state(GameState::InGame)))
            .add_system(handle_match_time_display.run_if(in_state(GameState::InGame)));
    }
}
