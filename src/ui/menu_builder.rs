use crate::ui::color_interaction::ColorInteractionMap;
use crate::ui::colors;
use bevy::prelude::*;

pub const DEFAULT_TEXT_COLOR: Color = colors::AERO_BLUE;
pub const DEFAULT_BUTTON_COLOR: Color = Color::YELLOW_GREEN;
pub const DEFAULT_BUTTON_HOVERED_COLOR: Color = colors::AERO_BLUE;
pub const DEFAULT_BUTTON_PRESSED_COLOR: Color = Color::CYAN;

pub const DEFAULT_FONT_SIZE: f32 = 27.0;

pub const DEFAULT_OUTLINE_THICKNESS: f32 = 3.0;
pub const DEFAULT_BUTTON_MARGIN: f32 = DEFAULT_OUTLINE_THICKNESS * 2.;
pub const DEFAULT_TEXT_INPUT_MARGIN: f32 = 7.0;

pub mod fonts {
    use bevy::prelude::{AssetServer, Font, Handle};

    pub const ULTRAGONIC: &str = "Ultragonicv2.otf";
    pub const SPACERUNNER: &str = "Spacerunner.otf";

    /// Load a font using `asset_server` and `font_file` located in fonts/ directory.
    pub fn load(asset_server: &AssetServer, font_file: &str) -> Handle<Font> {
        asset_server.load(format!("fonts/{}", font_file))
    }
}

pub const DEFAULT_FONT: &str = fonts::ULTRAGONIC;

macro_rules! make_menu_building_environment {
    {$($(#[$doc:meta])? $field:ident: $typ:ty $(,)?)*} => {
        /// Set of shared variables used by the menu-building macros.
        pub(crate) struct MenuBuildingEnvironment<'a> {
            $(
                $(#[$doc])?
                pub $field: $typ,
            )*
            pub asset_server: &'a AssetServer,
            pub temporaries: TempMenuBuildingEnvironment,
        }

        /// Set of shared variables used by the menu-building macros, but only once.
        /// Meant to be used as part of [`MenuBuildingEnvironment`], and to be reset when it's been used.
        #[derive(Clone, Debug, Default)]
        pub(crate) struct TempMenuBuildingEnvironment {
            $(
                $(#[$doc])?
                pub $field: Option<$typ>,
            )*
        }

        impl MenuBuildingEnvironment<'_> {
            /// Get a new copy of [`MenuBuildingEnvironment`], joined with its [`TempMenuBuildingEnvironment`],
            /// where fields that are present in the temporary environment take precedence.
            pub fn unite_with_temporaries(&mut self) -> Self {
                Self {
                    $(
                        $field: $crate::get!(self.$field),
                    )*
                    asset_server: self.asset_server,
                    temporaries: default(),
                }
            }
        }
    };
}

make_menu_building_environment! {
    font: Handle<Font>,
    /// Generally the background color of a UI node, use with caution with alpha stacking.
    node_color: Color,
    layout_width: MaybeDefault<Val>,
    layout_height: MaybeDefault<Val>,
    align_items: MaybeDefault<AlignItems>,
    align_self: MaybeDefault<AlignSelf>,
    justify_content: MaybeDefault<JustifyContent>,
    margin: MaybeDefault<UiRect>,
    text_font_size: f32,
    button_font_size: f32,
    button_width: Val,
    button_height: Val,
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

impl<'a> MenuBuildingEnvironment<'a> {
    pub fn default(asset_server: &'a AssetServer) -> Self {
        let font = fonts::load(asset_server, DEFAULT_FONT);
        let text_font_size = DEFAULT_FONT_SIZE;
        let button_font_size = text_font_size;

        let button_width = Val::Px(420.0);
        let button_height = Val::Px(65.0);
        let button_margin = UiRect::all(Val::Px(DEFAULT_BUTTON_MARGIN));
        let outline_width = Val::Px(DEFAULT_OUTLINE_THICKNESS);

        Self {
            asset_server,
            font,
            node_color: Color::NONE,
            layout_width: MaybeDefault::Default,
            layout_height: MaybeDefault::Default,
            align_items: MaybeDefault::Default,
            align_self: MaybeDefault::Default,
            justify_content: MaybeDefault::Default,
            margin: MaybeDefault::Default,
            text_font_size,
            button_font_size,
            button_width,
            button_height,
            button_margin,
            text_color: DEFAULT_TEXT_COLOR,
            button_color: DEFAULT_BUTTON_COLOR,
            button_hovered_color: Some(DEFAULT_BUTTON_HOVERED_COLOR),
            button_pressed_color: Some(DEFAULT_BUTTON_PRESSED_COLOR),
            button_text_color: InheritedColor::Inherit,
            button_text_hovered_color: Some(InheritedColor::Inherit),
            button_text_pressed_color: Some(InheritedColor::Inherit),
            outline_width,
            temporaries: TempMenuBuildingEnvironment::default(),
        }
    }

    /// Explicitly reset own [`TempMenuBuildingEnvironment`].
    pub fn reset_temporaries(&mut self) {
        self.temporaries = default();
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
        ( $($crate::get!($environment.$variable),)* )
    };
    ($environment:ident) => {
        $environment.unite_with_temporaries()
    };
}

/// Determines whether a color is its own unique color or should be inherited from a parent node.
#[derive(Clone, Copy, Debug)]
pub(crate) enum InheritedColor {
    /// Should inherit the color from a parent.
    Inherit,
    /// Has its own unique color.
    Own(Color),
}

impl From<Color> for InheritedColor {
    fn from(color: Color) -> Self {
        InheritedColor::Own(color)
    }
}

impl InheritedColor {
    /// Tries to resolve the inherited color, falling back on a parent color if necessary.
    pub fn try_resolve(&self, parent_color: Option<Color>) -> Result<Color, &str> {
        match self {
            InheritedColor::Own(color) => Ok(*color),
            InheritedColor::Inherit => parent_color.ok_or(
                "Did not provide parent color, while the inherited color does not have its `Own`.",
            ),
        }
    }

    /// Resolves the inherited color, using a parent color if necessary.
    pub fn resolve(&self, parent_color: Color) -> Color {
        match self {
            InheritedColor::Own(color) => *color,
            InheritedColor::Inherit => parent_color,
        }
    }
}

/// Generic enum that allows a type to either be a specific value or a default.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) enum MaybeDefault<T: Default + Copy> {
    /// Has a specific value.
    Some(T),
    /// Uses the default value for the type `T`.
    #[default]
    Default,
}

impl<T: Default + Copy> MaybeDefault<T> {
    /// Returns the value if `Some(T)`, or the default value of `T` otherwise.
    pub fn get_or_default(&self) -> T {
        match self {
            MaybeDefault::Some(x) => *x,
            MaybeDefault::Default => T::default(),
        }
    }

    /// Returns the value if `Some(T)`, or the provided value otherwise.
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
macro_rules! process_tokens {
    // Creating a text field out of sections
    ($parent:expr, $menu_shared_vars:ident, $token:ident [ $($nest:tt)* ], $($rest:tt)*) => {
        $crate::build_menu_item!($parent, $menu_shared_vars, $token, (), $($nest)*);
        $crate::process_tokens!($parent, $menu_shared_vars, $($rest)*);
    };
    // Creating a text field out of sections
    ($parent:expr, $menu_shared_vars:ident, $token:ident + ($($extra_components:expr,)*) [ $($nest:tt)* ], $($rest:tt)*) => {
        $crate::build_menu_item!($parent, $menu_shared_vars, $token, ($($extra_components,)*), $($nest)*);
        $crate::process_tokens!($parent, $menu_shared_vars, $($rest)*);
    };
    // Creating a text field out of sections with optional components
    ($parent:expr, $menu_shared_vars:ident, $token:ident [ $($nest:tt)* ] + ($($extra_components:expr,)*), $($rest:tt)*) => {
        $crate::build_menu_item!($parent, $menu_shared_vars, $token, ($($extra_components,)*), $($nest)*);
        $crate::process_tokens!($parent, $menu_shared_vars, $($rest)*);
    };
    // Creating a text field out of sections
    ($parent:expr, $menu_shared_vars:ident, $token:ident { $($nest:tt)* }, $($rest:tt)*) => {
        $crate::build_menu_item!($parent, $menu_shared_vars, $token, (), $($nest)*);
        $crate::process_tokens!($parent, $menu_shared_vars, $($rest)*);
    };
    // Creating a text field out of sections
    ($parent:expr, $menu_shared_vars:ident, $token:ident + ($($extra_components:expr,)*) { $($nest:tt)* }, $($rest:tt)*) => {
        $crate::build_menu_item!($parent, $menu_shared_vars, $token, ($($extra_components,)*), $($nest)*);
        $crate::process_tokens!($parent, $menu_shared_vars, $($rest)*);
    };
    // Creating a text field out of sections with optional components
    ($parent:expr, $menu_shared_vars:ident, $token:ident { $($nest:tt)* } + ($($extra_components:expr,)*), $($rest:tt)*) => {
        $crate::build_menu_item!($parent, $menu_shared_vars, $token, ($($extra_components,)*), $($nest)*);
        $crate::process_tokens!($parent, $menu_shared_vars, $($rest)*);
    };
    // Spawning a custom bundle under the parent
    ($parent:expr, $menu_shared_vars:ident, ($custom_bundle:expr), $($rest:tt)*) => {
        $parent.spawn($custom_bundle);
        $crate::process_tokens!($parent, $menu_shared_vars, $($rest)*);
    };
    // Changing one of the shared menu building variables for the current scope
    ($parent:expr, $menu_shared_vars:ident, $shared_menu_var:ident = $new_value:expr, $($rest:tt)*) => {
        $crate::change_menu_environment_context!($parent, $menu_shared_vars, $shared_menu_var = $new_value);
        $crate::process_tokens!($parent, $menu_shared_vars, $($rest)*);
    };
    // Changing one of the shared menu building variables for the current scope -- but only once
    ($parent:expr, $menu_shared_vars:ident, once $shared_menu_var:ident = $new_value:expr, $($rest:tt)*) => {
        $crate::change_menu_environment_context!($parent, $menu_shared_vars, once $shared_menu_var = $new_value);
        $crate::process_tokens!($parent, $menu_shared_vars, $($rest)*);
    };
    // Changing one of the shared menu building variables for the current scope -- but only once
    ($parent:expr, $menu_shared_vars:ident, #[$($attributes:tt)*] $($rest:tt)*) => {
        #[$($attributes)*]
        $crate::process_tokens!($parent, $menu_shared_vars, $($rest)*);
    };
    // Creating nested blocks, thus offering ability to apply and afterwards revert changes to shared variables
    ($parent:expr, $menu_shared_vars:ident, { $($body:tt)* }, $($rest:tt)*) => {
        {
            $crate::process_tokens!($parent, $menu_shared_vars, $($body)*);
        }
        $crate::process_tokens!($parent, $menu_shared_vars, $($rest)*);
    };
    // Exiting when there are no more tokens
    ($parent:expr, $menu_shared_vars:ident $(,)*) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! build_menu_item {
    // Creating a text field out of sections
    ($parent:expr, $menu_shared_vars:ident, Text, ($($extra_components:expr,)*), $($text:tt)*) => {
        $crate::build_text!($parent, $menu_shared_vars, ($($extra_components,)*), $($text)*);
    };
    // Creating a text input field
    ($parent:expr, $menu_shared_vars:ident, TextInput, ($($extra_components:expr,)*), $($initial_text:tt)*) => {
        $crate::build_text_input!($parent, $menu_shared_vars, ($($extra_components,)*), TextInput::default().max_symbols, "", $($initial_text)*);
    };
    // Handling buttons, their action component and text
    ($parent:expr, $menu_shared_vars:ident, Buttons, (), $($buttons:tt)*) => {
        $crate::build_buttons!($parent, $menu_shared_vars, $($buttons)*);
    };
    // Creating a menu layout
    ($parent:expr, $menu_shared_vars:ident, $layout:ident, ($($extra_component:expr,)*), $($body:tt)*) => {
        $crate::build_layout!($parent, $menu_shared_vars, $layout, ($($extra_component,)*), { $($body)* });
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! change_menu_environment_context {
    // Separate `font`, as it does not implement the Copyable trait
    ($parent:expr, $menu_shared_vars:ident, font = $new_value:expr) => {
        #[allow(unused_mut)]
        let mut $menu_shared_vars = $crate::ui::menu_builder::MenuBuildingEnvironment {
            font: fonts::load($menu_shared_vars.asset_server, $new_value),
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
    ($parent:expr, $menu_shared_vars:ident, once font = $new_value:expr) => {
        $menu_shared_vars.temporaries.font =
            Some(fonts::load($menu_shared_vars.asset_server, $new_value));
    };
    ($parent:expr, $menu_shared_vars:ident, once $shared_menu_var:ident = $new_value:expr) => {
        $menu_shared_vars.temporaries.$shared_menu_var = Some($new_value);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! build_layout {
    // Covering whole screen (or available area)
    ($parent:expr, $menu_shared_vars:ident, Screen, ($($extra_component:expr,)*), { $($body:tt)* }) => {
        let msv = $crate::get!($menu_shared_vars);
        let style = Style {
            size: Size::new(msv.layout_width.get_or(Val::Percent(100.)), msv.layout_height.get_or(Val::Percent(100.))),
            align_items: msv.align_items.get_or_default(),
            align_self: msv.align_self.get_or_default(),
            justify_content: msv.justify_content.get_or(JustifyContent::Center),
            padding: UiRect::all(Val::Percent(2.5)),
            margin: msv.margin.get_or_default(),
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
    // Creating a container at the bottom of the screen
    ($parent:expr, $menu_shared_vars:ident, Left, ($($extra_component:expr,)*), { $($body:tt)* }) => {
        $crate::build_layout!($parent, $menu_shared_vars, _quarter_screen, UiRect::left(Val::Percent(0.)), ($($extra_component,)*), $($body)*);
    };
    // Creating a container at the bottom of the screen
    ($parent:expr, $menu_shared_vars:ident, Right, ($($extra_component:expr,)*), { $($body:tt)* }) => {
        $crate::build_layout!($parent, $menu_shared_vars, _quarter_screen, UiRect::right(Val::Percent(0.)), ($($extra_component,)*), $($body)*);
    };
    // Built-in quarter-screen container
    ($parent:expr, $menu_shared_vars:ident, _quarter_screen, $position:expr, ($($extra_component:expr,)*), $($body:tt)*) => {
        let msv = $crate::get!($menu_shared_vars);

        let width = if matches!($position.left, Val::Undefined) && matches!($position.right, Val::Undefined) {
            msv.layout_width.get_or_default()
        } else {
            msv.layout_width.get_or(Val::Percent(25.))
        };
        let height = if matches!($position.top, Val::Undefined) && matches!($position.bottom, Val::Undefined) {
            msv.layout_height.get_or_default()
        } else {
            msv.layout_height.get_or(Val::Percent(25.))
        };
        let alignment = if let Val::Undefined = $position.top { AlignItems::End } else { AlignItems::Start };

        let style = Style {
            size: Size::new(width, height),
            align_items: msv.align_items.get_or(alignment),
            align_self: msv.align_self.get_or(AlignSelf::Center),
            justify_content: msv.justify_content.get_or(JustifyContent::Center),
            position_type: PositionType::Absolute,
            position: $position,
            margin: msv.margin.get_or(UiRect::all(Val::Percent(2.5))),
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
            align_items: msv.align_items.get_or(AlignItems::Center),
            align_self: msv.align_self.get_or_default(),
            justify_content: msv.justify_content.get_or(JustifyContent::Center),
            flex_direction: $flex_direction,
            margin: msv.margin.get_or_default(),
            ..default()
        };
        $crate::build_layout!($parent, $menu_shared_vars, style, ($($extra_component,)*), $($body)*);
    };
    ($parent:expr, $menu_shared_vars:ident, Node, ($($extra_component:expr,)*), { $($body:tt)* }) => {
        let msv = $crate::get!($menu_shared_vars);
        let style = Style {
            size: Size::new(msv.layout_width.get_or_default(), msv.layout_height.get_or_default()),
            align_items: msv.align_items.get_or_default(),
            align_self: msv.align_self.get_or_default(),
            justify_content: msv.justify_content.get_or_default(),
            margin: msv.margin.get_or_default(),
            ..default()
        };
        $crate::build_layout!($parent, $menu_shared_vars, style, ($($extra_component,)*), $($body)*);
    };
    // Handling arranged layout's style
    ($parent:expr, $menu_shared_vars:ident, $style:expr, ($($extra_component:expr,)*), $($body:tt)*) => {
        let mut entity_commands = $parent.spawn((
            NodeBundle {
                style: $style,
                background_color: $crate::get!($menu_shared_vars.node_color).into(),
                ..default()
            }
            $(, $extra_component)*
        ));
        $menu_shared_vars.reset_temporaries();
        entity_commands.with_children(|parent| {
            $crate::process_tokens!(parent, $menu_shared_vars, $($body)*);
        });
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! build_text {
    ($parent:expr, $menu_shared_vars:ident, ($($extra_components:expr,)*), $($body:tt)*) => {
        let mut sections = vec![];
        let bundle_msv = $menu_shared_vars.unite_with_temporaries();
        $crate::create_text_sections!($parent, $menu_shared_vars, sections, $($body)*);
        $parent.spawn((
            TextBundle::from_sections(sections)
                .with_text_alignment(TextAlignment::Center)
                .with_style(Style {
                    align_items: bundle_msv.align_items.get_or_default(),
                    align_self: bundle_msv.align_self.get_or_default(),
                    justify_content: bundle_msv.justify_content.get_or_default(),
                    margin: bundle_msv.margin.get_or(UiRect::all(Val::Percent(2.5))),
                    ..default()
                }),
            $($extra_components,)*
        ));
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! build_text_input {
    ($parent:expr, $menu_shared_vars:ident, ($($extra_components:expr,)*), $max_symbols:expr, $placeholder:expr, max_symbols: $new_max_symbols:expr, $($body:tt)*) => {
        $crate::build_text_input!($parent, $menu_shared_vars, ($($extra_components,)*), $new_max_symbols, $placeholder, $($body)*)
    };
    ($parent:expr, $menu_shared_vars:ident, ($($extra_components:expr,)*), $max_symbols:expr, $placeholder:expr, placeholder: $new_placeholder:expr, $($body:tt)*) => {
        $crate::build_text_input!($parent, $menu_shared_vars, ($($extra_components,)*), $max_symbols, $new_placeholder, $($body)*)
    };
    ($parent:expr, $menu_shared_vars:ident, ($($extra_components:expr,)*), $max_symbols:expr, $placeholder:expr, $($body:tt)*) => {
        let mut sections = vec![];
        let bundle_msv = $menu_shared_vars.unite_with_temporaries();
        $crate::create_text_sections!($parent, $menu_shared_vars, sections, $($body)*);

        let input_section = sections.pop().unwrap_or(TextSection::new(
            "",
            TextStyle {
                font_size: $crate::get!($menu_shared_vars.text_font_size),
                color: $crate::get!($menu_shared_vars.text_color),
                font: $crate::get!($menu_shared_vars.font),
            },
        ));

        let button_outline_interaction_colors = if bundle_msv.button_hovered_color.is_some() || bundle_msv.button_pressed_color.is_some() {
            ColorInteractionMap::from(vec![
                (Interaction::None, Some(bundle_msv.button_color)),
                (Interaction::Hovered, bundle_msv.button_hovered_color),
            ]).into()
        } else { None };

        $parent.spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(bundle_msv.button_width, bundle_msv.button_height),
                    align_items: bundle_msv.align_items.get_or_default(),
                    align_self: bundle_msv.align_self.get_or_default(),
                    justify_content: bundle_msv.justify_content.get_or_default(),
                    margin: bundle_msv.margin.get_or(UiRect::all(Val::Percent(2.5))),
                    ..default()
                },
                background_color: bundle_msv.node_color.into(),
                ..default()
            },
        )).with_children(|parent| {
            $crate::ui::menu_builder::outline_parent(parent, bundle_msv.outline_width, bundle_msv.button_color, button_outline_interaction_colors);

            #[allow(unused_variables, unused_mut)]
            let mut text_commands = parent.spawn((
                TextBundle {
                    text: Text {
                        sections,
                        // bevy's UI is a bit shit at the moment.
                        // if you wanna do any alignment that isn't left, you have to adjust the size and the parent's justify_content,
                        // since the text always starts from the left of the size. and if you do, Interaction's going to shit itself.
                        alignment: TextAlignment::Left,
                        linebreak_behaviour: bevy::text::BreakLineOn::AnyCharacter,
                    },
                    style: Style {
                        size: Size::new(
                            bundle_msv.button_width,
                            if matches!(bundle_msv.button_height, Val::Px(_)) {
                                Val::Px((bundle_msv.button_height.evaluate(0.).unwrap() / bundle_msv.text_font_size).floor() * bundle_msv.text_font_size)
                            } else {
                                bundle_msv.button_height
                            },
                        ),
                        margin: UiRect::all(Val::Px(crate::ui::menu_builder::DEFAULT_TEXT_INPUT_MARGIN)),
                        ..default()
                    },
                    ..default()
                },
                TextInput::new(input_section.value, input_section.style, Some($placeholder.into()))
                    .with_max_symbols($max_symbols),
                Interaction::None,
                Focus::<Interaction>::None,
                Focus::<TextInput>::None,
            ));

            $(
                text_commands.insert($extra_components);
            )*
        });
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
            size: Size::new(msv.button_width, msv.button_height),
            margin: msv.button_margin,
            justify_content: msv.justify_content.get_or(JustifyContent::Center),
            align_items: msv.align_items.get_or(AlignItems::Center),
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

        let background_color = BackgroundColor::from(msv.node_color);
        let outline_width = msv.outline_width;
        let button_color = msv.button_color;
        // let margin = msv.margin.get_or(UiRect::all(Val::Percent(2.5)));
        $(
            let mut entity_commands = $parent.spawn((
                ButtonBundle {
                    style: button_style.clone(),
                    background_color,
                    ..default()
                },
                Focus::<Interaction>::None,
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
                ).with_text_alignment(TextAlignment::Center));

                if let Some(states) = button_text_interaction_colors {
                    entity_commands.insert(states);
                }
            });
        )*
    };
}

/// Create outlining bar elements at the edges of the parent node.
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
                align_self: AlignSelf::Start,
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
        UiRect::bottom(Val::Px(-1.)),
        (Val::Percent(100.), outline_width).into(),
    );

    spawn_outline_bar(
        UiRect::left(Val::Percent(0.)),
        (outline_width, Val::Percent(100.)).into(),
    );

    spawn_outline_bar(
        UiRect::right(Val::Percent(0.)),
        (outline_width, Val::Percent(100.1)).into(),
    );
}
