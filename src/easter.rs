//! What have I done, sweet Jesus, what have I done
//! probably delete this later, it's making me uncomfortable

use bevy::prelude::*;
use crate::GameState;

#[derive(Default, Debug, Resource)]
struct EasterGreeting(bool);

#[derive(Default, Debug, Component)]
pub struct EasterAnnouncerActivator;

fn activate_easter_greeting(
    activators: Query<&Interaction, (With<EasterAnnouncerActivator>, Changed<Interaction>)>,
    mut greeting: ResMut<EasterGreeting>,
) {
    activators.for_each(|interaction| if *interaction == Interaction::Clicked {
        greeting.0 = !greeting.0;
    });
}

fn play_easter_greeting(asset_server: Res<AssetServer>, audio: Res<Audio>, should_play: Res<EasterGreeting>) {
    if !should_play.0 {
        return;
    }
    let greeting = asset_server.load("sounds/greeting.ogg");
    audio.play_with_settings(greeting, PlaybackSettings::default().with_volume(0.75));
}

pub struct EasterAnnouncementPlugin;
impl Plugin for EasterAnnouncementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EasterGreeting>()
            .add_system(activate_easter_greeting)
            .add_system(play_easter_greeting.in_schedule(OnEnter(GameState::InGame)));
    }
}
