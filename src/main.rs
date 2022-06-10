use bevy::prelude::*;


#[derive(Component)]
struct Player {
    speed: f32,
    rotation_speed: f32,
}

impl Player {
    fn new(speed: f32, rotation_speed: f32) -> Self {
        Self {
            speed,
            rotation_speed,
        }
    }
}

#[derive(Component)]
struct Bullet;


fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::CYAN,
            custom_size: Some(Vec2::new(50.0, 50.0)),
            ..default()
        },
        transform: Transform::from_xyz(0.0, -300.0, 0.0),
        ..default()
    }).insert(Player::new(4.0, 3.0));
}

fn handle_input(keys: Res<Input<KeyCode>>, mut input: ResMut<PlayerInput>) {
    input.up    = keys.pressed(KeyCode::W) || keys.pressed(KeyCode::Up);
    input.down  = keys.pressed(KeyCode::S) || keys.pressed(KeyCode::Down);
    input.left  = keys.pressed(KeyCode::A) || keys.pressed(KeyCode::Left);
    input.right = keys.pressed(KeyCode::D) || keys.pressed(KeyCode::Right);
    input.fire   = keys.pressed(KeyCode::Space);
}

fn handle_movement(
    time: Res<Time>,
    input: Res<PlayerInput>,
    mut query: Query<(&Player, &mut Transform)>
) {
    let (player, mut transform) = query.single_mut();
    let dt = time.delta_seconds();

    transform.rotate(Quat::from_axis_angle(-Vec3::Z, input.angular_speed() * dt * player.rotation_speed));
    let d_x = transform.up() * input.speed() * player.speed;
    transform.translation += d_x;
}

fn handle_fire(mut commands: Commands, input: Res<PlayerInput>, player: Query<(&mut Player, &Transform)>) {
    let (mut _player, player_transform) = player.single();

    if input.fire {
        commands.spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::ALICE_BLUE,
                custom_size: Some(Vec2::new(5.0, 5.0)),
                ..default()
            },
            transform: player_transform.clone(),
            ..default()
        }).insert(Bullet);
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
        if self.up { speed += 1.0; }
        if self.down { speed -= 1.0; }
        speed
    }

    fn angular_speed(&self) -> f32 {
        let mut angle = 0.0;
        if self.left  { angle -= 1.0 }
        if self.right { angle += 1.0 }
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
        .run();
}

