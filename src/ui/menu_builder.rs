use crate::ui::menu::{MenuButtonAction, DEFAULT_BUTTON_COLOR, TEXT_COLOR};
#[allow(unused_imports)]
use crate::ui::menu::{HOVERED_BUTTON_COLOR, PRESSED_BUTTON_COLOR};
use crate::ui::ColorInteractionMap;
use bevy::prelude::*;

pub(crate) struct MenuBuildingEnvironment<'a> {
    // maybe do something about privacy here
    // pub(crate) asset_server: &'a ResMut<'a, AssetServer>,
    pub font: Handle<Font>,
    /// Specifically the background color of a menu screen.
    pub menu_background_color: Color,
    /// Generally the background color of a UI node, use with caution with alpha stacking.
    pub node_background_color: Color,
    pub title: &'a str,
    pub text_font_size: f32,
    pub title_font_size: f32,
    pub button_font_size: f32,
    pub title_margin: UiRect,
    pub button_size: Size,
    pub button_margin: UiRect,
    pub text_color: Color,
    /// If None, uses [`button_color`]
    pub button_text_color: Option<Color>,
    /// If None, uses [`button_hovered_color`]
    pub button_text_hovered_color: Option<Color>,
    /// If None, uses [`button_pressed_color`]
    pub button_text_pressed_color: Option<Color>,
    /// If None, uses [`DEFAULT_BUTTON_COLOR`]
    pub button_color: Option<Color>,
    /// If None, uses [`HOVERED_BUTTON_COLOR`]
    pub button_hovered_color: Option<Color>,
    /// If None, uses [`PRESSED_BUTTON_COLOR`]
    pub button_pressed_color: Option<Color>,
    pub outline_width: Val,
}

impl<'a> MenuBuildingEnvironment<'a> {
    pub fn default(asset_server: &'a ResMut<'a, AssetServer>) -> Self {
        let font = asset_server.load("fonts/Spacerunner.otf");
        let text_font_size = 40.0;
        let button_font_size = text_font_size;
        let title_font_size = 60.0;

        let title_margin = UiRect::all(Val::Px(50.0));
        let button_size = Size::new(Val::Px(390.0), Val::Px(65.0));
        let button_margin = UiRect::all(Val::Px(4.0));
        let outline_width = Val::Px(4.0);

        Self {
            // asset_server,
            font,
            menu_background_color: Color::NONE,
            node_background_color: Color::NONE,
            title: "Cosmic\nSpaceball\nTactical Action Arena".into(),
            text_font_size,
            title_font_size,
            button_font_size,
            title_margin,
            button_size,
            button_margin,
            text_color: TEXT_COLOR,
            button_text_color: None,
            button_text_hovered_color: None,
            button_text_pressed_color: None,
            button_color: None,
            button_hovered_color: None,
            button_pressed_color: None,
            outline_width,
        }
    }

    // pub fn load_asset<T: Asset, P: Into<AssetPath<'a>>>(&self, path: P) -> Handle<T> {
    //     self.asset_server.load(path)
    // }
}

#[macro_export]
macro_rules! build_menu_system {
    ($system:ident, $($body:tt)*) => {
        fn $system(mut commands: Commands, asset_server: ResMut<AssetServer>) {
            // Configuration values
            let menu_env = $crate::ui::menu_builder::MenuBuildingEnvironment::default(&asset_server);

            $crate::build_menu_item!(commands, menu_env, $($body)*);
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! build_menu_item {
    // Handling a Column layout
    ($parent:expr, $menu_shared_vars:ident, Column { $($body:tt)* }, $($rest:tt)*) => {
        // Create the column element and recursively add its the body as its children
        $crate::build_column!($parent, $menu_shared_vars, $($body)*);
        // Recurse on the rest
        $crate::build_menu_item!($parent, $menu_shared_vars, $($rest)*);
    };
    // Printing the game's name
    ($parent:expr, $menu_shared_vars:ident, Title, $($rest:tt)*) => {
        $crate::build_title!($parent, $menu_shared_vars);
        $crate::build_menu_item!($parent, $menu_shared_vars, $($rest)*);
    };
    // Handling buttons, their action component and text
    ($parent:expr, $menu_shared_vars:ident, Buttons [ $($buttons:tt)* ], $($rest:tt)*) => {
        $crate::build_buttons!($parent, $menu_shared_vars, $($buttons)*);
        $crate::build_menu_item!($parent, $menu_shared_vars, $($rest)*);
    };
    // Changing one of the shared menu building variables for the current scope
    ($parent:expr, $menu_shared_vars:ident, $shared_menu_var:ident = $new_value:expr, $($rest:tt)*) => {
        $crate::change_menu_environment_context!($parent, $menu_shared_vars, $shared_menu_var = $new_value);
        $crate::build_menu_item!($parent, $menu_shared_vars, $($rest)*);
    };
    // Creating menu screen to cover the whole window
    ($parent:expr, $menu_shared_vars:ident, $menu:ident { $($body:tt)* }, $($rest:tt)*) => {
        $crate::build_menu_screen!($parent, $menu_shared_vars, $menu, $($body)*);
        $crate::build_menu_item!($parent, $menu_shared_vars, $($rest)*);
    };
    // Adding a custom bundle to the parent
    ($parent:expr, $menu_shared_vars:ident, ($custom_bundle:expr), $($rest:tt)*) => {
        $parent.spawn($custom_bundle);
        $crate::build_menu_item!($parent, $menu_shared_vars, $($rest)*);
    };
    // extra components?
    // ...
    /*($parent:expr, $menu_shared_vars:ident, $($rest:tt)*) => {
        $crate::build_menu_item!($parent, $menu_shared_vars $($rest)*);
    };*/

    // Creating nested blocks, thus offering ability to apply and afterwards revert changes to shared variables
    ($parent:expr, $menu_shared_vars:ident, { $($body:tt)* }, $($rest:tt)*) => {
        //$parent.spawn(NodeBundle::default()).with_children(|parent| {
        {
            $crate::build_menu_item!(parent, $menu_shared_vars, $($body)*);
        }
        //});
        $crate::build_menu_item!($parent, $menu_shared_vars, $($rest)*);
    };
    // Exiting when there are no more tokens
    ($parent:expr, $menu_shared_vars:ident $(,)*) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! change_menu_environment_context {
    // Separate `font`, as it does not implement the Copyable trait
    ($parent:expr, $menu_shared_vars:ident, font = $new_value:expr) => {
        let $menu_shared_vars = $crate::ui::menu_builder::MenuBuildingEnvironment {
            font: $new_value,
            ..$menu_shared_vars
        };
    };
    ($parent:expr, $menu_shared_vars:ident, $shared_menu_var:ident = $new_value:expr) => {
        let $menu_shared_vars = $crate::ui::menu_builder::MenuBuildingEnvironment {
            $shared_menu_var: $new_value,
            font: $menu_shared_vars.font.clone(),
            ..$menu_shared_vars
        };
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! build_menu_screen {
    ($parent:expr, $menu_shared_vars:ident, $menu:ident, $($body:tt)*) => {
        $parent.spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: $menu_shared_vars.menu_background_color.into(),
                ..default()
            },
            OnMenu::<menu_state::$menu>::default(),
        ))
        .with_children(|parent| {
            $crate::build_menu_item!(parent, $menu_shared_vars, $($body)*);
        });
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! build_column {
    ($parent:expr, $menu_shared_vars:ident, $($body:tt)*) => {
        $parent.spawn(
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: $menu_shared_vars.node_background_color.into(),
                ..default()
            }
        )
        .with_children(|parent| {
            $crate::build_menu_item!(parent, $menu_shared_vars, $($body)*);
        });
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! build_title {
    ($parent:expr, $menu_shared_vars:ident) => {
        let title_text_style = TextStyle {
            font_size: $menu_shared_vars.title_font_size,
            color: $menu_shared_vars.text_color,
            font: $menu_shared_vars.font.clone(),
        };

        $parent.spawn(
            TextBundle::from_section($menu_shared_vars.title, title_text_style)
                .with_style(Style {
                    margin: $menu_shared_vars.title_margin,
                    ..default()
                })
                .with_text_alignment(TextAlignment::Center),
        );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! build_buttons {
    ($parent:expr, $menu_shared_vars:ident, $(($action:expr, $text:expr),)*) => {
        let button_color = $menu_shared_vars.button_color.unwrap_or(DEFAULT_BUTTON_COLOR);

        let button_style = Style {
            size: $menu_shared_vars.button_size,
            margin: $menu_shared_vars.button_margin,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        };
        let button_text_style = TextStyle {
            font_size: $menu_shared_vars.button_font_size,
            color: $menu_shared_vars.button_text_color.unwrap_or(button_color),
            font: $menu_shared_vars.font.clone(),
        };
        $(
            let mut entity_commands = $parent.spawn((
                ButtonBundle {
                    style: button_style.clone(),
                    background_color: $menu_shared_vars.node_background_color.into(),
                    ..default()
                },
                $action,
            ));
            let color_states = if $menu_shared_vars.button_color.is_some() || $menu_shared_vars.button_hovered_color.is_some() || $menu_shared_vars.button_pressed_color.is_some() {
                let states = [
                    (Interaction::None, $menu_shared_vars.button_color),
                    (Interaction::Hovered, $menu_shared_vars.button_hovered_color),
                    (Interaction::Clicked, $menu_shared_vars.button_pressed_color),
                ];

                states.into()
            } else { None };

            entity_commands.with_children(|parent| {
                let mut entity_commands = parent.spawn(TextBundle::from_section(
                    $text,
                    button_text_style.clone(),
                ));

                let states = [
                    (Interaction::None, $menu_shared_vars.button_text_color.or($menu_shared_vars.button_color)),
                    (Interaction::Hovered, $menu_shared_vars.button_text_hovered_color.or($menu_shared_vars.button_hovered_color)),
                    (Interaction::Clicked, $menu_shared_vars.button_text_pressed_color.or($menu_shared_vars.button_pressed_color)),
                ];
                if states.iter().map(|(_, opt)| opt).any(Option::is_some) {
                    entity_commands.insert(ColorInteractionMap::new(states.iter().copied()));
                }

                $crate::ui::menu_builder::outline_parent(parent, $menu_shared_vars.outline_width, button_color, color_states);
            });
        )*
    };
}

#[doc(hidden)]
pub(crate) fn build_scene_select_grid(
    parent: &mut ChildBuilder,
    button_style: Style,
    button_text_style: TextStyle,
) {
    for (index, scene_name) in [String::from("Lite"), String::from("Experimental")]
        .iter()
        .enumerate()
    {
        parent
            .spawn((
                ButtonBundle {
                    style: button_style.clone(),
                    background_color: Color::NONE.into(),
                    ..default()
                },
                MenuButtonAction::SelectScene(index),
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    scene_name,
                    button_text_style.clone(),
                ));

                outline_parent(parent, Val::Px(4.), DEFAULT_BUTTON_COLOR, None);
            });
    }
}

pub(crate) fn outline_parent(
    child_builder: &mut ChildBuilder,
    outline_width: Val,
    color: Color,
    color_states: Option<[(Interaction, Option<Color>); 3]>,
) {
    let background_color = color.into();

    let mut spawn_outline_bar = |position: UiRect, size: Size| {
        let mut entity_commander = child_builder.spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position,
                size,
                // rendering for borders is not yet implemented
                // border: UiRect::all(outline_width),
                ..default()
            },
            background_color,
            ..default()
        });
        if let Some(states) = color_states {
            entity_commander.insert(ColorInteractionMap::new(states.iter().copied()));
        }
    };

    spawn_outline_bar(
        UiRect::top(Val::Percent(0.)),
        (Val::Percent(100.), outline_width).into(),
    );

    spawn_outline_bar(
        UiRect::left(Val::Percent(0.)),
        (outline_width, Val::Percent(100.)).into(),
    );

    spawn_outline_bar(
        UiRect::right(Val::Percent(0.)),
        (outline_width, Val::Percent(100.)).into(),
    );

    spawn_outline_bar(
        UiRect::bottom(Val::Percent(0.)),
        (Val::Percent(100.), outline_width).into(),
    );
}
