use crate::characters::CHARACTER_SIZE;
use crate::teams::{Team, NONEXISTENT_TEAM};
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Bundle, Color, Component, Sprite, SpriteBundle, Transform};
use bevy::utils::default;

const GUN_TRANSPARENCY: f32 = 0.95;
const GUN_NEUTRAL_COLOR: Color = Color::Rgba {
    red: 0.25,
    green: 0.25,
    blue: 0.25,
    alpha: GUN_TRANSPARENCY,
}; // Color::DARK_GRAY

const GUN_LENGTH: f32 = CHARACTER_SIZE * 1.25;
const GUN_WIDTH: f32 = CHARACTER_SIZE * 0.25;

const GUN_CENTER_X: f32 = 0.0;
const GUN_CENTER_Y: f32 = CHARACTER_SIZE * -0.15 + GUN_LENGTH * 0.5;
const GUN_Z_LAYER: f32 = 1.0;

// rename to weapon? nah dude this is spaceballs
#[derive(Bundle)]
pub struct GunBundle {
    preset: GunPreset,
    team: Team,
    // scenarios for modifying a single weapon?
    // - power-ups: apply damage boost, but why, use another function that'd go through the vec to skew
    // -
    // also bullets would have the enum (gun type? projectile type? when is it not 1-to-1?) to determine their behaviour, too
    // the enum is much easier to transfer online
    // much easier and less expensive to switch guns
    #[bundle]
    sprite_bundle: SpriteBundle,
}

impl Default for GunBundle {
    fn default() -> Self {
        let preset = GunPreset::default();
        Self {
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: GUN_NEUTRAL_COLOR,
                    custom_size: Some(Vec2::new(GUN_WIDTH, GUN_LENGTH)),
                    ..default()
                },
                transform: preset.get_transform(),
                ..default()
            },
            team: Team(NONEXISTENT_TEAM),
            preset,
        }
    }
}

impl GunBundle {
    pub fn new(preset: GunPreset) -> Self {
        let mut gun = Self::default();
        gun.preset = preset;
        // todo set new transform/sprite according to preset?
        gun
    }

    // todo refactor to work with queries
    pub(crate) fn make_ones_own(&mut self, team: &Team) {
        self.team = team.clone();
        self.sprite_bundle.sprite.color = *self.team.color().set_a(GUN_TRANSPARENCY);
    }
}

#[derive(Component, Clone)]
pub enum GunPreset {
    Regular,
    Imprecise,
    RailGun,
    Scattershot,
    Typhoon,
    // EMPCannon, SmokeCannon, LaserGun, RocketLauncher, Termite, PortalGun, MechSword?,
    // AssemblyNanoSwarmLauncher, MinePlanter, TeslaCoilLauncher
}

impl Default for GunPreset {
    fn default() -> Self {
        GunPreset::Regular
    }
}

impl GunPreset {
    fn get_transform(&self) -> Transform {
        // match self {
        Transform::from_translation(Vec3::new(GUN_CENTER_X, GUN_CENTER_Y, GUN_Z_LAYER))
        // right- and left-handedness?
    }
}
// todo projectiles_per_shot, fire cooldown, spread, damage, recoil

// gun behavior for different aspects, have gun presets -- just functions to create a new gun bundle? does it have to be a bundle, components are not attached to anything else, though
// but gun bundle possesses extra vars that don't change, a waste of memory; just have an enum that'd match to a specific preset? looks like it's going to be a lookup tree, so no biggie

/*let good_distance = match character.firing_mode {
FiringMode::Regular | FiringMode::RailGun => Quat::IDENTITY,
FiringMode::Imprecise => {
Quat::from_axis_angle(-Vec3::Z, (rand::random::<f32>() - 0.5) * PI / 12.0)
}
FiringMode::Scattershot => {
Quat::from_axis_angle(-Vec3::Z, (rand::random::<f32>() - 0.5) * PI / 2.0)
}
FiringMode::Typhoon => {
Quat::from_axis_angle(-Vec3::Z, (rand::random::<f32>() - 0.5) * PI * 2.0)
}
} * character_transform.up();*/
