use crate::MenuState;
use bevy::prelude::*;

/// Tag component used to mark highlighted and focusable entities.
/// Since it is generic, it allows for multiple focuses with different contexts on an entity at once.
///
/// Beware that its uniqueness is expected, but might not be enforced outside of this crate.
#[derive(Component, Clone, Copy, Debug)]
pub enum Focus<Context = ()> {
    /// Not focused, but focusable.
    None,
    /// Focused with some optional context
    Focused(Option<Context>),
}

impl<Context> Focus<Context> {
    /// Returns `true` if the `Focus` is a `None` value (not focused on).
    pub fn is_none(&self) -> bool {
        matches!(self, Focus::None)
    }

    /// Wraps the incoming `Context` argument into an `Option` and returns `Focus::Focused`.
    pub fn focused(focus_point: Context) -> Self {
        Self::Focused(Some(focus_point))
    }

    /// Returns the contained `Context` value, if present.
    pub fn extract_context(self) -> Option<Context> {
        match self {
            Focus::Focused(context) => context,
            Focus::None => None,
        }
    }
}

/// Event signifying a new entity has been focused on, and the other entities should probably switch to `Focus::None`.
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

/// System to mark all entities with `Focus<Context>` non-focused, except for the entity from newly received events.
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

/// Handle changing all buttons' colors based on mouse interaction.
fn transfer_focus_on_interaction(
    mut interaction_query: Query<
        (Entity, &Interaction, &mut Focus<Interaction>),
        Changed<Interaction>,
    >,
    mut focus_switch_events: EventWriter<FocusSwitchedEvent<Interaction>>,
) {
    interaction_query.for_each_mut(|(entity, interaction, mut focus)| {
        let focused_entity = match interaction {
            Interaction::Clicked | Interaction::Hovered => {
                *focus = Focus::focused(interaction.clone());
                Some(entity)
            }
            _ => None,
        };

        focus_switch_events.send(FocusSwitchedEvent::new(focused_entity));
    });
}

/// Plugin handling the [`Focus`] and [`FocusSwitchedEvent`] systems for the default ([`Interaction`]) generic components.
pub struct FocusPlugin;

impl Plugin for FocusPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FocusSwitchedEvent<Interaction>>()
            .add_systems((
                transfer_focus_on_interaction.run_if(not(in_state(MenuState::Disabled))),
                remove_focus_from_non_focused_entities::<Interaction>
                    .run_if(not(in_state(MenuState::Disabled))),
            ));
    }
}
