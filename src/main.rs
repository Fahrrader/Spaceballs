use bevy::prelude::*;
use bevy::render::primitives::{Frustum, Sphere};

#[derive(Component)]
struct Player {
    speed: f32,
    rotation_speed: f32,
    projectile_speed: f32,
}

impl Player {
    fn new(speed: f32, rotation_speed: f32, projectile_speed: f32) -> Self {
        Self {
            speed,
            rotation_speed,
            projectile_speed,
        }
    }
}

#[derive(Component)]
struct Bullet {
    velocity: Vec2,
}

impl Bullet {
    fn new(velocity: Vec2) -> Self {
        Self { velocity }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::CYAN,
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..default()
            },
            ..default()
        })
        .insert(Player::new(200.0, 3.0, 300.0));
}

fn handle_input(keys: Res<Input<KeyCode>>, mut input: ResMut<PlayerInput>) {
    input.up = keys.pressed(KeyCode::W) || keys.pressed(KeyCode::Up);
    input.down = keys.pressed(KeyCode::S) || keys.pressed(KeyCode::Down);
    input.left = keys.pressed(KeyCode::A) || keys.pressed(KeyCode::Left);
    input.right = keys.pressed(KeyCode::D) || keys.pressed(KeyCode::Right);
    input.fire = keys.pressed(KeyCode::Space);
}

fn handle_movement(
    time: Res<Time>,
    input: Res<PlayerInput>,
    mut query: Query<(&Player, &mut Transform)>,
) {
    let (player, mut transform) = query.single_mut();
    let dt = time.delta_seconds();

    transform.rotate(Quat::from_axis_angle(
        -Vec3::Z,
        input.angular_speed() * player.rotation_speed * dt,
    ));
    let d_x = transform.up() * input.speed() * player.speed * dt;
    transform.translation += d_x;
}

fn handle_fire(
    mut commands: Commands,
    input: Res<PlayerInput>,
    player_q: Query<(&mut Player, &Transform)>,
) {
    let (mut player, player_transform) = player_q.single();

    if input.fire {
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::ALICE_BLUE,
                    custom_size: Some(Vec2::new(5.0, 5.0)), // todo replace with constant
                    ..default()
                },
                transform: player_transform.clone(),
                ..default()
            })
            .insert(Bullet::new(
                player_transform.up().truncate() * player.projectile_speed,
            ));
    }
}

fn handle_bullets(
    mut commands: Commands,
    time: Res<Time>,
    mut query_bullets: Query<(&Bullet, &mut Transform, Entity)>,
    query_frustum: Query<&Frustum, With<Camera>>,
) {
    let dt = time.delta_seconds();

    let frustum = query_frustum.get_single().unwrap();

    for (bullet, mut transform, entity) in query_bullets.iter_mut() {
        transform.translation += bullet.velocity.extend(0.0) * dt;

        let model_sphere = Sphere {
            center: transform.translation.into(),
            radius: 5.0, // todo replace with constant
        };

        if !frustum.intersects_sphere(&model_sphere, false)
        {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Default)]
struct PlayerInput {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    fire: bool,
}

impl PlayerInput {
    fn speed(&self) -> f32 {
        let mut speed = 0.0;
        if self.up {
            speed += 1.0;
        }
        if self.down {
            speed -= 1.0;
        }
        speed
    }

    fn angular_speed(&self) -> f32 {
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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .init_resource::<PlayerInput>()
        .add_startup_system(setup)
        .add_system(handle_input)
        .add_system(handle_movement)
        .add_system(handle_fire)
        .add_system(handle_bullets)
        .run();
}
