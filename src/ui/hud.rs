use crate::ui::menu_builder::outline_parent;
use crate::ui::{despawn_node, fonts};
use crate::{CharacterActionInput, Equipped, GameState, Gun, Health};
use bevy::ecs::query::QuerySingleError;
use bevy::prelude::*;

#[derive(Component)]
pub struct HUDElement;

#[derive(Component)]
pub struct HealthDisplay;

#[derive(Component)]
pub struct GunDisplayHolder;

#[derive(Component)]
pub struct GunDisplay {
    pub gun_entity: Entity,

    pub name_display: Entity,
    pub max_ammo_display: Entity,
    pub ammo_display: Entity,
    pub fire_cooldown_display: Entity,
    pub reload_display: Entity,
}

impl GunDisplay {
    fn set_text(&self, entity: Entity, new_text: String, text_query: &mut Query<&mut Text>) {
        if let Ok(mut text) = text_query.get_mut(entity) {
            if let Some(section) = text.sections.last_mut() {
                section.value = new_text;
            }
        }
    }

    fn set_progress_bar_height(
        &self,
        entity: Entity,
        progress: f32,
        style_query: &mut Query<&mut Style>,
    ) {
        if let Ok(mut style) = style_query.get_mut(entity) {
            style.size.height = Val::Px(GUN_PROGRESS_BAR_HEIGHT * progress);
        }
    }

    pub fn update_gun_name_display(&self, gun: &Gun, text_query: &mut Query<&mut Text>) {
        self.set_text(
            self.name_display,
            gun.preset.stats().name.to_string(),
            text_query,
        );
    }

    pub fn update_max_ammo_display(&self, gun: &Gun, text_query: &mut Query<&mut Text>) {
        let gun_stats = gun.preset.stats();
        let max_ammo_text = match gun_stats.shots_before_reload {
            0 => "∞".to_string(),
            _ => gun_stats.shots_before_reload.to_string(),
        };
        self.set_text(self.max_ammo_display, max_ammo_text, text_query);
    }

    pub fn update_ammo_display(&self, gun: &Gun, text_query: &mut Query<&mut Text>) {
        let ammo_text = match gun.preset.stats().shots_before_reload {
            0 => "∞".to_string(),
            _ => gun.shots_before_reload.to_string(),
        };
        self.set_text(self.ammo_display, ammo_text, text_query);
    }

    pub fn update_fire_cooldown(&self, gun: &Gun, style_query: &mut Query<&mut Style>) {
        let fire_cooldown_progress =
            1. - gun.fire_cooldown.elapsed_secs() / gun.fire_cooldown.duration().as_secs_f32();
        self.set_progress_bar_height(
            self.fire_cooldown_display,
            fire_cooldown_progress,
            style_query,
        );
    }

    pub fn update_reload_display(&self, gun: &Gun, style_query: &mut Query<&mut Style>) {
        let reload_progress =
            gun.reload_progress.elapsed_secs() / gun.reload_progress.duration().as_secs_f32();
        self.set_progress_bar_height(self.reload_display, reload_progress, style_query);
    }
}

pub const GUN_PROGRESS_BAR_HEIGHT: f32 = 40. + 40. + 10. + 15. + 2.0;

fn setup_player_health_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = fonts::load(&asset_server, fonts::SPACERUNNER);

    // Spawn the health bar
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(300.0), Val::Px(36.0)),
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        left: Val::Px(20.0),
                        bottom: Val::Px(20.0),
                        ..default()
                    },
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::MAROON.with_a(0.1).into(),
                ..default()
            },
            HUDElement,
        ))
        .with_children(|parent| {
            parent.spawn((
                NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    background_color: Color::CRIMSON.with_a(0.8).into(),
                    ..default()
                },
                HealthDisplay,
            ));
            parent.spawn((
                TextBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        position: UiRect::left(Val::Percent(5.0)),
                        ..default()
                    },
                    text: Text::from_section(
                        "100", // Placeholder health value
                        TextStyle {
                            font: font.clone(),
                            font_size: 27.0,
                            color: Color::WHITE,
                        },
                    ),
                    ..default()
                },
                HealthDisplay,
            ));

            outline_parent(parent, Val::Px(2.), Color::WHITE, None);
        });
}

fn handle_health_hud(
    mut health_text_query: Query<&mut Text, With<HealthDisplay>>,
    mut health_bar_query: Query<&mut Style, (With<HealthDisplay>, Without<Text>)>,
    character_health_query: Query<&Health, (/*With<LocalPlayer>,*/ Changed<Health>,)>,
) {
    let health = match character_health_query.get_single() {
        Ok(health) => health.hp(),
        Err(err) => {
            match err {
                QuerySingleError::MultipleEntities(_) => {
                    error!("Multiple entities with `LocalPlayer` have changed `Health`!")
                }
                _ => {}
            };
            return;
        }
    };

    health_text_query.for_each_mut(|mut text| {
        if let Some(section) = text.sections.last_mut() {
            section.value = health.max(0.0).ceil().to_string();
        }
    });

    // todo add outer bar change due to max hp

    health_bar_query.for_each_mut(|mut style| {
        style.size.width = Val::Percent(health);
    });
}

fn setup_player_guns_hud(mut commands: Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                flex_direction: FlexDirection::RowReverse,
                align_items: AlignItems::End,
                position_type: PositionType::Absolute,
                position: UiRect {
                    right: Val::Px(20.0),
                    bottom: Val::Px(20.0),
                    ..default()
                },
                ..default()
            },
            ..default()
        },
        HUDElement,
        GunDisplayHolder,
    ));
}

fn handle_guns_hud_setup_change(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    gun_display_rack_query: Query<(Entity, Option<&Children>), With<GunDisplayHolder>>,
    mut gun_display_query: Query<(&mut GunDisplay, &mut Visibility)>,
    mut gun_text_query: Query<&mut Text>,
    mut gun_style_query: Query<&mut Style>,
    character_query: Query<
        &Children,
        (
            With<CharacterActionInput>, /*With<LocalPlayer>*/
            Changed<Children>,
        ),
    >,
    gun_query: Query<&Gun, With<Equipped>>,
) {
    let guns = match character_query.get_single() {
        Ok(children) /*if !children.is_empty()*/ => children,
        Err(err) => {
            match err {
                QuerySingleError::MultipleEntities(_) => error!("Multiple entities with `LocalPlayer` have changed `Children`!"),
                _ => {},
            };
            return;
        },
    };

    for (display_entity, display_children) in gun_display_rack_query.iter() {
        let display_children = match display_children {
            Some(children) => children.iter().cloned().collect::<Vec<_>>(),
            _ => vec![],
        };
        let min_len = display_children.len().min(guns.len());

        for i in 0..min_len {
            let (mut display, mut visibility) = gun_display_query
                .get_mut(display_children[i])
                .expect("Could not find a child of `GunDisplayHolder` in `GunDisplay`s!");
            let gun = gun_query
                .get(guns[i])
                .expect("Could not find an `Equipped` `Gun`!");
            *visibility = Visibility::Visible;

            display.gun_entity = guns[i];
            display.update_gun_name_display(gun, &mut gun_text_query);
            display.update_max_ammo_display(gun, &mut gun_text_query);
            display.update_ammo_display(gun, &mut gun_text_query);
            display.update_fire_cooldown(gun, &mut gun_style_query);
            display.update_reload_display(gun, &mut gun_style_query);
        }

        for i in min_len..display_children.len() {
            let (_, mut visibility) = gun_display_query
                .get_mut(display_children[i])
                .expect("Could not find a child of `GunDisplayHolder` in `GunDisplay`s!");
            *visibility = Visibility::Hidden;
        }

        if guns.len() > display_children.len() {
            let font = fonts::load(&asset_server, fonts::SPACERUNNER);
            let readable_font = fonts::load(&asset_server, fonts::FIRA_SANS);
            let readable_font_size = 40.0; // non_readable_font_size * 1.5;
            let non_readable_font_size = 17.0;
            let color = Color::WHITE.with_a(0.8);

            commands.entity(display_entity).with_children(|parent| {
                for i in display_children.len()..guns.len() {
                    let gun_stats = match gun_query.get(guns[i]) {
                        Ok(gun) => gun.preset.stats(),
                        Err(_) => continue,
                    };

                    let mut name_id = Entity::PLACEHOLDER;
                    let mut max_ammo_id = Entity::PLACEHOLDER;
                    let mut current_ammo_id = Entity::PLACEHOLDER;
                    let mut fire_cooldown_id = Entity::PLACEHOLDER;
                    let mut reload_id = Entity::PLACEHOLDER;

                    // Spawn a gun information panel
                    let mut gun_display_panel = parent.spawn((NodeBundle {
                        style: Style {
                            size: Size::new(Val::Px(110.), Val::Px(200.)),
                            margin: UiRect::left(Val::Px(5.0)),
                            flex_direction: FlexDirection::ColumnReverse,
                            justify_content: JustifyContent::End,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    },));

                    gun_display_panel.with_children(|parent| {
                        // Name of the gun
                        name_id = parent
                            .spawn((
                                TextBundle {
                                    text: Text::from_section(
                                        gun_stats.name,
                                        TextStyle {
                                            font: font.clone(),
                                            font_size: non_readable_font_size,
                                            color,
                                        },
                                    ),
                                    ..default()
                                },
                                // GunNameDisplay,
                            ))
                            .id();

                        // Maximum amount of ammo possible in the magazine
                        max_ammo_id = parent
                            .spawn(TextBundle {
                                text: Text::from_section(
                                    match gun_stats.shots_before_reload {
                                        0 => "∞".to_string(),
                                        _ => gun_stats.shots_before_reload.to_string(),
                                    },
                                    TextStyle {
                                        font: readable_font.clone(),
                                        font_size: readable_font_size,
                                        color,
                                    },
                                ),
                                style: Style {
                                    margin: UiRect::all(Val::Px(10.)),
                                    ..default()
                                },
                                ..default()
                            })
                            .id();

                        // Horizontal separating line
                        parent.spawn(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(75.), Val::Px(2.)),
                                ..default()
                            },
                            background_color: color.into(),
                            ..default()
                        });

                        // Current amount of ammo in the magazine
                        current_ammo_id = parent
                            .spawn(TextBundle {
                                text: Text::from_section(
                                    match gun_stats.shots_before_reload {
                                        0 => "∞".to_string(),
                                        _ => gun_stats.shots_before_reload.to_string(),
                                    },
                                    TextStyle {
                                        font: readable_font.clone(),
                                        font_size: readable_font_size,
                                        color,
                                    },
                                ),
                                style: Style {
                                    margin: UiRect::bottom(Val::Px(15.)),
                                    ..default()
                                },
                                ..default()
                            })
                            .id();

                        // Vertical line of fire cooldown
                        fire_cooldown_id = parent
                            .spawn((NodeBundle {
                                style: Style {
                                    position_type: PositionType::Absolute,
                                    position: UiRect {
                                        left: Val::Px(10.),
                                        bottom: Val::Px(22.),
                                        ..default()
                                    },
                                    size: Size::new(
                                        Val::Px(6.),
                                        // Progress bar height
                                        Val::Px(0.0),
                                    ),
                                    ..default()
                                },
                                background_color: color.into(),
                                ..default()
                            },))
                            .id();

                        // Vertical line of reload progress
                        reload_id = parent
                            .spawn((NodeBundle {
                                style: Style {
                                    position_type: PositionType::Absolute,
                                    position: UiRect {
                                        right: Val::Px(10.),
                                        bottom: Val::Px(22.),
                                        ..default()
                                    },
                                    size: Size::new(
                                        Val::Px(6.),
                                        // Progress bar height
                                        Val::Px(0.0),
                                    ),
                                    ..default()
                                },
                                background_color: color.into(),
                                ..default()
                            },))
                            .id();
                    });

                    gun_display_panel.insert(GunDisplay {
                        gun_entity: guns[i],
                        name_display: name_id,
                        max_ammo_display: max_ammo_id,
                        ammo_display: current_ammo_id,
                        fire_cooldown_display: fire_cooldown_id,
                        reload_display: reload_id,
                    });
                }
            });
        }
    }
}

fn handle_guns_hud_update(
    mut gun_text_query: Query<&mut Text>,
    mut gun_style_query: Query<&mut Style>,
    gun_display_query: Query<&GunDisplay>,
    gear_query: Query<&Gun, (With<Equipped>, Changed<Gun>)>,
) {
    for display in gun_display_query.iter() {
        let gun = match gear_query.get(display.gun_entity) {
            Ok(gun) => gun,
            Err(_) => continue,
        };

        display.update_ammo_display(gun, &mut gun_text_query);
        display.update_fire_cooldown(gun, &mut gun_style_query);
        display.update_reload_display(gun, &mut gun_style_query);
    }
}

pub struct HUDPlugin;
impl Plugin for HUDPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup_player_health_hud.in_schedule(OnEnter(GameState::InGame)))
            .add_system(setup_player_guns_hud.in_schedule(OnEnter(GameState::InGame)))
            // .add_system(setup_player_names_hud.in_schedule(OnEnter(GameState::InGame)))
            .add_system(handle_health_hud.run_if(in_state(GameState::InGame)))
            .add_system(handle_guns_hud_setup_change.run_if(in_state(GameState::InGame)))
            .add_system(handle_guns_hud_update.run_if(in_state(GameState::InGame)))
            .add_system(despawn_node::<HUDElement>.in_schedule(OnExit(GameState::InGame)));
    }
}
