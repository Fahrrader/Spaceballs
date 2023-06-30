use crate::network::peers::{PeerHandles, PeerNames};
use crate::GameState;
use bevy::log::prelude::*;
use bevy::prelude::{
    in_state, not, App, DetectChanges, IntoSystemConfig, Plugin, Res, ResMut, Resource,
};

#[derive(Debug, Default)]
pub struct PlayerData {
    pub name: String,
    // team?
    pub kills: usize,
    pub deaths: usize,
}

#[derive(Resource, Debug, Default)]
pub struct PlayerRegistry(pub Vec<PlayerData>);

impl PlayerRegistry {
    #[allow(unused)]
    pub fn get(&self, handle: usize) -> Option<&PlayerData> {
        self.0.get(handle).or_else(|| {
            warn!("Could not find player by handle {}!", handle);
            None
        })
    }

    #[allow(unused)]
    pub fn get_mut(&mut self, handle: usize) -> Option<&mut PlayerData> {
        self.0.get_mut(handle).or_else(|| {
            warn!("Could not find player by handle {}!", handle);
            None
        })
    }
}

pub fn update_player_names(
    peer_names: Res<PeerNames>,
    peer_handles: Res<PeerHandles>,
    mut players: ResMut<PlayerRegistry>,
) {
    if !peer_names.is_changed() {
        return;
    }

    for (id, name) in &peer_names.map {
        if let Some(handle) = peer_handles.map.get(id) {
            if let Some(data) = players.get_mut(*handle) {
                data.name = name.clone();
            }
        }
    }
}

pub(crate) struct OnlinePlayerPlugin;
impl Plugin for OnlinePlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerRegistry::default())
            .add_system(update_player_names.run_if(not(in_state(GameState::MainMenu))));
    }
}
