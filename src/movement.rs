use bevy::math::{Quat, Vec3};
use bevy::prelude::{Component, Query, Res, Time, Transform};

#[derive(Component, Default)]
pub struct Velocity {
    // todo look into rapier first
    // also possibly replace with Option to send and parse less data // would be pretty with Rapier
    pub linear: Vec3,
    pub angular: f32,
}

impl Velocity {
    pub fn stop(&mut self) {
        self.linear = Vec3::default();
        self.angular = 0.0;
    }
}

pub fn handle_movement(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    let dt = time.delta_seconds();

    for (mut transform, velocity) in query.iter_mut() {
        transform.rotate(Quat::from_axis_angle(-Vec3::Z, velocity.angular * dt));
        let dx = velocity.linear * dt;
        transform.translation += dx;
    }
}
