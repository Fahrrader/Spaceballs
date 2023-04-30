use crate::controls::GgrsInput;
use bevy::prelude::Component;
use bevy::reflect::{FromReflect, Reflect};

// todo conjoin with 'controls'
/// Inputs for characters to act on during the next frame.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Reflect, FromReflect)]
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

const INPUT_UP: u8 = 1 << 0;
const INPUT_DOWN: u8 = 1 << 1;
const INPUT_LEFT: u8 = 1 << 2;
const INPUT_RIGHT: u8 = 1 << 3;
const INPUT_FIRE: u8 = 1 << 4;
const INPUT_RELOAD: u8 = 1 << 5;
const INPUT_ENV_1: u8 = 1 << 6;
const INPUT_ENV_2: u8 = 1 << 7;

// todo:mp displace the following into 'multiplayer' module
impl Into<GgrsInput> for CharacterActionInput {
    fn into(self) -> GgrsInput {
        let mut input = GgrsInput::default();
        if self.up > 0.0 {
            input |= INPUT_UP;
        }
        if self.up < 0.0 {
            input |= INPUT_DOWN;
        }
        if self.right > 0.0 {
            input |= INPUT_RIGHT;
        }
        if self.right < 0.0 {
            input |= INPUT_LEFT;
        }
        if self.fire {
            input |= INPUT_FIRE;
        }
        if self.reload {
            input |= INPUT_RELOAD;
        }
        if self.use_environment_1 {
            input |= INPUT_ENV_1;
        }
        if self.use_environment_2 {
            input |= INPUT_ENV_2;
        }
        input
    }
}

impl From<GgrsInput> for CharacterActionInput {
    fn from(value: GgrsInput) -> Self {
        CharacterActionInput {
            up: (value & INPUT_UP) as f32 - (value & INPUT_DOWN) as f32,
            right: (value & INPUT_RIGHT) as f32 - (value & INPUT_LEFT) as f32,
            fire: value & INPUT_FIRE != 0,
            reload: value & INPUT_RELOAD != 0,
            use_environment_1: value & INPUT_ENV_1 != 0,
            use_environment_2: value & INPUT_ENV_2 != 0,
        }
    }
}
