use bevy::prelude::Component;

/// Inputs for characters to act on during the next frame.
#[derive(Component, Clone, Copy, Default)]
pub struct CharacterActionInput {
    /// Forward movement. Clamp to 1.0!
    pub up: f32,
    /// Right-hand movement. Clamp to 1.0!
    pub right: f32,
    /// To shoot or not to shoot? That is the question.
    pub fire: bool,
    /// Whether a reload must be triggered this frame.
    pub reload: bool,

    /// Whether an environmental interactive action must be triggered this frame,
    /// such as picking guns up from the ground.
    pub use_environment_1: bool,
    /// Whether an auxiliary environmental interactive action must be triggered this frame,
    /// such as throwing equipped guns away.
    pub use_environment_2: bool,
}

impl CharacterActionInput {
    /// Bring every input to the default state.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Get direction of linear speed.
    pub fn speed(&self) -> f32 {
        self.up.clamp(-1.0, 1.0)
    }

    /// Get direction of rotational speed.
    pub fn angular_speed(&self) -> f32 {
        self.right.clamp(-1.0, 1.0)
    }
}
