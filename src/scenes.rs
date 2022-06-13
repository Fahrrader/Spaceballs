use crate::{
    BaseCharacterBundle, ControlledPlayerCharacterBundle, AI_DEFAULT_TEAM, PLAYER_DEFAULT_TEAM,
};
use bevy::math::{Quat, Vec3};
use bevy::prelude::{Commands, OrthographicCameraBundle, Res, Transform};

#[derive(clap::ValueEnum, Clone)]
pub enum SceneArg {
    Experimental,
    Lite,
}

impl TryFrom<String> for SceneArg {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "experimental" | "exp" | "e" => Ok(SceneArg::Experimental),
            "lite" | "l" => Ok(SceneArg::Lite),
            _ => Err("Nothing too bad, should use the default scene"),
        }
    }
}

pub fn summon_scene(commands: Commands, scene: Res<Option<SceneArg>>) {
    match scene.into_inner() {
        None => setup_lite(commands),
        Some(scene) => match scene {
            SceneArg::Experimental => setup_experimental(commands),
            SceneArg::Lite => setup_lite(commands),
        },
    }
}

pub fn setup_experimental(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    commands.spawn_bundle(ControlledPlayerCharacterBundle::new(
        PLAYER_DEFAULT_TEAM,
        Transform::default(),
    ));

    commands.spawn_bundle(BaseCharacterBundle::new(
        AI_DEFAULT_TEAM,
        Transform::from_scale(Vec3::new(2.0, 3.0, 1.0))
            .with_rotation(Quat::from_axis_angle(-Vec3::Z, 30.0)),
    ));
}

pub fn setup_lite(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    commands.spawn_bundle(ControlledPlayerCharacterBundle::new(
        PLAYER_DEFAULT_TEAM,
        Transform::default(),
    ));
}
