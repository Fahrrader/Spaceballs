//! Peers are simply clients when they are not handled as players.
//! While players are assigned GGRS' `PlayerHandle`, peers are assigned `PeerId`.

use crate::network::socket::SpaceballSocket;
use crate::network::{PeerId, PlayerHandle};
use crate::ui::chat::ChatMessage;
use crate::ui::user_settings::{UserInputForm, UserSettings};
use crate::GameState;
use bevy::prelude::{
    in_state, not, App, EventReader, EventWriter, IntoSystemConfig, Plugin, Res, ResMut, Resource,
};
use bevy::utils::HashMap;
use bevy_matchbox::prelude::PeerState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct PeerConnectionEvent {
    pub id: PeerId,
    pub state: PeerState,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PeerMessage {
    PlayerName { name: String },
    Chat { message: String },
}

#[derive(Resource, Debug, Default)]
pub struct PeerHandles {
    pub map: HashMap<PeerId, PlayerHandle>,
}

#[derive(Resource, Debug, Default)]
pub struct PeerNames {
    pub map: HashMap<PeerId, String>,
}

pub fn handle_player_name_broadcast(
    mut socket: ResMut<SpaceballSocket>,
    settings: Res<UserSettings>,
    mut peer_events: EventReader<PeerConnectionEvent>,
) {
    if peer_events.iter().any(|event| {
        matches!(
            event,
            PeerConnectionEvent {
                state: PeerState::Connected,
                ..
            }
        )
    }) {
        // broadcasting our local user-set name
        if let Some(name) = settings.get_string(UserInputForm::PlayerName) {
            socket.broadcast_tcp_message(PeerMessage::PlayerName { name });
        }
    }
}

pub fn handle_receiving_peer_messages(
    mut socket: ResMut<SpaceballSocket>,
    mut peer_names: ResMut<PeerNames>,
    peer_handles: Res<PeerHandles>,
    mut messenger: EventWriter<ChatMessage>,
) {
    let messages = socket.receive_tcp_messages();
    for (sender, message) in messages {
        match message {
            PeerMessage::PlayerName { name } => {
                if !peer_names.map.contains_key(&sender) {
                    messenger.send(ChatMessage {
                        player_handle: None,
                        message: format!("{} joined!", name),
                    });
                }
                peer_names.map.insert(sender, name);
            }
            PeerMessage::Chat { message } => {
                // ignore the message if it came from an unregistered source
                if let Some(handle) = peer_handles.map.get(&sender) {
                    messenger.send(ChatMessage {
                        player_handle: Some(*handle),
                        message,
                    });
                }
            }
        }
    }
}

pub fn handle_reporting_peer_disconnecting(
    mut peer_names: ResMut<PeerNames>,
    mut peer_events: EventReader<PeerConnectionEvent>,
    mut messenger: EventWriter<ChatMessage>,
) {
    for event in peer_events.iter() {
        match event {
            PeerConnectionEvent {
                state: PeerState::Disconnected,
                id,
            } => {
                if let Some(name) = peer_names.map.remove(id) {
                    messenger.send(ChatMessage {
                        player_handle: None,
                        message: format!("{} left!", name),
                    });
                }
            }
            _ => {}
        }
    }
}

pub(crate) struct OnlinePeerPlugin;
impl Plugin for OnlinePeerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PeerConnectionEvent>()
            .insert_resource(PeerNames::default())
            .insert_resource(PeerHandles::default())
            // ideally, there should be `or` between `Matchmaking` and `InGame`, but no, ok
            .add_system(handle_player_name_broadcast.run_if(not(in_state(GameState::MainMenu))))
            .add_system(handle_receiving_peer_messages.run_if(not(in_state(GameState::MainMenu))))
            .add_system(
                handle_reporting_peer_disconnecting.run_if(not(in_state(GameState::MainMenu))),
            );
    }
}
