use crate::movement::{GetVelocity, Velocity};
use bevy::input::Input;
use bevy::prelude::{Component, KeyCode, Res, ResMut};

#[derive(Component, Default)]
pub struct ActionInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub fire: bool,
}

impl ActionInput {
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

impl GetVelocity for ActionInput {
    fn get_velocity(&self) -> Velocity {
        // transform?
        Velocity {
            // linear:
            //angular: self.angular_speed(),
        }
    }
}

pub fn handle_player_input(keys: Res<Input<KeyCode>>, mut input: ResMut<ActionInput>) {
    input.up = keys.pressed(KeyCode::W) || keys.pressed(KeyCode::Up);
    input.down = keys.pressed(KeyCode::S) || keys.pressed(KeyCode::Down);
    input.left = keys.pressed(KeyCode::A) || keys.pressed(KeyCode::Left);
    input.right = keys.pressed(KeyCode::D) || keys.pressed(KeyCode::Right);
    input.fire = keys.pressed(KeyCode::Space);
}
