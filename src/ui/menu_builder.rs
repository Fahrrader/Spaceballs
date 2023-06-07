//use crate::ui::menu::MenuButtonAction;
use crate::ui::{colors, ColorInteractionMap};
use bevy::prelude::*;

macro_rules! make_menu_building_environment {
    {$($(#[$doc:meta])? $field:ident: $typ:ty $(,)?)*} => {
        pub(crate) struct MenuBuildingEnvironment {
            $(
                $(#[$doc])?
                pub $field: $typ,
            )*
            pub temporaries: TempMenuBuildingEnvironment,
        }

        #[derive(Clone, Debug, Default)]
        pub(crate) struct TempMenuBuildingEnvironment {
            $(
                $(#[$doc])?
                pub $field: Option<$typ>,
            )*
        }

        impl MenuBuildingEnvironment {
            pub fn unite_with_temporaries(&mut self) -> Self {
                Self {
                    $(
                        $field: $crate::get!(self.$field),
                    )*
                    temporaries: default(),
                }
                //self.take_temporaries();
            }
        }
    };
}

make_menu_building_environment! {
    // maybe do something about privacy here
    // pub(crate) asset_server: &'a ResMut<'a, AssetServer>,
    font: Handle<Font>,
    /// Generally the background color of a UI node, use with caution with alpha stacking.
    node_background_color: Color,
    layout_width: MaybeDefault<Val>,
    layout_height: MaybeDefault<Val>,
    layout_alignment: MaybeDefault<AlignItems>,
    layout_own_alignment: MaybeDefault<AlignSelf>,
    layout_content: MaybeDefault<JustifyContent>,
    text_font_size: f32,
    button_font_size: f32,
    button_size: Size,
    button_margin: UiRect,
    text_color: Color,
    button_color: Color,
    /// If None, will not be changed on interaction
    button_hovered_color: Option<Color>,
    /// If None, will not be changed on interaction
    button_pressed_color: Option<Color>,
    button_text_color: InheritedColor,
    /// If None, will not be changed on interaction
    button_text_hovered_color: Option<InheritedColor>,
    /// If None, will not be changed on interaction
    button_text_pressed_color: Option<InheritedColor>,
    outline_width: Val,
}

impl MenuBuildingEnvironment {
    pub fn default(asset_server: &ResMut<AssetServer>) -> Self {
        let font = asset_server.load("fonts/Spacerunner.otf");
        let text_font_size = 30.0;
        let button_font_size = text_font_size;

        let button_size = Size::new(Val::Px(390.0), Val::Px(65.0));
        let button_margin = UiRect::all(Val::Px(8.0));
        let outline_width = Val::Px(4.0);

        Self {
            // asset_server,
            font,
            node_background_color: Color::NONE,
            layout_width: MaybeDefault::Default,
            layout_height: MaybeDefault::Default,
            layout_alignment: MaybeDefault::Default,
            layout_own_alignment: MaybeDefault::Default,
            layout_content: MaybeDefault::Default,
            text_font_size,
            button_font_size,
            button_size,
            button_margin,
            text_color: colors::AERO_BLUE,
            button_color: Color::YELLOW_GREEN,
            button_hovered_color: Some(colors::AERO_BLUE),
            button_pressed_color: Some(Color::CYAN),
            button_text_color: InheritedColor::Inherit,
            button_text_hovered_color: Some(InheritedColor::Inherit),
            button_text_pressed_color: Some(InheritedColor::Inherit),
            outline_width,
            temporaries: TempMenuBuildingEnvironment::default(),
        }
    }

    pub fn reset_temporaries(&mut self) {
        self.temporaries = default()
    }

    // pub fn load_asset<T: Asset, P: Into<AssetPath<'a>>>(&self, path: P) -> Handle<T> {
    //     self.asset_server.load(path)
    // }
}

#[doc(hidden)]
#[macro_export]
macro_rules! get {
    ($environment:ident.font) => {
        $environment.temporaries.font
            .clone()
            .unwrap_or_else(|| $environment.font.clone())
    };
    ($environment:ident.$variable:ident) => {
        $environment.temporaries.$variable.unwrap_or($environment.$variable)
    };
    ($environment:ident, $($variable:ident $(,)?)*) => {
        ($(
            $crate::get!($environment.$variable),
        )*)
    };
    ($environment:ident) => {
        $environment.unite_with_temporaries()
    };
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum InheritedColor {
    Inherit,
    Own(Color),
}

impl From<Color> for InheritedColor {
    fn from(color: Color) -> Self {
        InheritedColor::Own(color)
    }
}

impl InheritedColor {
    pub fn try_resolve(&self, parent_color: Option<Color>) -> Result<Color, &str> {
        match self {
            InheritedColor::Own(color) => Ok(*color),
            InheritedColor::Inherit => parent_color.ok_or(
                "Did not provide parent color, while the inherited color does not have its `Own`.",
            ),
        }
    }

    pub fn resolve(&self, parent_color: Color) -> Color {
        match self {
            InheritedColor::Own(color) => *color,
            InheritedColor::Inherit => parent_color,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) enum MaybeDefault<T: Default + Copy> {
    Some(T),
    #[default]
    Default,
}

impl<T: Default + Copy> MaybeDefault<T> {
    pub fn get_or_default(&self) -> T {
        match self {
            MaybeDefault::Some(x) => *x,
            MaybeDefault::Default => T::default(),
        }
    }

    pub fn get_or(&self, default: T) -> T {
        match self {
            MaybeDefault::Some(x) => *x,
            MaybeDefault::Default => default,
        }
    }
}

impl<T: Default + Copy> From<T> for MaybeDefault<T> {
    fn from(value: T) -> Self {
        MaybeDefault::Some(value)
    }
}

#[macro_export]
macro_rules! build_menu_plugin {
    (($system_name:ident, $menu:ident), $($body:tt)*) => {
        $crate::build_menu_system!(($system_name, $menu), $($body)*);

        impl Plugin for SingleMenuPlugin<menu_state::$menu> {
            fn build(&self, app: &mut App) {
                app
                    .add_system($system_name.in_schedule(OnEnter(MenuState::$menu)))
                    .add_system(despawn_node::<OnMenu<menu_state::$menu>>.in_schedule(OnExit(MenuState::$menu)))
                ;
            }
        }
    };
}

#[macro_export]
macro_rules! build_menu_system {
    (($system_name:ident, $menu:ident), $($body:tt)*) => {
        fn $system_name(mut commands: Commands, asset_server: ResMut<AssetServer>) {
            // Configuration values
            #[allow(unused_mut)]
            let mut menu_env = $crate::ui::menu_builder::MenuBuildingEnvironment::default(&asset_server);
            $crate::build_layout!(commands, menu_env, Screen, (OnMenu::<menu_state::$menu>::default(),), { $($body)* });
        }
    };
}

#[macro_export]
macro_rules! build_menu {
    ($commands:expr, $asset_server:expr, $($body:tt)*) => {
        // Configuration values
        let menu_env = $crate::ui::menu_builder::MenuBuildingEnvironment::default(&$asset_server);
        $crate::build_menu_item!($commands, menu_env, $($body)*);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! build_menu_item {
    // Creating a text field out of sections
    ($parent:expr, $menu_shared_vars:ident, Text [ $($text:tt)* ], $($rest:tt)*) => {
        $crate::build_text!($parent, $menu_shared_vars, $($text)*);
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
    // Changing one of the shared menu building variables for the current scope -- but only once
    ($parent:expr, $menu_shared_vars:ident, once $shared_menu_var:ident = $new_value:expr, $($rest:tt)*) => {
        $crate::change_menu_environment_context!($parent, $menu_shared_vars, once $shared_menu_var = $new_value);
        $crate::build_menu_item!($parent, $menu_shared_vars, $($rest)*);
    };
    // Creating a menu layout
    ($parent:expr, $menu_shared_vars:ident, $layout:ident { $($body:tt)* }, $($rest:tt)*) => {
        $crate::build_layout!($parent, $menu_shared_vars, $layout, (), { $($body)* });
        $crate::build_menu_item!($parent, $menu_shared_vars, $($rest)*);
    };
    // Creating a menu layout with extra components attached to the node
    ($parent:expr, $menu_shared_vars:ident, $layout:ident + ($($extra_component:expr,)*) { $($body:tt)* }, $($rest:tt)*) => {
        $crate::build_layout!($parent, $menu_shared_vars, $layout, ($($extra_component,)*), { $($body)* });
        $crate::build_menu_item!($parent, $menu_shared_vars, $($rest)*);
    };
    // Spawning a custom bundle under the parent
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
        {
            $crate::build_menu_item!($parent, $menu_shared_vars, $($body)*);
        }
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
        #[allow(unused_mut)]
        let mut $menu_shared_vars = $crate::ui::menu_builder::MenuBuildingEnvironment {
            font: $new_value,
            temporaries: $menu_shared_vars.temporaries.clone(),
            ..$menu_shared_vars
        };
    };
    ($parent:expr, $menu_shared_vars:ident, $shared_menu_var:ident = $new_value:expr) => {
        #[allow(unused_mut)]
        let mut $menu_shared_vars = $crate::ui::menu_builder::MenuBuildingEnvironment {
            $shared_menu_var: $new_value,
            font: $menu_shared_vars.font.clone(),
            temporaries: $menu_shared_vars.temporaries.clone(),
            ..$menu_shared_vars
        };
    };
    ($parent:expr, $menu_shared_vars:ident, once $shared_menu_var:ident = $new_value:expr) => {
        $menu_shared_vars.temporaries.$shared_menu_var = Some($new_value);
        /*let $menu_shared_vars = $crate::ui::menu_builder::MenuBuildingEnvironment {
            font: $menu_shared_vars.font.clone(),
            temporaries: $crate::ui::menu_builder::TempMenuBuildingEnvironment {
                $shared_menu_var: $new_value,
                font: $menu_shared_vars.temporaries.font.clone(),
                ..$menu_shared_vars.temporaries
            }
            ..$menu_shared_vars
        };*/
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! build_layout {
    // Covering whole screen (or available area)
    ($parent:expr, $menu_shared_vars:ident, Screen, ($($extra_component:expr,)*), { $($body:tt)* }) => {
        let (layout_alignment, layout_content) = $crate::get!($menu_shared_vars, layout_alignment, layout_content);
        let style = Style {
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            align_items: layout_alignment.get_or_default(),
            justify_content: layout_content.get_or(JustifyContent::Center),
            padding: UiRect::all(Val::Percent(2.5)),
            ..default()
        };
        $crate::build_layout!($parent, $menu_shared_vars, style, ($($extra_component,)*), $($body)*);
    };
    // Creating a container at the top of the screen
    ($parent:expr, $menu_shared_vars:ident, Top, ($($extra_component:expr,)*), { $($body:tt)* }) => {
        $crate::build_layout!($parent, $menu_shared_vars, _quarter_screen, UiRect::top(Val::Percent(0.)), ($($extra_component,)*), $($body)*);
    };
    // Creating a container at the bottom of the screen
    ($parent:expr, $menu_shared_vars:ident, Bottom, ($($extra_component:expr,)*), { $($body:tt)* }) => {
        $crate::build_layout!($parent, $menu_shared_vars, _quarter_screen, UiRect::bottom(Val::Percent(0.)), ($($extra_component,)*), $($body)*);
    };
    // Built-in quarter-screen container
    ($parent:expr, $menu_shared_vars:ident, _quarter_screen, $position:expr, ($($extra_component:expr,)*), $($body:tt)*) => {
        let msv = $crate::get!($menu_shared_vars);
        let style = Style {
            size: Size::new(msv.layout_width.get_or_default(), msv.layout_height.get_or(Val::Percent(25.))),
            align_items: msv.layout_alignment.get_or(AlignItems::End),
            align_self: msv.layout_own_alignment.get_or_default(),
            justify_content: msv.layout_content.get_or(JustifyContent::SpaceEvenly),
            position_type: PositionType::Absolute,
            position: $position,
            margin: UiRect::all(Val::Percent(2.5)),
            ..default()
        };
        $crate::build_layout!($parent, $menu_shared_vars, style, ($($extra_component,)*), $($body)*);
    };
    // Creating a column
    ($parent:expr, $menu_shared_vars:ident, Column, ($($extra_component:expr,)*), { $($body:tt)* }) => {
        $crate::build_layout!($parent, $menu_shared_vars, _column, FlexDirection::Column, ($($extra_component,)*), $($body)*);
    };
    ($parent:expr, $menu_shared_vars:ident, ColumnReverse, ($($extra_component:expr,)*), { $($body:tt)* }) => {
        $crate::build_layout!($parent, $menu_shared_vars, _column, FlexDirection::ColumnReverse, ($($extra_component,)*), $($body)*);
    };
    // Built-in column
    ($parent:expr, $menu_shared_vars:ident, _column, $flex_direction:path, ($($extra_component:expr,)*), $($body:tt)*) => {
        let msv = $crate::get!($menu_shared_vars);
        let style = Style {
            size: Size::new(msv.layout_width.get_or_default(), msv.layout_height.get_or_default()),
            align_items: msv.layout_alignment.get_or(AlignItems::Center),
            align_self: msv.layout_own_alignment.get_or_default(),
            justify_content: msv.layout_content.get_or(JustifyContent::SpaceEvenly),
            flex_direction: $flex_direction,
            ..default()
        };
        $crate::build_layout!($parent, $menu_shared_vars, style, ($($extra_component,)*), $($body)*);
    };
    // Handling arranged layout's style
    ($parent:expr, $menu_shared_vars:ident, $style:expr, ($($extra_component:expr,)*), $($body:tt)*) => {
        let mut entity_commands = $parent.spawn((
            NodeBundle {
                style: $style,
                background_color: $crate::get!($menu_shared_vars.node_background_color).into(),
                ..default()
            }
            $(, $extra_component)*
        ));
        $menu_shared_vars.reset_temporaries();
        entity_commands.with_children(|parent| {
            $crate::build_menu_item!(parent, $menu_shared_vars, $($body)*);
        });
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! build_text {
    ($parent:expr, $menu_shared_vars:ident, $($body:tt)*) => {
        let mut sections = vec![];
        $crate::create_text_sections!($parent, $menu_shared_vars, sections, $($body)*);
        $parent.spawn(
            TextBundle::from_sections(sections)
                .with_text_alignment(TextAlignment::Center),
        );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! create_text_sections {
    // Handling shared variable changes
    ($parent:expr, $menu_shared_vars:ident, $sections:ident, $shared_menu_var:ident = $new_value:expr, $($rest:tt)*) => {
        $crate::change_menu_environment_context!($parent, $menu_shared_vars, $shared_menu_var = $new_value);
        $crate::create_text_sections!($parent, $menu_shared_vars, $sections, $($rest)*);
    };
    // Handling shared variable changes -- but only for a single use
    ($parent:expr, $menu_shared_vars:ident, $sections:ident, once $shared_menu_var:ident = $new_value:expr, $($rest:tt)*) => {
        $crate::change_menu_environment_context!($parent, $menu_shared_vars, once $shared_menu_var = $new_value);
        $crate::create_text_sections!($parent, $menu_shared_vars, $sections, $($rest)*);
    };
    // Creating nested blocks, thus offering ability to apply and afterwards revert changes to shared variables
    ($parent:expr, $menu_shared_vars:ident, $sections:ident, { $($body:tt)* }, $($rest:tt)*) => {
        {
            $crate::create_text_sections!($parent, $menu_shared_vars, $sections, $($body)*);
        }
        $crate::create_text_sections!($parent, $menu_shared_vars, $sections, $($rest)*);
    };
    ($parent:expr, $menu_shared_vars:ident, $sections:ident, $text:expr, $($rest:tt)*) => {
        let text_section = TextSection::new(
            $text,
            TextStyle {
                font_size: $crate::get!($menu_shared_vars.text_font_size),
                color: $crate::get!($menu_shared_vars.text_color),
                font: $crate::get!($menu_shared_vars.font),
            },
        );
        $sections.push(text_section);
        $menu_shared_vars.reset_temporaries();
        $crate::create_text_sections!($parent, $menu_shared_vars, $sections, $($rest)*);
    };
    ($parent:expr, $menu_shared_vars:ident, $sections:ident,) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! build_buttons {
    //                                                                 $(+ $extra:tt)*, )*) => { ...
    ($parent:expr, $menu_shared_vars:ident, $(($action:expr, $text:expr),)*) => {
        let msv = $crate::get!($menu_shared_vars);
        $menu_shared_vars.reset_temporaries();

        let button_style = Style {
            size: msv.button_size,
            margin: msv.button_margin,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        };
        let button_text_style = TextStyle {
            font_size: msv.button_font_size,
            color: msv.button_text_color.resolve(msv.button_color),
            font: msv.font,
        };

        let button_outline_interaction_colors = if msv.button_hovered_color.is_some() || msv.button_pressed_color.is_some() {
            ColorInteractionMap::from(vec![
                (Interaction::None, Some(msv.button_color)),
                (Interaction::Hovered, msv.button_hovered_color),
                (Interaction::Clicked, msv.button_pressed_color),
            ]).into()
        } else { None };

        let button_text_interaction_colors = if msv.button_text_hovered_color.is_some() || msv.button_text_pressed_color.is_some() {
            ColorInteractionMap::from([
                (Interaction::None, msv.button_text_color.try_resolve(Some(msv.button_color)).ok()),
                (Interaction::Hovered, msv.button_text_hovered_color.and_then(|color| color.try_resolve(msv.button_hovered_color).ok())),
                (Interaction::Clicked, msv.button_text_pressed_color.and_then(|color| color.try_resolve(msv.button_pressed_color).ok())),
            ]).into()
        } else { None };

        let background_color = msv.node_background_color.into();
        let outline_width = msv.outline_width;
        let button_color = msv.button_color;
        $(
            let mut entity_commands = $parent.spawn((
                ButtonBundle {
                    style: button_style.clone(),
                    background_color,
                    ..default()
                },
                $action,
            ));

            if button_outline_interaction_colors.is_some() || button_text_interaction_colors.is_some() {
                entity_commands.insert(ColorInteractionMap::from([]));
            }

            entity_commands.with_children(|parent| {
                $crate::ui::menu_builder::outline_parent(parent, outline_width, button_color, button_outline_interaction_colors);

                let mut entity_commands = parent.spawn(TextBundle::from_section(
                    $text,
                    button_text_style.clone(),
                ));

                if let Some(states) = button_text_interaction_colors {
                    entity_commands.insert(states);
                }
            });
        )*
    };
}

/*
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
}*/

pub(crate) fn outline_parent(
    child_builder: &mut ChildBuilder,
    outline_width: Val,
    color: Color,
    color_states: Option<ColorInteractionMap>,
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
            entity_commander.insert(states);
        }
    };

    spawn_outline_bar(
        UiRect::top(Val::Percent(0.)),
        (Val::Percent(100.), outline_width).into(),
    );

    spawn_outline_bar(
        UiRect::top(Val::Percent(100.)),
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
}
