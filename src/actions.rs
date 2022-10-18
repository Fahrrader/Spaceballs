use bevy::prelude::Component;

#[derive(Component, Default)]
pub struct CharacterActionInput {
    /// Forward movement. Clamp to 1.0!
    pub up: f32,
    /// Right-hand movement. Clamp to 1.0!
    pub right: f32,
    /// To shoot or not to shoot? That is the question.
    pub fire: bool,
}

impl CharacterActionInput {
    pub fn reset(&mut self) {
        self.up = 0.0;
        self.right = 0.0;
        self.fire = false;
    }

    pub fn speed(&self) -> f32 {
        self.up.clamp(-1.0, 1.0)
    }

    pub fn angular_speed(&self) -> f32 {
        self.right.clamp(-1.0, 1.0)
    }
}
