use crate::actions::CharacterActionInput;
use crate::characters::{Character, CHARACTER_SPEED};
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
const GUN_MAX_BOBBING_VELOCITY: f32 = CHARACTER_SPEED / 8.0;
/// Convenience. See [`GUN_MAX_BOBBING_VELOCITY`]
const GUN_MAX_BOBBING_VELOCITY_SQR: f32 = GUN_MAX_BOBBING_VELOCITY * GUN_MAX_BOBBING_VELOCITY;
/// The velocity-damping ratio of the gun, in effect when pushed or thrown.
const GUN_VELOCITY_DAMPING_RATIO: f32 = 1.15;

const GUN_Z_LAYER: f32 = 5.0;

// rename to weapon? nah dude this is spaceballs
#[derive(Bundle)]
pub struct GunBundle {
    pub gun: Gun,
    #[bundle]
    pub kinematics: KinematicsBundle,
    pub collisions: heron::Collisions,
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
            kinematics: stats.get_kinematics(transform.scale),
            collisions: heron::Collisions::default(),
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
        let mut gun_bundle = Self::default();
        gun_bundle.gun = Gun::new(preset, random_seed);
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
        paint_gun(
            self.gun.preset,
            &mut self.sprite_bundle.sprite,
            Some(team_number),
        );
        self
    }
}

/// Holder of all non-constant properties of a weapon.
#[derive(Component)]
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
            fire_cooldown: Timer::new(stats.fire_cooldown, false),
            shots_before_reload: stats.shots_before_reload,
            reload_progress: Timer::new(stats.reload_time, false),
            random_state: StdRng::seed_from_u64(0),
        }
    }
}

impl Gun {
    pub fn new(preset: GunPreset, random_seed: u64) -> Self {
        let stats = preset.stats();

        let mut fire_cooldown = Timer::new(stats.fire_cooldown, false);
        // Mark cooldown after firing as finished, the player shouldn't wait for the gun to recover when first picking it up
        fire_cooldown.tick(stats.fire_cooldown);

        let mut reload_progress = Timer::new(stats.reload_time, false);
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
    fn tick_cooldown(&mut self, time_delta: Duration) -> bool {
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

    fn eject_shot_and_check_if_empty(&mut self) -> bool {
        if self.preset.stats().shots_before_reload > 0 {
            self.shots_before_reload -= 1;
            self.shots_before_reload <= 0
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
}

/// Marker signifying that the entity is equipped "by" another entity and is a child (transforms are shared).
#[derive(Component)]
pub struct Equipped {
    pub by: Entity,
}

/// Marker signifying that the gun has been thrown away from the player, has some velocity, and shouldn't idle-bob yet.
#[derive(Component)]
pub struct Thrown;

/// Reset everything about the gun's transform, replacing the component's parts with their default state.
pub(crate) fn reset_gun_transform(preset: GunPreset, transform: &mut Transform) {
    let preset_transform = preset.stats().get_transform();
    transform.translation = preset_transform.translation;
    transform.rotation = preset_transform.rotation;
    transform.scale = preset_transform.scale;
}

/// Make a gun look in line with a team's color or neutral (usually when not equipped by anybody).
pub(crate) fn paint_gun(preset: GunPreset, sprite: &mut Sprite, team_number: Option<TeamNumber>) {
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
    mut query_characters: Query<(&CharacterActionInput, &Team, &mut Transform), With<Character>>,
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
        if gun.tick_cooldown(time.delta()) && wants_to_fire {
            let cooldown_time_elapsed = cooldown_time_previously_elapsed + time.delta().as_nanos();
            let cooldown_duration = gun.fire_cooldown.duration().as_nanos();
            let cooldown_times_over = cooldown_time_elapsed / cooldown_duration;

            let gun_type = gun.preset;
            let gun_stats = gun_type.stats();

            // todo add a ray cast from the body to the gun barrel to check for collisions
            // but currently it's kinda like shooting from cover / over shoulder, fun

            // any spawn point displacement causes artifacts in reflection angle (thanks, heron) -- look into elasticity
            for cd in 0..cooldown_times_over {
                let bullets =
                    gun_stats.produce_projectiles(gun_transform, gun_type, &mut gun, team);

                for mut bullet in bullets {
                    let linear_velocity = bullet.kinematics.velocity.linear;
                    bullet.sprite_bundle.transform.translation += ((cd + 1) * cooldown_duration - cooldown_time_previously_elapsed) as f32
                            * linear_velocity
                            // nanos per second
                            / 1_000_000_000.0;

                    let mut bullet_commands = commands.spawn_bundle(bullet);

                    // 0.5 is applied as the default restitution when no PhysicMaterial is present
                    if gun_stats.projectile_elasticity != 0.0 {
                        bullet_commands.insert(heron::PhysicMaterial {
                            restitution: gun_stats.projectile_elasticity,
                            ..default()
                        });
                    }

                    // Add any extra components that a bullet should have
                    for component in gun_type.extra_components() {
                        bullet_commands.insert(component);
                    }
                    /*for combundle in gun_type.extra_components_and_bundles() {
                        if let ExtraCombundle::Component(component) = combundle {
                            bullet_commands.insert(*component);
                        } else if let ExtraCombundle::Bundle(bundle) = combundle {
                            //bullet_commands.insert_bundle(*bundle);
                        }
                    }*/
                }

                if gun_stats.recoil != 0.0 {
                    let offset = transform.down() * gun_stats.recoil;
                    transform.translation += offset;
                }

                if gun.eject_shot_and_check_if_empty() {
                    gun.start_reloading();
                    break;
                }
            }

            gun.reset_fire_cooldown();
            gun.tick_cooldown(Duration::from_nanos(
                (cooldown_time_elapsed % cooldown_duration) as u64,
            ));
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
        (&heron::Velocity, &mut heron::RigidBody, Entity),
        (With<Gun>, With<Thrown>, Without<Equipped>),
    >,
) {
    for (velocity, mut body_type, entity) in query_weapons.iter_mut() {
        if GUN_MAX_BOBBING_VELOCITY_SQR > velocity.linear.length_squared() {
            commands.entity(entity).remove::<Thrown>();
            *body_type = heron::RigidBody::Sensor;
        }
    }
}
