use crate::ui::menu::MenuPlugin;
use crate::ui::text_input::TextInputPlugin;
use bevy::prelude::*;

pub mod menu;
mod menu_builder;
pub mod text_input;

pub use crate::ui::menu::MenuState;

/// Generic system that takes a component as a parameter, and will despawn all entities with that component.
fn despawn_node<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}

/// Tag component used to mark highlighted and focusable entities.
/// Since it is generic, it allows for multiple focuses with different contexts on an entity at once.
///
/// Beware that its uniqueness is expected, but might not be enforced outside of this crate.
#[derive(Component, Clone, Copy, Debug)]
pub enum Focus<Context = ()> {
    None,
    Focused(Context),
}

impl<Context> Focus<Context> {
    pub fn is_none(&self) -> bool {
        matches!(self, Focus::None)
    }

    pub fn extract_context(self) -> Option<Context> {
        match self {
            Focus::Focused(context) => Some(context),
            Focus::None => None,
        }
    }
}

#[derive(Default)]
pub struct FocusSwitchedEvent<Context> {
    pub new_focused_entity: Option<Entity>,
    _marker: std::marker::PhantomData<Context>,
}

impl<Context: Default> FocusSwitchedEvent<Context> {
    pub fn new(new_focused_entity: Option<Entity>) -> Self {
        Self {
            new_focused_entity,
            ..default()
        }
    }
}

pub fn remove_focus_from_non_focused_entities<Context: Send + Sync + 'static>(
    mut focus_change_events: EventReader<FocusSwitchedEvent<Context>>,
    mut focus_query: Query<(Entity, &mut Focus<Context>)>,
) {
    if focus_change_events.is_empty() {
        return;
    }

    let mut focused_entity = None;

    for event in focus_change_events.iter() {
        focused_entity = event.new_focused_entity;
    }

    for (entity, mut focus_input) in focus_query.iter_mut() {
        if Some(entity) != focused_entity {
            *focus_input = Focus::None;
        }
    }
}

pub struct SpaceballsUIPlugin;
impl Plugin for SpaceballsUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MenuPlugin).add_plugin(TextInputPlugin);
    }
}

#[allow(dead_code)]
pub mod colors {
    use bevy::prelude::Color;

    /// <div style="background-color:rgb(30.6%, 43.1%, 50.6%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const AEGEAN: Color = Color::rgb(0.306, 0.431, 0.506);
    /// <div style="background-color:rgb(78.8%, 100%, 89.8%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const AERO_BLUE: Color = Color::rgb(0.788, 1., 0.898);
    /// <div style="background-color:rgb(64.7%, 16.5%, 16.5%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const ALMOND: Color = Color::rgb(0.647, 0.165, 0.165);
    /// <div style="background-color:rgb(100.0%, 74.9%, 0.0%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const AMBER: Color = Color::rgb(1., 0.749, 0.);
    /// <div style="background-color:rgb(75.7%, 71%, 66.3%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const ASH_GRAY: Color = Color::rgb(0.757, 0.71, 0.663);
    /// <div style="background-color:rgb(94.9%, 67.5%, 72.5%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const BABY_PINK: Color = Color::rgb(0.949, 0.675, 0.725);
    /// <div style="background-color:rgb(62.4%, 50.6%, 43.9%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const BEAVER: Color = Color::rgb(0.624, 0.506, 0.439);
    /// <div style="background-color:rgb(71%, 65.1%, 25.9%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const BRASS: Color = Color::rgb(0.71, 0.651, 0.259);
    /// <div style="background-color:rgb(68.6%, 34.9%, 24.3%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const BROWN_RUST: Color = Color::rgb(0.686, 0.349, 0.243);
    /// <div style="background-color:rgb(73.3%, 53.3%, 33.3%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const CANVAS: Color = Color::rgb(0.733, 0.533, 0.333);
    /// <div style="background-color:rgb(0%, 48.2%, 65.5%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const CERULEAN: Color = Color::rgb(0., 0.482, 0.655);
    /// <div style="background-color:rgb(94.9%, 51%, 49.8%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const CORAL: Color = Color::rgb(0.949, 0.51, 0.498);
    /// <div style="background-color:rgb(59.6%, 41.2%, 37.6%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const DARK_CHESTNUT: Color = Color::rgb(0.596, 0.412, 0.376);
    /// <div style="background-color:rgb(3.1%, 57.3%, 81.6%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const ELECTRIC_BLUE: Color = Color::rgb(0.031, 0.573, 0.816);
    /// <div style="background-color:rgb(11%, 20.8%, 17.6%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const JUNGLE_GREEN: Color = Color::rgb(0.11, 0.208, 0.176);
    /// <div style="background-color:rgb(80%, 80%, 100%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const LAVENDER: Color = Color::rgb(0.8, 0.8, 1.0);
    /// <div style="background-color:rgb(100%, 95.7%, 31%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const LEMON: Color = Color::rgb(1., 0.957, 0.31);
    /// <div style="background-color:rgb(98.4%, 28.2%, 76.9%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const NEON_PINK: Color = Color::rgb(0.984, 0.282, 0.769);
    /// <div style="background-color:rgb(100%, 89.8%, 67.8%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const PEACH: Color = Color::rgb(1.0, 0.898, 0.678);
    /// <div style="background-color:rgb(10.98%, 22.35%, 73.33%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const PERSIAN_BLUE: Color = Color::rgb(0.1098, 0.2235, 0.7333);
    // /// <div style="background-color:rgb(40.0%, 20.0%, 80.0%); width: 10px; padding: 10px; border: 1px solid;"></div>
    // pub const YELLOW_GREEN_COMPLEMENT: Color = Color::rgb(0.4, 0.2, 0.8);
    /// <div style="background-color:rgb(72.9%,33.3%,82.7%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const ORCHID: Color = Color::rgb(0.729, 0.333, 0.827);
    /// <div style="background-color:rgb(0%, 12.9%, 27.8%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const OXFORD_BLUE: Color = Color::rgb(0., 0.129, 0.278);
    /// <div style="background-color:rgb(0%, 0.4%, 20%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const STRATOS: Color = Color::rgb(0., 0.004, 0.2);
    /// <div style="background-color:rgb(7.1%, 3.9%, 56.1%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const ULTRAMARINE: Color = Color::rgb(0.071, 0.039, 0.561);
    /// <div style="background-color:rgb(96.1%, 87.1%, 70.2%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const WHEAT: Color = Color::rgb(0.961, 0.871, 0.702);
}
