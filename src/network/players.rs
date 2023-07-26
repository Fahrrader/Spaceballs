use crate::network::peers::{PeerHandles, PeerNames};
use crate::network::PlayerHandle;
use crate::GameState;
use bevy::prelude::*;

#[derive(Debug, Default)]
pub struct PlayerData {
    pub name: String,
    // team?
    pub kills: usize,
    pub deaths: usize,
}

#[derive(Resource, Debug, Default)]
pub struct PlayerRegistry(pub Vec<PlayerData>);

#[derive(Debug, Default)]
pub struct PlayerJoined {
    pub player_handle: PlayerHandle,
}

#[derive(Debug, Default)]
pub struct PlayerDied {
    pub player_handle: PlayerHandle,
    // killed_by, bla-bla-bla
}

impl PlayerRegistry {
    #[allow(unused)]
    pub fn get(&self, handle: PlayerHandle) -> Option<&PlayerData> {
        self.0.get(handle).or_else(|| {
            warn!("Could not find player by handle {}!", handle);
            None
        })
    }

    #[allow(unused)]
    pub fn get_mut(&mut self, handle: PlayerHandle) -> Option<&mut PlayerData> {
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

pub fn send_all_players_joined(
    players: Res<PlayerRegistry>,
    mut player_teller: EventWriter<PlayerJoined>,
) {
    // maybe should compare to all existing players, see if their PlayerControlled characters exist
    // for drop-in
    for (index, _) in players.0.iter().enumerate() {
        player_teller.send(PlayerJoined {
            player_handle: index,
        });
    }
}

/*pub fn send_player_joined_event_if_in_game(
    // mut peer_reader: EventReader<PeerConnectionEvent>,
    players: Res<PlayerRegistry>,
    // mut player_teller: EventWriter<PlayerJoined>,
) {
    // only send the new event if the player handle (not available from peer_connection_event) is not present in the player registry
}*/

pub(crate) struct OnlinePlayerPlugin;
impl Plugin for OnlinePlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerRegistry::default())
            .add_event::<PlayerJoined>()
            .add_event::<PlayerDied>()
            .add_system(update_player_names.run_if(in_state(GameState::InGame)))
            .add_system(send_all_players_joined.in_schedule(OnEnter(GameState::InGame))/* GGRSSchedule? rework if implementing drop-in */)
        ;
    }
}
