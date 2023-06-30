use crate::controls::CharacterActionInput;
use bevy::core::{Pod, Zeroable};
use bevy::reflect::{FromReflect, Reflect};

/// Players' input data structure, used and encoded by GGRS and exchanged over the internet.
#[derive(Copy, Clone, Debug, Default, PartialEq, Pod, Zeroable, Reflect, FromReflect)]
#[repr(C)]
pub struct GGRSInput {
    pub up: f32,
    pub right: f32,
    // bytemuck::Pod does not accept "padding"/uninit bytes,
    // therefore fields must make up a multiple of the byte size of the biggest field
    pub bit_flags: u32,
}

impl GGRSInput {
    pub const FIRE: u32 = 1 << 0;
    pub const RELOAD: u32 = 1 << 1;
    pub const INTERACT_1: u32 = 1 << 2;
    pub const INTERACT_2: u32 = 1 << 3;
}

impl Into<GGRSInput> for CharacterActionInput {
    fn into(self) -> GGRSInput {
        let mut input = GGRSInput::default();

        input.up = self.up;
        input.right = self.right;

        if self.fire {
            input.bit_flags |= GGRSInput::FIRE;
        }
        if self.reload {
            input.bit_flags |= GGRSInput::RELOAD;
        }
        if self.interact_1 {
            input.bit_flags |= GGRSInput::INTERACT_1;
        }
        if self.interact_2 {
            input.bit_flags |= GGRSInput::INTERACT_2;
        }

        input
    }
}

impl From<GGRSInput> for CharacterActionInput {
    fn from(value: GGRSInput) -> Self {
        CharacterActionInput {
            up: value.up,
            right: value.right,
            fire: value.bit_flags & GGRSInput::FIRE != 0,
            reload: value.bit_flags & GGRSInput::RELOAD != 0,
            interact_1: value.bit_flags & GGRSInput::INTERACT_1 != 0,
            interact_2: value.bit_flags & GGRSInput::INTERACT_2 != 0,
        }
    }
}
