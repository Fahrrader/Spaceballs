//! Peers are simply clients when they are not handled as players.
//! While players are assigned GGRS' `PlayerHandle`, peers are assigned `PeerId`.

use crate::network::socket::SpaceballSocket;
use crate::network::{PeerId, PlayerHandle};
use crate::ui::chat::ChatMessage;
use crate::ui::user_settings::UserSettings;
use crate::GameState;
use bevy::prelude::*;
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
    // todo ChatHistory { ... },
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
        // broadcasting our local user-set name to other (including newly joined) peers
        socket.broadcast_tcp_message(PeerMessage::PlayerName {
            name: settings.player_name.clone(),
        });
    }
}

pub fn handle_chat_message_broadcast(
    mut socket: ResMut<SpaceballSocket>,
    mut messenger: EventReader<PeerMessage>,
) {
    for message in messenger.iter() {
        socket.broadcast_tcp_message(message.clone());
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
                        player_handles: vec![],
                        message: format!("{} joined!", name),
                    });
                }
                peer_names.map.insert(sender, name);
            }
            PeerMessage::Chat { message } => {
                // ignore the message if it came from an unregistered source
                match (peer_handles.map.get(&sender), peer_names.map.get(&sender)) {
                    (Some(handle), _) => {
                        messenger.send(ChatMessage::message(message).by_player(*handle));
                    }
                    (None, Some(name)) => {
                        messenger.send(ChatMessage::message(message).by(name.clone()));
                    }
                    _ => {}
                }
            }
        }
    }
}

// todo actually go through how all this works and should works, high risk of going wtf
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
                        player_handles: vec![],
                        message: format!("{} left!", name),
                    });
                }
            }
            _ => {}
        }
    }
}

pub fn reset_peer_names(mut peer_names: ResMut<PeerNames>, mut peer_handles: ResMut<PeerHandles>) {
    peer_names.map.clear();
    peer_handles.map.clear();
}

pub(crate) struct OnlinePeerPlugin;
impl Plugin for OnlinePeerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PeerConnectionEvent>()
            .add_event::<PeerMessage>()
            .init_resource::<PeerNames>()
            .init_resource::<PeerHandles>()
            // ideally, there should be `or` between `Matchmaking` and `InGame`, but no, ok
            .add_system(handle_player_name_broadcast.run_if(not(in_state(GameState::MainMenu))))
            .add_system(handle_chat_message_broadcast.run_if(not(in_state(GameState::MainMenu))))
            .add_system(handle_receiving_peer_messages.run_if(not(in_state(GameState::MainMenu))))
            .add_system(
                handle_reporting_peer_disconnecting.run_if(not(in_state(GameState::MainMenu))),
            )
            .add_system(reset_peer_names.in_schedule(OnEnter(GameState::MainMenu)))
            .add_system(reset_peer_names.in_schedule(OnExit(GameState::InGame)));
    }
}
