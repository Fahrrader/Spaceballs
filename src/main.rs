use bevy::prelude::*;


#[derive(Component)]
struct Player;


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
    }).insert(Player);
}

fn handle_input(keys: Res<Input<KeyCode>>, mut input: ResMut<PlayerInput>) {
    input.up    = keys.pressed(KeyCode::W) || keys.pressed(KeyCode::Up);
    input.down  = keys.pressed(KeyCode::S) || keys.pressed(KeyCode::Down);
    input.left  = keys.pressed(KeyCode::A) || keys.pressed(KeyCode::Left);
    input.right = keys.pressed(KeyCode::D) || keys.pressed(KeyCode::Right);
}

fn handle_movement(
    time: Res<Time>,
    input: Res<PlayerInput>,
    mut query: Query<&mut Transform, With<Player>>
) {
    let d_x = input.into_direction() * time.delta_seconds() * 300.0;

    query.single_mut().translation += d_x.extend(0.0);
}


#[derive(Default)]
struct PlayerInput {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

impl PlayerInput {
    fn into_direction(&self) -> Vec2 {
        let mut dir = Vec2::ZERO;
        if self.up {
            dir.y += 1.0;
        }
        if self.down {
            dir.y -= 1.0;
        }
        if self.left {
            dir.x -= 1.0;
        }
        if self.right {
            dir.x += 1.0;
        }
        dir.normalize_or_zero()
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
        .run();
}

