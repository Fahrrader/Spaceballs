use crate::controls::ActionInput;
use crate::characters::{PlayerControlled, CHARACTER_RAD_SPEED, CHARACTER_SPEED};
use bevy::math::{Quat, Vec3};
use bevy::prelude::{Component, Query, Res, Time, Transform, With};

#[derive(Component)]
pub struct Velocity {
    // todo look into rapier first
// also possibly replace with Option to send and parse less data // would be pretty with Rapier
// linear: Vec2,
// angular: f32,
}

pub trait GetVelocity {
    fn get_velocity(&self) -> Velocity;
}

// todo handle movement for all moving entities -- perm velocity for bullets, depending on action input for chars
pub fn handle_movement(
    time: Res<Time>,
    input: Res<ActionInput>,
    mut query: Query<&mut Transform, With<PlayerControlled>>,
) {
    let dt = time.delta_seconds();

    for mut transform in query.iter_mut() {
        transform.rotate(Quat::from_axis_angle(
            -Vec3::Z,
            input.angular_speed() * CHARACTER_RAD_SPEED * dt,
        ));
        let dx = transform.up() * input.speed() * CHARACTER_SPEED * dt;
        transform.translation += dx;
    }
}
