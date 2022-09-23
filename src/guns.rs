use crate::actions::CharacterActionInput;
use crate::characters::{Character, CHARACTER_SPEED};
use crate::guns::presets::REGULAR_GUN_FIRE_COOLDOWN_TIME_MILLIS;
use crate::physics::KinematicsBundle;
use crate::teams::{team_color, Team, TeamNumber};
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{
    Bundle, Commands, Component, Entity, GlobalTransform, Query, Res, Sprite, SpriteBundle, Time,
    Timer, Transform, With, Without,
};
use bevy::utils::default;
use rand::prelude::StdRng;
use rand::SeedableRng;
use std::f32::consts::PI;
use std::time::Duration;

pub mod colours;
mod presets;
mod stats;

pub use colours::GUN_TRANSPARENCY;
pub use presets::GunPreset;

/// The gun is slightly darker than the main color of the character body to be distinct.
const GUN_COLOR_MULTIPLIER: f32 = 0.75;

/// Gun minimum and maximum scale offset when bobbing when idle.
const GUN_BOBBING_AMPLITUDE: f32 = 0.2;
/// Gun full-cycle pulse (bobbing) time.
const GUN_BOBBING_TIME: f32 = 1.0;
/// Gun bobbing tempo, multiplier to the cosine.
const GUN_BOBBING_TEMPO: f64 = (2.0 * PI / GUN_BOBBING_TIME) as f64;
/// Gun's maximum velocity to start bobbing when it's below it.
const GUN_MAX_BOBBING_VELOCITY: f32 = CHARACTER_SPEED / 10.0;
/// Convenience. See [`GUN_MAX_BOBBING_VELOCITY`]
const GUN_MAX_BOBBING_VELOCITY_SQR: f32 = GUN_MAX_BOBBING_VELOCITY * GUN_MAX_BOBBING_VELOCITY;
/// The velocity-damping ratio of the gun, in effect when pushed or thrown.
const GUN_VELOCITY_DAMPING_RATIO: f32 = 1.15;

const GUN_Z_LAYER: f32 = 5.0;

// rename to weapon? nah dude this is spaceballs
#[derive(Bundle)]
pub struct GunBundle {
    preset: GunPreset,
    gun: Gun,
    #[bundle]
    kinematics: KinematicsBundle,
    #[bundle]
    sprite_bundle: SpriteBundle,
}

impl Default for GunBundle {
    fn default() -> Self {
        let preset = GunPreset::default();
        let stats = preset.stats();
        let transform = stats.get_transform();
        Self {
            gun: Gun::default(),
            kinematics: stats.get_kinematics(transform.scale),
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: stats.gun_neutral_color.0,
                    custom_size: Some(Vec2::new(stats.gun_width, stats.gun_length)),
                    ..default()
                },
                transform,
                ..default()
            },
            preset,
        }
    }
}

impl GunBundle {
    pub fn new(preset: GunPreset, transform: Option<Transform>) -> Self {
        let mut gun = Self::default();
        gun.preset = preset;
        gun.gun = Gun::new(&gun.preset, 0); // todo get new random u64 from the game's global random state
        gun.sprite_bundle.sprite.color = gun.preset.stats().gun_neutral_color.0;
        if let Some(transform) = transform {
            gun.sprite_bundle.transform = transform;
        } else {
            gun.sprite_bundle.transform = gun.preset.stats().get_transform();
        }
        gun
    }

    /// Change the bundled gun's color from neutral to that in line with the team.
    pub fn with_paint_job(mut self, team_number: TeamNumber) -> Self {
        paint_gun(
            &self.preset,
            &mut self.sprite_bundle.sprite,
            Some(team_number),
        );
        self
    }
}

/// Holder of all non-constant properties of a weapon.
#[derive(Component)]
pub struct Gun {
    fire_cooldown: Timer,
    random_state: StdRng,
}

impl Default for Gun {
    fn default() -> Self {
        Self {
            fire_cooldown: Timer::new(
                Duration::from_millis(REGULAR_GUN_FIRE_COOLDOWN_TIME_MILLIS),
                false,
            ),
            random_state: StdRng::seed_from_u64(0),
        }
    }
}

impl Gun {
    pub fn new(preset: &GunPreset, random_seed: u64) -> Self {
        let cooldown = preset.stats().fire_cooldown;
        let mut timer = Timer::new(cooldown, false);
        // Mark cooldown after firing as finished, the player shouldn't wait for the gun to recover when first picking it up
        timer.tick(cooldown);
        Self {
            fire_cooldown: timer,
            random_state: StdRng::seed_from_u64(random_seed),
        }
    }

    /// Check if the cooldown timer has finished, and the gun can be fired again.
    pub fn check_fire_cooldown(&self) -> bool {
        self.fire_cooldown.finished()
    }

    /// Tick some time off the cooldown timer and optionally check if finished.
    fn tick_fire_cooldown(&mut self, time_delta: Duration) -> bool {
        self.fire_cooldown.tick(time_delta).finished()
    }

    /// Set the cooldown timer to run anew. To be used usually when making a shot.
    fn reset_fire_cooldown(&mut self) {
        self.fire_cooldown.reset();
    }
}

/// Marker signifying that the entity is equipped "by" another entity and is a child (transforms are shared).
#[derive(Component)]
pub struct Equipped {
    pub by: Entity,
}

/// Marker signifying that the gun has been thrown away from the player, has some velocity, and shouldn't idle-bob yet.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Thrown;

/// Reset everything about the gun's transform, replacing the component's parts with their default state.
pub(crate) fn reset_gun_transform(preset: &GunPreset, transform: &mut Transform) {
    let preset_transform = preset.stats().get_transform();
    transform.translation = preset_transform.translation;
    transform.rotation = preset_transform.rotation;
    transform.scale = preset_transform.scale;
}

/// Make a gun look in line with a team's color or neutral (usually when not equipped by anybody).
pub(crate) fn paint_gun(preset: &GunPreset, sprite: &mut Sprite, team_number: Option<TeamNumber>) {
    if let Some(team_number) = team_number {
        // todo decide how to discern equipped weapons, just sprite/mesh shape or color, too
        sprite.color = (team_color(team_number) * GUN_COLOR_MULTIPLIER)
            .set_a(GUN_TRANSPARENCY)
            .as_rgba();
    } else {
        sprite.color = preset.stats().gun_neutral_color.0;
    }
}

/// System to spawn projectiles out of guns and keep track of their firing cooldowns, magazine sizes, and character recoil.
pub fn handle_gunfire(
    mut commands: Commands,
    time: Res<Time>,
    mut query_weapons: Query<(&mut Gun, &GunPreset, &GlobalTransform, &Equipped)>,
    query_characters: Query<(&CharacterActionInput, &Team), With<Character>>,
) {
    for (mut gun, gun_type, gun_transform, equipped) in query_weapons.iter_mut() {
        let (is_firing, team) = query_characters
            .get(equipped.by)
            .map(|(input, team)| (input.fire, team))
            .unwrap();

        // todo fixed time increment and potentially spawning multiple projectiles with go-ahead distance if cooldown is small enough
        // that is, fix shot skipping if cooldown is close to the frame time

        if gun.tick_fire_cooldown(time.delta()) && is_firing {
            let gun_stats = gun_type.stats();

            // todo add a ray cast from the body to the gun barrel to check for collisions
            // but currently it's kinda like shooting from cover / over shoulder, fun

            let bullets = gun_stats.produce_projectiles(gun_transform, gun_type, &mut gun, team);

            for bullet in bullets {
                let mut bullet_commands = commands.spawn_bundle(bullet);

                // 0.5 is applied as the default restitution when no PhysicMaterial is present
                if gun_stats.projectile_elasticity != 0.5 {
                    bullet_commands.insert(heron::PhysicMaterial {
                        restitution: gun_stats.projectile_elasticity,
                        ..default()
                    });
                }
            }

            gun.reset_fire_cooldown();
        }
    }
}

/// System to make weapons more noticeable when not equipped and otherwise at rest.
pub fn handle_gun_idle_bobbing(
    time: Res<Time>,
    mut query_weapons: Query<&mut Transform, (With<Gun>, Without<Thrown>, Without<Equipped>)>,
) {
    fn eval_bobbing(a: f32, cos_dt: f32) -> f32 {
        a + cos_dt
    }

    let time_cos_dt = -(GUN_BOBBING_TEMPO
        * (GUN_BOBBING_TEMPO * time.seconds_since_startup()).sin()) as f32
        * GUN_BOBBING_AMPLITUDE
        * time.delta_seconds();

    for mut transform in query_weapons.iter_mut() {
        transform.scale = Vec3::new(
            eval_bobbing(transform.scale.x, time_cos_dt),
            eval_bobbing(transform.scale.y, time_cos_dt),
            transform.scale.z,
        );
    }
}

/// System to strip the thrown guns of flying components if they have arrived within the threshold of rest.
pub fn handle_gun_arriving_at_rest(
    mut commands: Commands,
    mut query_weapons: Query<
        (&heron::Velocity, Entity),
        (With<Gun>, With<Thrown>, Without<Equipped>),
    >,
) {
    for (velocity, entity) in query_weapons.iter_mut() {
        if GUN_MAX_BOBBING_VELOCITY_SQR > velocity.linear.length_squared() {
            //transform.scale = Vec3::ONE;
            commands.entity(entity).remove::<Thrown>();
            // also possibly kinematics, or replace with sensors instead of rigidbodies. but first do projectiles to find a universal solution.
        }
    }
}
