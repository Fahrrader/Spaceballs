use crate::characters::CHARACTER_SPEED;
use crate::controls::CharacterActionInput;
use crate::guns::stats::ProjectileSpawnSpace;
use crate::physics::{
    ColliderScale, KinematicsBundle, OngoingCollisions, RigidBody, Sensor, Velocity,
};
use crate::projectiles::ProjectileBundle;
use crate::teams::{team_color, Team, TeamNumber};
use crate::EntropyGenerator;
use bevy::ecs::system::EntityCommands;
use bevy::math::{Quat, Vec2, Vec3};
use bevy::prelude::{
    Bundle, Changed, Children, Commands, Component, Entity, GlobalTransform, Query,
    ReflectComponent, Res, Sprite, SpriteBundle, Time, Timer, TimerMode, Transform, With, Without,
};
use bevy::reflect::{FromReflect, Reflect, ReflectFromReflect};
use bevy::utils::default;
use std::f32::consts::PI;
use std::time::Duration;

pub mod additives;
mod presets;
mod stats;

use crate::network::PlayerHandle;
pub use presets::GunPreset;
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

/// Maximum number of unequipped guns in the world before they start disappearing.
const MAX_FREE_WEAPONS: usize = 5;

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
    pub fn new(preset: GunPreset, transform: Option<Transform>, rng: EntropyGenerator) -> Self {
        let mut gun_bundle = Self {
            gun: Gun::new(preset, rng),
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
        Gun::team_paint(
            self.gun.preset,
            &mut self.sprite_bundle.sprite,
            Some(team_number),
        );
        self
    }
}

/// Holder of all non-constant properties of a weapon.
#[derive(Component, Clone, Debug, Reflect, FromReflect)]
#[reflect_value(Debug, Component, FromReflect)]
pub struct Gun {
    pub preset: GunPreset,
    pub fire_cooldown: Timer,
    pub shots_before_reload: u32,
    pub reload_progress: Timer,
    // displace to a component? risk of nondeterministic order of execution
    entropy: EntropyGenerator,
}

impl Default for Gun {
    fn default() -> Self {
        let preset = GunPreset::default();
        let stats = preset.stats();
        Self {
            preset,
            fire_cooldown: Timer::new(stats.fire_cooldown, TimerMode::Once),
            shots_before_reload: stats.shots_before_reload,
            reload_progress: Timer::new(stats.reload_time, TimerMode::Once),
            entropy: EntropyGenerator::new(0),
        }
    }
}

impl Gun {
    pub fn new(preset: GunPreset, rng: EntropyGenerator) -> Self {
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
            entropy: rng,
        }
    }

    /// Reset everything about the gun's transform, replacing the component's parts with their default state.
    pub fn reset_to_default(
        entity_commands: &mut EntityCommands,
        preset: GunPreset,
        transform_to_reset: Option<&mut Transform>,
    ) {
        entity_commands
            .remove::<KinematicsBundle>()
            .remove::<Sensor>()
            .remove::<ColliderScale>();

        if let Some(transform) = transform_to_reset {
            let preset_transform = preset.stats().get_transform();
            transform.translation = preset_transform.translation;
            transform.rotation = preset_transform.rotation;
            transform.scale = preset_transform.scale;
        }
    }

    /// Make a gun look in line with a team's color or neutral (usually when not equipped by anybody).
    pub fn team_paint(preset: GunPreset, sprite: &mut Sprite, team_number: Option<TeamNumber>) {
        if let Some(team_number) = team_number {
            sprite.color = (team_color(team_number) * GUN_COLOR_MULTIPLIER).into();
        } else {
            sprite.color = preset.stats().gun_neutral_color.0;
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
                self.fire_cooldown.tick(self.reload_progress.duration());
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
                (self.entropy.gen::<f32>() - 0.5) * self.preset.stats().projectile_spread_angle,
            )
        }
    }

    /// Get a round (and, optionally, several more ahead, if that should have happened in the past)
    /// of projectiles that come out of a gun when a trigger is pressed. These still have to be spawned.
    /// The gun will change its state. If the gun has recoil, the character will be affected by it.
    fn fire_and_produce_projectiles(
        &mut self,
        gun_transform: &GlobalTransform,
        maybe_shooter_handle: Option<PlayerHandle>,
        team: &Team,
        character_transform: &mut Transform,
        fast_forward_rounds: Option<(u128, u128)>,
    ) -> (Vec<ProjectileBundle>, u128) {
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

                let mut bullet = ProjectileBundle::new(
                    self.preset,
                    maybe_shooter_handle,
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
#[derive(Component, Debug, Default, PartialEq, Reflect, FromReflect)]
pub struct Equipped {
    pub by: Option<Entity>,
}

/// Component telling when, at what time this entity has been last unequipped.
#[derive(Component, Debug, Default, PartialEq, Reflect, FromReflect)]
pub struct LastUnequippedAt(pub Duration);

pub mod systems {
    pub use super::additives::systems::*;
    use super::*;
    use crate::characters::PlayerControlled;

    /// System to spawn projectiles out of guns and keep track of their firing cooldowns, magazine sizes, and character recoil.
    pub fn handle_gunfire(
        mut commands: Commands,
        time: Res<Time>,
        mut query_weapons: Query<(&mut Gun, &GlobalTransform, &Equipped)>,
        mut query_characters: Query<(
            &CharacterActionInput,
            &Team,
            &mut Transform,
            Option<&PlayerControlled>,
        )>,
    ) {
        for (mut gun, gun_transform, equipped) in query_weapons.iter_mut() {
            if equipped.by.is_none() {
                continue;
            }

            let (wants_to_fire, wants_to_reload, team, mut transform, maybe_player_handle) =
                query_characters
                    .get_mut(equipped.by.expect(
                        "Should've checked if it was none! The gun is not equipped by anyone.",
                    ))
                    .map(|(input, team, transform, maybe_player)| {
                        (
                            input.fire,
                            input.reload,
                            team,
                            transform,
                            maybe_player.map(|player| player.handle),
                        )
                    })
                    .unwrap();

            if wants_to_reload {
                gun.start_reloading();
            }

            let cooldown_time_previously_elapsed = gun.fire_cooldown.elapsed().as_nanos();
            if gun.tick_cooldowns(time.delta()) && wants_to_fire {
                let cooldown_time_elapsed =
                    cooldown_time_previously_elapsed + time.delta().as_nanos();
                let cooldown_duration = gun.fire_cooldown.duration().as_nanos();
                let cooldown_times_over = cooldown_time_elapsed / cooldown_duration;
                let cooldown_latest_time_elapsed = cooldown_time_elapsed % cooldown_duration;

                let gun_type = gun.preset;

                // todo add a ray cast from the body to the gun barrel to check for collisions
                // but currently it's kinda like shooting from cover / over shoulder, fun

                let (bullets, _rounds_fired) = gun.fire_and_produce_projectiles(
                    gun_transform,
                    maybe_player_handle,
                    team,
                    &mut transform,
                    Some((cooldown_times_over, cooldown_latest_time_elapsed)),
                );

                if gun_type.has_extra_projectile_components() {
                    /* todo expand macro instead of function to return a bundle of components. would take the bullet bundle or other necessary information to form new components
                    let bullets = bullets.into_iter().map(|default_bundle| (default_bundle, gun_type.get_extra_projectile_components())).collect();
                    commands.spawn_batch(bullets);*/
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

    pub fn handle_gun_ownership_cosmetic_change(
        mut commands: Commands,
        mut q_guns: Query<(&Gun, &mut Sprite, &Equipped, Entity), Changed<Equipped>>,
        // maybe with character component if ever present
        q_characters: Query<&Team, With<Children>>,
    ) {
        for (gun, mut sprite, equipped, entity) in q_guns.iter_mut() {
            if equipped.by.is_none() {
                commands.entity(entity).remove::<Equipped>();
                Gun::team_paint(gun.preset, &mut sprite, None);
                continue;
            }

            let owner = equipped
                .by
                .expect("Should've checked if it was none! The gun is not equipped by anyone.");
            let team = q_characters
                .get(owner)
                .expect("Couldn't find the entity the gun is equipped by!");
            Gun::team_paint(gun.preset, &mut sprite, Some(team.0));
        }
    }

    /// System to make weapons more noticeable when not equipped and otherwise at rest.
    pub fn handle_gun_idle_bobbing(
        time: Res<Time>,
        // make sure that only the transform's scale changes, and doesn't affect the collider
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
                commands
                    .entity(entity)
                    .insert(Sensor)
                    .insert(ColliderScale::Absolute(Vec2::ONE));
                *body_type = RigidBody::Fixed;
            }
        }
    }

    /// System to clean up guns when there are too many free ones in the world.
    pub fn handle_gun_cleanup(
        mut commands: Commands,
        query_weapons: Query<(Entity, &LastUnequippedAt), (With<Gun>, Without<Equipped>)>,
    ) {
        if query_weapons.iter().len() <= MAX_FREE_WEAPONS {
            return;
        }

        let mut oldest = (Duration::from_secs(u64::MAX), Entity::PLACEHOLDER);

        for (entity, last_unequipped) in query_weapons.iter() {
            if last_unequipped.0 < oldest.0 {
                oldest = (last_unequipped.0, entity);
            }
        }

        commands.entity(oldest.1).despawn();
    }
}
