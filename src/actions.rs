use bevy::prelude::Component;

/// Inputs for characters to act on during the next frame.
#[derive(Component, Default)]
pub struct CharacterActionInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub fire: bool,
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

    /// Copy data from other input data (used primarily for components)
    pub fn replace_from(&mut self, another: &Self) -> &mut Self {
        self.up = another.up;
        self.down = another.down;
        self.left = another.left;
        self.right = another.right;
        self.fire = another.fire;
        self
    }
}
