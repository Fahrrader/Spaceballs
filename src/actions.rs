use bevy::prelude::Component;

/// Inputs for characters to act on during the next frame.
#[derive(Component, Clone, Copy, Default)]
pub struct CharacterActionInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,

    pub fire: bool,
    pub reload: bool,

    pub use_environment_1: bool,
    pub use_environment_2: bool,
}

impl CharacterActionInput {
    /// Get direction of linear speed.
    pub fn speed(&self) -> f32 {
        let mut speed = 0.0;
        if self.up {
            speed += 1.0;
        }
        if self.down {
            speed -= 1.0;
        }
        speed
    }

    /// Get direction of rotational speed.
    pub fn angular_speed(&self) -> f32 {
        let mut angle = 0.0;
        if self.left {
            angle -= 1.0
        }
        if self.right {
            angle += 1.0
        }
        angle
    }
}
