use crate::ui::focus::Focus;
use crate::{MenuState, SceneSelector};
use bevy::ecs::query::WorldQuery;
use bevy::prelude::*;

/// Atlas component serving to provide colors for each different [`Interaction`] with the entity.
///
/// [`None`] means the entity should not react to this interaction variant.
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct ColorInteractionMap {
    default: Option<Color>,
    selected: Option<Color>,
    clicked: Option<Color>,
}

impl ColorInteractionMap {
    /// Returns `ColorInteractionMap` formed from a pseudo-map of `Interactions` and their corresponding `Options<Color>`.
    pub fn new(states: impl IntoIterator<Item = (Interaction, Option<Color>)>) -> Self {
        let mut map = Self::default();

        for (interaction, maybe_color) in states {
            match interaction {
                Interaction::None => map.default = maybe_color,
                Interaction::Hovered => map.selected = maybe_color,
                Interaction::Clicked => map.clicked = maybe_color,
            }
        }

        map
    }

    /// Returns the color of the corresponding `Interaction`.
    pub const fn get(&self, state: Interaction) -> Option<&Color> {
        match state {
            Interaction::None => self.default.as_ref(),
            Interaction::Hovered => self.selected.as_ref(),
            Interaction::Clicked => self.clicked.as_ref(),
        }
    }

    /// Returns `true` if any of the colors in the map equals to the `color` argument.
    pub fn has_color(&self, color: Color) -> bool {
        self.default == Some(color) || self.selected == Some(color) || self.clicked == Some(color)
    }
}

impl<T: IntoIterator<Item = (Interaction, Option<Color>)>> From<T> for ColorInteractionMap {
    fn from(states: T) -> Self {
        Self::new(states)
    }
}

#[derive(WorldQuery)]
#[world_query(mutable)]
#[doc(hidden)]
pub struct ColoredTextQuery {
    text: &'static mut Text,
    color_interaction_map: Option<&'static ColorInteractionMap>,
}

#[derive(WorldQuery)]
#[world_query(mutable)]
#[doc(hidden)]
pub struct ColoredNodeQuery {
    color: &'static mut BackgroundColor,
    children: Option<&'static Children>,
    color_interaction_map: Option<&'static ColorInteractionMap>,
}

/// Handle changing all buttons' colors based on mouse interaction.
fn handle_button_style_change(
    interaction_query: Query<
        (
            &Interaction,
            Option<&Focus<Interaction>>,
            Option<&Focus<SceneSelector>>,
            Entity,
        ),
        (
            With<ColorInteractionMap>,
            Or<(
                Changed<Interaction>,
                Changed<Focus<Interaction>>,
                Changed<Focus<SceneSelector>>,
            )>,
        ),
    >,
    mut text_children_query: Query<ColoredTextQuery>,
    mut node_children_query: Query<ColoredNodeQuery>,
) {
    /// Extract color from the color interaction map, if present, and if the current color is in the map.
    fn extract_color(
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

    /// Try to get a color from the color interaction map, or return the current color.
    fn distill_color(
        interaction: Interaction,
        present_color: Color,
        color_interaction_map: Option<&ColorInteractionMap>,
    ) -> Color {
        color_interaction_map
            .and_then(|map| extract_color(interaction, map, present_color))
            .unwrap_or(present_color)
    }

    /// Recursively go over a vector of entities and its children,
    /// painting the entities that have a color interaction map according to the new interaction.
    fn paint_nodes(
        interaction: Interaction,
        children: &Vec<Entity>,
        text_children_query: &mut Query<ColoredTextQuery>,
        node_children_query: &mut Query<ColoredNodeQuery>,
    ) {
        for &child in children {
            if let Ok(ColoredTextQueryItem {
                mut text,
                color_interaction_map,
            }) = text_children_query.get_mut(child)
            {
                text.sections.iter_mut().for_each(|section| {
                    section.style.color =
                        distill_color(interaction, section.style.color, color_interaction_map);
                });
            }

            if let Ok(mut node) = node_children_query.get_mut(child) {
                *node.color =
                    distill_color(interaction, node.color.0, node.color_interaction_map).into();

                if let Some(more_children) = node.children {
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

    for (interaction, interaction_focus, scene_focus, entity) in interaction_query.iter() {
        let interaction = match (interaction, interaction_focus, scene_focus) {
            // Highest priority: if anything is Clicked, we're Clicked
            (&Interaction::Clicked, _, _)
            | (_, Some(&Focus::Focused(Some(Interaction::Clicked))), _) => Interaction::Clicked,

            // Next priority: if interaction or interaction_focus is Hovered or if there is a focused scene, we're Hovered
            (&Interaction::Hovered, _, _)
            | (_, Some(&Focus::Focused(Some(Interaction::Hovered))), _)
            | (_, _, Some(&Focus::Focused(_))) => Interaction::Hovered,

            // Lowest priority: if nothing above matched, we're None
            _ => Interaction::None,
        };

        paint_nodes(
            interaction,
            &vec![entity],
            &mut text_children_query,
            &mut node_children_query,
        );
    }
}

/// Plugin handling the [`Focus`] and [`FocusSwitchedEvent`] systems for the default ([`Interaction`]) generic components.
pub(crate) struct ColorInteractionPlugin;

impl Plugin for ColorInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(handle_button_style_change.run_if(not(in_state(MenuState::Disabled))));
    }
}
