mod scenes;

pub use crate::scenes::{summon_scene, SceneArg};

pub use bevy::prelude::*;
use clap::Parser;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
pub fn create_window_descriptor(resolution: (f32, f32)) -> WindowDescriptor {
    let (width, height) = resolution;
    WindowDescriptor {
        width,
        height,
        scale_factor_override: Some(1.0),
        ..default()
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn create_window_descriptor(resolution: (f32, f32)) -> WindowDescriptor {
    let (width, height) = resolution;
    WindowDescriptor {
        width,
        height,
        ..default()
    }
}

#[derive(Parser)]
#[clap(version, about)]
struct Cli {
    /// The scene to load at the game start
    #[clap(value_enum, short, long)]
    scene: Option<SceneArg>,
}

#[cfg(not(target_arch = "wasm32"))]
pub fn parse_scene_ext_input() -> Option<SceneArg> {
    let args = Cli::parse();
    args.scene
}

#[cfg(target_arch = "wasm32")]
pub fn parse_scene_ext_input() -> Option<SceneArg> {
    get_scene_from_js().try_into().ok()
}

#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! log {
    () => (println!());
    ($($arg:tt)*) => ({
        println!($($arg)*)
    })
}

#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! log {
    () => (log("\n"));
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(module = "/public/main.js")]
extern "C" {
    #[wasm_bindgen(js_name = getSceneFromUrl)]
    fn get_scene_from_js() -> String;
}

// todo delete, replace with imports
use bevy::render::primitives::{Frustum, Sphere};
use bevy::sprite::collide_aabb::collide;
use std::time::Duration;

pub const CHARACTER_SIZE: f32 = 50.0;
pub const BULLET_SIZE: f32 = 5.0;

pub const CHARACTER_SPEED: f32 = 200.0;
pub const CHARACTER_RAD_SPEED: f32 = 5.0;
pub const PROJECTILE_SPEED: f32 = 300.0;

pub const CHARACTER_FIRE_COOLDOWN: Duration = Duration::from_millis(25);

pub const PLAYER_DEFAULT_TEAM: u8 = 0;
pub const AI_DEFAULT_TEAM: u8 = 8;

pub const CHARACTER_MAX_HEALTH: i8 = 100;
pub const BULLET_DAMAGE: i8 = 5;

#[derive(Bundle)]
pub struct NonPlayerCharacterBundle {
    character: Character,
    #[bundle]
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

impl NonPlayerCharacterBundle {
    pub fn new(team: u8, transform: Transform) -> Self {
        Self {
            character: Character { team, ..default() },
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: team_color(team),
                    custom_size: Some(Vec2::new(CHARACTER_SIZE, CHARACTER_SIZE)),
                    ..default()
                },
                transform,
                ..default()
            },
            collider: Collider,
        }
    }
}

#[derive(Bundle)]
pub struct ControlledPlayerCharacterBundle {
    #[bundle]
    character_bundle: NonPlayerCharacterBundle,
    player_controlled_marker: PlayerControlled,
}

impl ControlledPlayerCharacterBundle {
    pub fn new(team: u8, transform: Transform) -> Self {
        Self {
            character_bundle: NonPlayerCharacterBundle::new(team, transform),
            player_controlled_marker: PlayerControlled,
        }
    }
}

#[derive(Component)]
pub struct Character {
    team: u8,
    health: i8,
    fire_cooldown: Timer,
}

impl Default for Character {
    fn default() -> Self {
        Self {
            team: PLAYER_DEFAULT_TEAM,
            health: CHARACTER_MAX_HEALTH,
            fire_cooldown: Timer::new(CHARACTER_FIRE_COOLDOWN, false),
        }
    }
}

impl Character {
    pub fn damage(&mut self, damage: i8) -> bool {
        self.health -= damage;
        self.is_dead()
    }

    pub fn is_dead(&self) -> bool {
        self.health <= 0
    }
}

pub struct CharacterDamagedEvent {
    entity: Entity,
    damage: i8,
}

#[derive(Component)]
pub struct PlayerControlled;

pub fn team_color(team: u8) -> Color {
    match team {
        0 => Color::CYAN,
        1 => Color::CRIMSON,
        2 => Color::LIME_GREEN,
        3 => Color::GOLD,
        4 => Color::PURPLE,
        5 => Color::SEA_GREEN,
        6 => Color::ORANGE_RED,
        7 => Color::INDIGO,
        8 => Color::SILVER,
        _ => panic!("The team number is too big!"),
    }
}

#[derive(Bundle)]
pub struct BulletBundle {
    bullet: Bullet,
    #[bundle]
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

impl BulletBundle {
    pub fn new(team: u8, transform: Transform, velocity: Vec2) -> Self {
        Self {
            bullet: Bullet { team, velocity },
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: Color::ALICE_BLUE,
                    custom_size: Some(Vec2::new(BULLET_SIZE, BULLET_SIZE)),
                    ..default()
                },
                transform,
                ..default()
            },
            collider: Collider,
        }
    }
}

#[derive(Component)]
pub struct Bullet {
    team: u8,
    velocity: Vec2,
}

impl Bullet {
    pub fn stop(&mut self) {
        self.velocity = Vec2::default();
    }
}

#[derive(Component)]
pub struct Collider;

pub fn handle_input(keys: Res<Input<KeyCode>>, mut input: ResMut<PlayerInput>) {
    input.up = keys.pressed(KeyCode::W) || keys.pressed(KeyCode::Up);
    input.down = keys.pressed(KeyCode::S) || keys.pressed(KeyCode::Down);
    input.left = keys.pressed(KeyCode::A) || keys.pressed(KeyCode::Left);
    input.right = keys.pressed(KeyCode::D) || keys.pressed(KeyCode::Right);
    input.fire = keys.pressed(KeyCode::Space);
}

pub fn handle_movement(
    time: Res<Time>,
    input: Res<PlayerInput>,
    mut query: Query<&mut Transform, With<PlayerControlled>>,
) {
    let dt = time.delta_seconds();

    for mut transform in query.iter_mut() {
        transform.rotate(Quat::from_axis_angle(
            -Vec3::Z,
            input.angular_speed() * CHARACTER_RAD_SPEED * dt,
        ));
        let d_x = transform.up() * input.speed() * CHARACTER_SPEED * dt;
        transform.translation += d_x;
    }
}

pub fn handle_bullet_spawn(
    mut commands: Commands,
    time: Res<Time>,
    input: Res<PlayerInput>,
    mut query_characters: Query<(&mut Character, &Transform), With<PlayerControlled>>,
) {
    for (mut character, character_transform) in query_characters.iter_mut() {
        if character.fire_cooldown.tick(time.delta()).finished() && input.fire {
            commands.spawn_bundle(BulletBundle::new(
                character.team,
                character_transform.with_translation(
                    character_transform.translation
                        + character_transform.up()
                            * (CHARACTER_SIZE / 2.0
                                + BULLET_SIZE
                                + input.speed() * CHARACTER_SPEED * time.delta_seconds()),
                ),
                character_transform.up().truncate() * PROJECTILE_SPEED,
            ));

            character.fire_cooldown.reset();
        }
    }
}

pub fn handle_bullet_flight(
    mut commands: Commands,
    time: Res<Time>,
    mut query_bullets: Query<(&Bullet, &mut Transform, Entity)>,
    query_frustum: Query<&Frustum, With<Camera>>,
) {
    let dt = time.delta_seconds();

    let frustum = query_frustum.single();

    for (bullet, mut transform, entity) in query_bullets.iter_mut() {
        transform.translation += bullet.velocity.extend(0.0) * dt;

        let model_sphere = Sphere {
            center: transform.translation.into(),
            radius: BULLET_SIZE,
        };

        if !frustum.intersects_sphere(&model_sphere, false) {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn handle_bullet_collision(
    mut commands: Commands,
    mut query_bullets: Query<(&Bullet, &Transform, Entity), With<Collider>>,
    mut query_characters: Query<(&Character, &Transform, Entity), With<Collider>>,
    mut ew_damage: EventWriter<CharacterDamagedEvent>,
) {
    for (bullet, bullet_transform, bullet_entity) in query_bullets.iter_mut() {
        for (character, character_transform, character_entity) in query_characters.iter_mut() {
            let collision = collide(
                bullet_transform.translation,
                Vec2::new(BULLET_SIZE, BULLET_SIZE) * bullet_transform.scale.truncate(),
                character_transform.translation,
                Vec2::new(CHARACTER_SIZE, CHARACTER_SIZE) * character_transform.scale.truncate(),
            );

            if collision.is_some() {
                // perhaps send damage to bullets as well to handle multiple types / buffs?
                commands.entity(bullet_entity).despawn_recursive();
                if bullet.team != character.team {
                    ew_damage.send(CharacterDamagedEvent {
                        entity: character_entity,
                        damage: BULLET_DAMAGE,
                    })
                } else {
                    // friendly fire!
                }
            }
        }
    }
}

pub fn calculate_damages(
    mut commands: Commands,
    mut damage_events: EventReader<CharacterDamagedEvent>,
    mut query_characters: Query<&mut Character, With<Collider>>,
) {
    for event in damage_events.iter() {
        let character = query_characters.get_mut(event.entity);
        if let Ok(mut character) = character {
            character.damage(event.damage);
            if character.is_dead() {
                commands.entity(event.entity).despawn_recursive();
            }
        }
    }
}

#[derive(Default)]
pub struct PlayerInput {
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
