use bevy::prelude::*;

/// Resource that manages the activation status and the number of distinct levels of UI elements
/// that listen for user input within the game's UI system.
#[derive(Resource, Debug)]
pub struct ActiveInputConsumerLayers {
    /// Byte where each bit represents a layer, where 1 means active and 0 means inactive.
    active_layers: u8,
    /// Array counting the current number of users for each layer.
    layer_user_count: [usize; 8],
}

/// Component representing the input layer that an entity is set to consume.
/// The [`u8`] value represents the priority of the input layer, where a higher value means higher priority.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct InputConsumerPriority(u8);

impl ActiveInputConsumerLayers {
    /// Creates new `ActiveInputConsumerLayers` with no active layers.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            active_layers: 0u8,
            layer_user_count: [0; 8],
        }
    }

    /// Activates the specified layer, and increases the reference count.
    pub fn activate_layer(&mut self, layer: &InputConsumerPriority) {
        self.active_layers |= layer.0;
        let layer_num = layer.0.trailing_zeros() as usize;
        self.layer_user_count[layer_num] += 1;
    }

    /// Decreases the reference count for the specified layer and disables it, if it is 0.
    pub fn deactivate_layer(&mut self, layer: &InputConsumerPriority) {
        let layer_num = layer.0.trailing_zeros() as usize;
        if self.layer_user_count[layer_num] > 0 {
            self.layer_user_count[layer_num] -= 1;
            if self.layer_user_count[layer_num] == 0 {
                self.active_layers &= !layer.0; // deactivate layer only when no more users are using it.
            }
        }
    }

    /// Checks whether the specified layer has active users.
    #[must_use]
    pub const fn is_layer_active(&self, layer: &InputConsumerPriority) -> bool {
        self.active_layers & layer.0 != 0
    }

    /// Checks whether the specified layer has no other layer blocking it.
    #[must_use]
    pub const fn is_input_allowed_for_layer(&self, layer: &InputConsumerPriority) -> bool {
        self.active_layers.leading_zeros() >= layer.0.leading_zeros()
    }

    /// Checks whether the specified layer has another layer blocking it.
    #[must_use]
    pub const fn is_input_blocked_for_layer(&self, layer: &InputConsumerPriority) -> bool {
        self.active_layers.leading_zeros() < layer.0.leading_zeros()
    }

    /// Deactivates all layers.
    pub fn clear(&mut self) {
        *self = Self::new();
    }
}

impl InputConsumerPriority {
    /// Creates a new `InputConsumerPriority` with a single active layer.
    /// Panics if `layer` is greater than or equal to 8.
    #[must_use]
    pub const fn new(layer: u8) -> Self {
        assert!(layer < 8, "Layer must be less than 8");
        Self(1 << layer)
    }

    /// Returns the layer that this `InputConsumerPriority` is set to consume.
    #[must_use]
    pub const fn get_layer(&self) -> u8 {
        self.0.trailing_zeros() as u8
    }
}

impl Into<InputConsumerPriority> for u8 {
    fn into(self) -> InputConsumerPriority {
        InputConsumerPriority::new(self)
    }
}

/// System to handle changes in input consumption priorities.
/// Updates the [`ActiveInputConsumerLayers`] resource when a visibility of an entity containing an input consumer changes,
/// or the input consumer gets added or removed from an entity.
fn handle_input_consumption_priority_change(
    mut active_layers: ResMut<ActiveInputConsumerLayers>,
    consumer_added_query: Query<&InputConsumerPriority, Added<InputConsumerPriority>>,
    visibility_changed_query: Query<(&InputConsumerPriority, &Visibility), Changed<Visibility>>,
    removed_query: RemovedComponents<InputConsumerPriority>,
    consumer_query: Query<(&InputConsumerPriority, Option<&Visibility>)>,
) {
    if removed_query.is_empty() {
        for priority in consumer_added_query.iter() {
            active_layers.activate_layer(priority);
        }

        for (priority, visibility) in visibility_changed_query.iter() {
            match visibility {
                Visibility::Visible => active_layers.activate_layer(priority),
                Visibility::Hidden => active_layers.deactivate_layer(priority),
                // maybe should panic instead. But oopsie!
                Visibility::Inherited => active_layers.activate_layer(priority),
            }
        }
    } else {
        active_layers.clear();
        for (priority, maybe_visibility) in consumer_query.iter() {
            if maybe_visibility.map_or(true, |v| {
                // maybe should panic instead. But oopsie!
                matches!(*v, Visibility::Visible | Visibility::Inherited)
            }) {
                active_layers.activate_layer(priority);
            }
        }
    }
}

/// Plugin handling the [`ActiveInputConsumerLayers`] systems.
pub(crate) struct InputConsumptionPlugin;
impl Plugin for InputConsumptionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ActiveInputConsumerLayers::new())
            .add_system(handle_input_consumption_priority_change);
    }
}
