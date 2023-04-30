use crate::characters::CHARACTER_SPEED;
use crate::controls::CharacterActionInput;
use crate::guns::stats::ProjectileSpawnSpace;
use crate::physics::{KinematicsBundle, OngoingCollisions, RigidBody, Sensor, Velocity};
use crate::projectiles::BulletBundle;
use crate::teams::{team_color, Team, TeamNumber};
use bevy::math::{Quat, Vec2, Vec3};
use bevy::prelude::{
    Bundle, Commands, Component, Entity, GlobalTransform, Query, Res, Sprite, SpriteBundle, Time,
    Timer, TimerMode, Transform, With, Without,
};
use bevy::utils::default;
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use std::f32::consts::PI;
use std::time::Duration;

pub mod colours;
mod presets;
mod stats;

pub use colours::GUN_TRANSPARENCY;
pub use presets::{GunPreset, RAIL_GUN_DAMAGE_PER_SECOND};
pub use stats::GunPersistentStats;

/// The gun is slightly darker than the main color of the character body to be distinct.
const GUN_COLOR_MULTIPLIER: f32 = 0.75;

/// Gun minimum and maximum scale offset when bobbing when idle.
const GUN_BOBBING_AMPLITUDE: f32 = 0.2;
/// Gun full-cycle pulse (bobbing) time.
const GUN_BOBBING_TIME: f32 = 1.0;
/// Gun bobbing tempo, multiplier to the cosine.
const GUN_BOBBING_TEMPO: f32 = 2.0 * PI / GUN_BOBBING_TIME;
/// Gun's maximum velocity to start bobbing when it's below it.
const GUN_MAX_BOBBING_VELOCITY: f32 = CHARACTER_SPEED / 8.0;
/// Convenience. See [`GUN_MAX_BOBBING_VELOCITY`]
const GUN_MAX_BOBBING_VELOCITY_SQR: f32 = GUN_MAX_BOBBING_VELOCITY * GUN_MAX_BOBBING_VELOCITY;
/// The velocity-damping ratio of the gun, in effect when pushed or thrown.
const GUN_VELOCITY_DAMPING_RATIO: f32 = 1.15;
/// Make it visible!
const GUN_Z_LAYER: f32 = 5.0;

/// Collection of components making up a gun entity. Starts independent and has to be equipped.
/// Note that the kinematics bundle will be stripped when equipped.
#[derive(Bundle)]
pub struct GunBundle {
    pub gun: Gun,
    #[bundle]
    pub kinematics: KinematicsBundle,
    pub sensor: Sensor,
    pub collisions: OngoingCollisions,
    #[bundle]
    pub sprite_bundle: SpriteBundle,
}

impl Default for GunBundle {
    fn default() -> Self {
        let gun = Gun::default();
        let stats = gun.preset.stats();
        let transform = stats.get_transform();
        Self {
            gun,
            kinematics: stats.get_kinematics(),
            sensor: Sensor,
            collisions: OngoingCollisions::default(),
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: stats.gun_neutral_color.0,
                    custom_size: Some(Vec2::new(stats.gun_width, stats.gun_length)),
                    ..default()
                },
                transform,
                ..default()
            },
        }
    }
}

impl GunBundle {
    pub fn new(preset: GunPreset, transform: Option<Transform>, random_seed: u64) -> Self {
        let mut gun_bundle = Self {
            gun: Gun::new(preset, random_seed),
            ..default()
        };
        gun_bundle.sprite_bundle.sprite.color = preset.stats().gun_neutral_color.0;
        if let Some(transform) = transform {
            gun_bundle.sprite_bundle.transform = transform;
        } else {
            gun_bundle.sprite_bundle.transform = preset.stats().get_transform();
        }
        gun_bundle
    }

    /// Change the bundled gun's color from neutral to that in line with the team.
    pub fn with_paint_job(mut self, team_number: TeamNumber) -> Self {
        team_paint_gun(
            self.gun.preset,
            &mut self.sprite_bundle.sprite,
            Some(team_number),
        );
        self
    }
}

/// Holder of all non-constant properties of a weapon.
//#[derive(Component, Debug, PartialEq, Clone, Reflect, FromReflect)]
#[derive(Component, Debug, Clone)]
pub struct Gun {
    pub(crate) preset: GunPreset,
    fire_cooldown: Timer,
    shots_before_reload: u32,
    reload_progress: Timer,
    random_state: StdRng,
}

impl Default for Gun {
    fn default() -> Self {
        let preset = GunPreset::Regular;
        let stats = preset.stats();
        Self {
            preset,
            fire_cooldown: Timer::new(stats.fire_cooldown, TimerMode::Once),
            shots_before_reload: stats.shots_before_reload,
            reload_progress: Timer::new(stats.reload_time, TimerMode::Once),
            random_state: StdRng::seed_from_u64(0),
        }
    }
}

impl Gun {
    pub fn new(preset: GunPreset, random_seed: u64) -> Self {
        let stats = preset.stats();

        let mut fire_cooldown = Timer::new(stats.fire_cooldown, TimerMode::Once);
        // Mark cooldown after firing as finished, the player shouldn't wait for the gun to recover when first picking it up
        fire_cooldown.tick(stats.fire_cooldown);

        let mut reload_progress = Timer::new(stats.reload_time, TimerMode::Once);
        // Reloading is not active at the start, firing cooldown and reloading must be mutually exclusive
        reload_progress.pause();

        Self {
            preset,
            fire_cooldown,
            shots_before_reload: stats.shots_before_reload,
            reload_progress,
            random_state: StdRng::seed_from_u64(random_seed),
        }
    }

    /// Check if the cooldown timers have finished, and the gun can be fired.
    pub fn can_fire(&self) -> bool {
        self.fire_cooldown.finished() && self.reload_progress.paused()
    }

    /// Tick some time on the cooldown timers and return if the gun is able to fire.
    fn tick_cooldowns(&mut self, time_delta: Duration) -> bool {
        let was_reloading = !self.reload_progress.paused();
        if was_reloading {
            // todo only if equipped and on display (in case of gun switching)
            if self.reload_progress.tick(time_delta).finished() {
                self.shots_before_reload = self.preset.stats().shots_before_reload;
                self.reload_progress.reset();
                self.reload_progress.pause();
                self.fire_cooldown.unpause();
            }
            false
        } else {
            self.fire_cooldown.tick(time_delta).finished()
            // Reloading is triggered separately
        }
    }

    /// Expend a round from the magazine and return true if there are no more rounds left, and we should reload the gun.
    fn eject_shot_and_check_if_empty(&mut self) -> bool {
        if self.preset.stats().shots_before_reload > 0 {
            self.shots_before_reload -= 1;
            self.shots_before_reload == 0
        } else {
            false
        }
    }

    /// Set the cooldown timer to run anew. To be used usually when making a shot.
    fn reset_fire_cooldown(&mut self) {
        self.fire_cooldown.reset();
    }

    /// Change the behaviour of the gun to reloading. Only when the reloading has finished, can the gun resume firing.
    fn start_reloading(&mut self) {
        self.fire_cooldown.pause();
        self.reload_progress.unpause();
    }

    /// Calculate a possibly random vector of flight direction of a projectile. The gun will change its state.
    pub fn choose_spread_direction(&mut self) -> Quat {
        if self.preset.stats().projectile_spread_angle == 0.0 {
            Quat::IDENTITY
        } else {
            Quat::from_axis_angle(
                -Vec3::Z,
                (self.random_state.gen::<f32>() - 0.5)
                    * self.preset.stats().projectile_spread_angle,
            )
        }
    }

    /// Get a round (and, optionally, several more ahead, if that should have happened in the past)
    /// of projectiles that come out of a gun when a trigger is pressed. These still have to be spawned.
    /// The gun will change its state. If the gun has recoil, the character will be affected by it.
    fn fire_and_produce_projectiles(
        &mut self,
        gun_transform: &GlobalTransform,
        team: &Team,
        character_transform: &mut Transform,
        fast_forward_rounds: Option<(u128, u128)>,
    ) -> (Vec<BulletBundle>, u128) {
        let gun_stats = self.preset.stats();
        let (gun_scale, _, gun_translation) = gun_transform.to_scale_rotation_translation();
        let bullet_spawn_distance = gun_stats.get_bullet_spawn_offset(gun_scale);
        let cooldown_duration = self.fire_cooldown.duration().as_nanos();

        // "Perimeter" does not have a set spawn point, so it will have to have another pass later.
        let bullet_spawn_point = match gun_stats.projectile_spawn_point {
            // Set at the gun barrel exit.
            ProjectileSpawnSpace::Gunpoint => {
                gun_translation + bullet_spawn_distance * gun_transform.up()
            }
            // For now, set at the character center - change the offset individually.
            ProjectileSpawnSpace::Perimeter => {
                gun_translation - gun_stats.gun_center_y * gun_transform.up()
            }
        };

        let bullet_transform = gun_transform
            .compute_transform()
            .with_translation(bullet_spawn_point)
            .with_scale(Vec3::ONE);

        let mut bullets = vec![];

        let mut rounds_fired = 0;
        let (rounds_to_fire, time_in_nanos_elapsed_since_latest_cooldown) =
            if let Some(fast_forward) = fast_forward_rounds {
                fast_forward
            } else {
                (1, 0)
            };
        while rounds_fired < rounds_to_fire {
            for _ in 0..gun_stats.projectiles_per_shot {
                let facing_direction = self.choose_spread_direction() * gun_transform.up();

                // Adjust spawn points for "Perimeter" individually around the perimeter according to the established random direction.
                let bullet_transform = match gun_stats.projectile_spawn_point {
                    ProjectileSpawnSpace::Gunpoint => bullet_transform,
                    ProjectileSpawnSpace::Perimeter => bullet_transform.with_translation(
                        bullet_transform.translation + bullet_spawn_distance * facing_direction,
                    ),
                };

                let mut bullet = BulletBundle::new(
                    self.preset,
                    team.0,
                    bullet_transform,
                    facing_direction * gun_stats.projectile_speed,
                );

                let linear_velocity = bullet.kinematics.velocity.linvel.extend(0.);
                bullet.sprite_bundle.transform.translation += (rounds_fired * cooldown_duration + time_in_nanos_elapsed_since_latest_cooldown) as f32
                    / cooldown_duration as f32
                    * linear_velocity
                    // nanos per second
                    / 1_000_000_000.0;

                bullets.push(bullet);
            }

            // not clean. but the guy has to be moved in between shots and frame-skips, and it's better than to repeat calculations.
            // oh -- gun's global transform will probably not manage to change in time anyway?
            if gun_stats.recoil != 0.0 {
                let offset = character_transform.down() * gun_stats.recoil;
                character_transform.translation += offset;
            }

            rounds_fired += 1;

            if self.eject_shot_and_check_if_empty() {
                self.start_reloading();
                break;
            }
        }

        self.reset_fire_cooldown();
        self.tick_cooldowns(Duration::from_nanos(
            ((rounds_to_fire - rounds_fired) * cooldown_duration
                + time_in_nanos_elapsed_since_latest_cooldown) as u64,
        ));

        (bullets, rounds_fired)
    }
}

/// Marker signifying that the entity is equipped "by" another entity and is a child (transforms are shared).
#[derive(Component)]
pub struct Equipped {
    pub by: Entity,
}

/// Reset everything about the gun's transform, replacing the component's parts with their default state.
pub(crate) fn reset_gun_transform(preset: GunPreset, transform: &mut Transform) {
    let preset_transform = preset.stats().get_transform();
    transform.translation = preset_transform.translation;
    transform.rotation = preset_transform.rotation;
    transform.scale = preset_transform.scale;
}

/// Make a gun look in line with a team's color or neutral (usually when not equipped by anybody).
pub(crate) fn team_paint_gun(
    preset: GunPreset,
    sprite: &mut Sprite,
    team_number: Option<TeamNumber>,
) {
    if let Some(team_number) = team_number {
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
    mut query_weapons: Query<(&mut Gun, &GlobalTransform, &Equipped)>,
    // todo:mp event for movement instead
    mut query_characters: Query<(&CharacterActionInput, &Team, &mut Transform)>,
) {
    for (mut gun, gun_transform, equipped) in query_weapons.iter_mut() {
        let (wants_to_fire, wants_to_reload, team, mut transform) = query_characters
            .get_mut(equipped.by)
            .map(|(input, team, transform)| (input.fire, input.reload, team, transform))
            .unwrap();

        if wants_to_reload {
            gun.start_reloading();
        }

        let cooldown_time_previously_elapsed = gun.fire_cooldown.elapsed().as_nanos();
        if gun.tick_cooldowns(time.delta()) && wants_to_fire {
            let cooldown_time_elapsed = cooldown_time_previously_elapsed + time.delta().as_nanos();
            let cooldown_duration = gun.fire_cooldown.duration().as_nanos();
            let cooldown_times_over = cooldown_time_elapsed / cooldown_duration;
            let cooldown_latest_time_elapsed = cooldown_time_elapsed % cooldown_duration;

            let gun_type = gun.preset;

            // todo add a ray cast from the body to the gun barrel to check for collisions
            // but currently it's kinda like shooting from cover / over shoulder, fun

            let (bullets, _rounds_fired) = gun.fire_and_produce_projectiles(
                gun_transform,
                team,
                &mut transform,
                Some((cooldown_times_over, cooldown_latest_time_elapsed)),
            );

            if gun_type.has_extra_projectile_components() {
                for bullet in bullets {
                    let mut bullet_commands = commands.spawn(bullet);
                    // Add any extra components that a bullet should have
                    gun_type.add_projectile_components(&mut bullet_commands);
                }
            } else {
                commands.spawn_batch(bullets);
            }
        }
    }
}

/// System to make weapons more noticeable when not equipped and otherwise at rest.
pub fn handle_gun_idle_bobbing(
    time: Res<Time>,
    mut query_weapons: Query<&mut Transform, (With<Gun>, With<Sensor>, Without<Equipped>)>,
) {
    fn eval_bobbing(a: f32, cos_dt: f32) -> f32 {
        // a crutch for the time being. if frame time is too low (as when the window is not focused on),
        // cos_dt gets so big that it breaks the function.
        (a + cos_dt).clamp(1. - GUN_BOBBING_AMPLITUDE, 1. + GUN_BOBBING_AMPLITUDE)
    }

    if query_weapons.is_empty() {
        return;
    }

    let time_cos_dt = -GUN_BOBBING_TEMPO
        * GUN_BOBBING_AMPLITUDE
        * (GUN_BOBBING_TEMPO as f64 * time.elapsed_seconds_f64()).sin() as f32
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
        (&Velocity, &mut RigidBody, Entity),
        (With<Gun>, Without<Equipped>, Without<Sensor>),
    >,
) {
    for (velocity, mut body_type, entity) in query_weapons.iter_mut() {
        if GUN_MAX_BOBBING_VELOCITY_SQR > velocity.linvel.length_squared() {
            commands.entity(entity).insert(Sensor);
            *body_type = RigidBody::Fixed;
        }
    }
}
