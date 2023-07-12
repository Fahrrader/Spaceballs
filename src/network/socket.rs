use crate::network::peers::PeerMessage;
use crate::network::session::PlayerCount;
use crate::ui::user_settings::UserSettings;
use crate::{App, GameState};
use bevy::log::prelude::*;
use bevy::prelude::{Commands, IntoSystemAppConfig, OnEnter, Plugin, Res, Resource};
use bevy::tasks::IoTaskPool;
use bevy_ggrs::ggrs;
use bevy_ggrs::ggrs::PlayerType;
use bevy_matchbox::matchbox_socket::{MessageLoopFuture, WebRtcSocket};
use bevy_matchbox::prelude::{MultipleChannels, PeerId};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Resource, Debug, Clone)]
pub struct SpaceballSocket(pub Arc<RwLock<WebRtcSocket<MultipleChannels>>>);

impl ggrs::NonBlockingSocket<PeerId> for SpaceballSocket {
    fn send_to(&mut self, msg: &ggrs::Message, addr: &PeerId) {
        self.0
            .write()
            // if the lock is poisoned, we're already doomed, time to panic
            .expect("Failed to lock socket for sending!")
            .channel(Self::GGRS_CHANNEL)
            .send_to(msg, addr);
    }

    fn receive_all_messages(&mut self) -> Vec<(PeerId, ggrs::Message)> {
        self.0
            .write()
            // if the lock is poisoned, we're already doomed, time to panic
            .expect("Failed to lock socket for receiving!")
            .channel(Self::GGRS_CHANNEL)
            .receive_all_messages()
    }
}

impl From<(WebRtcSocket<MultipleChannels>, MessageLoopFuture)> for SpaceballSocket {
    fn from(
        (socket, message_loop_fut): (WebRtcSocket<MultipleChannels>, MessageLoopFuture),
    ) -> Self {
        let task_pool = IoTaskPool::get();
        task_pool.spawn(message_loop_fut).detach();
        SpaceballSocket(Arc::new(RwLock::new(socket)))
    }
}

impl SpaceballSocket {
    pub const GGRS_CHANNEL: usize = 0;
    pub const RELIABLE_CHANNEL: usize = 1;

    #[allow(unused)]
    pub fn send_tcp_message(&mut self, peer: PeerId, message: PeerMessage) {
        let bytes = bincode::serialize(&message).expect("failed to serialize message");
        self.inner_mut()
            .channel(Self::RELIABLE_CHANNEL)
            .send(bytes.into(), peer);
    }

    #[allow(unused)]
    pub fn broadcast_tcp_message(&mut self, message: PeerMessage) {
        let bytes = bincode::serialize(&message).expect("failed to serialize message");
        let peers = self.inner().connected_peers().collect::<Vec<_>>();
        for peer in peers {
            self.inner_mut()
                .channel(Self::RELIABLE_CHANNEL)
                .send(bytes.clone().into(), peer);
        }
    }

    #[allow(unused)]
    pub fn receive_tcp_messages(&mut self) -> Vec<(PeerId, PeerMessage)> {
        self.inner_mut()
            .channel(Self::RELIABLE_CHANNEL)
            .receive()
            .into_iter()
            .map(|(id, packet)| {
                let msg = bincode::deserialize(&packet).unwrap();
                (id, msg)
            })
            .collect()
    }

    #[allow(unused)]
    pub fn players(&self) -> Vec<PlayerType<PeerId>> {
        let Some(our_id) = self.inner().id() else {
            // we're still waiting for the server to initialize our id
            // no peers should be added at this point anyway
            return vec![PlayerType::Local];
        };

        // player order needs to be consistent order across all peers
        let mut ids: Vec<_> = self
            .inner()
            .connected_peers()
            .chain(std::iter::once(our_id))
            .collect();
        ids.sort();

        ids.into_iter()
            .map(|id| {
                if id == our_id {
                    PlayerType::Local
                } else {
                    PlayerType::Remote(id)
                }
            })
            .collect()
    }

    #[allow(unused)]
    pub fn inner(&self) -> RwLockReadGuard<'_, WebRtcSocket<MultipleChannels>> {
        // we don't care about handling lock poisoning
        self.0.read().expect("Failed to lock socket for reading!")
    }

    #[allow(unused)]
    pub fn inner_mut(&mut self) -> RwLockWriteGuard<'_, WebRtcSocket<MultipleChannels>> {
        // we don't care about handling lock poisoning
        self.0.write().expect("Failed to lock socket for writing!")
    }
}

/// Initialize a socket for connecting to the matchbox server.
pub fn start_matchbox_socket(
    mut commands: Commands,
    player_count: Res<PlayerCount>,
    settings: Res<UserSettings>,
) {
    let (room_url, reconnect_attempts) = if player_count.0 > 1 {
        (
            format!(
                "{}/spaceballs?next={}",
                settings.server_url, settings.room_name,
            )
            .to_lowercase(),
            Some(3),
        )
    } else {
        ("ws://localhost/spaceballs?next=".to_string(), None)
    };
    info!("Connecting to Matchbox server: {:?}", room_url);
    commands.insert_resource(SpaceballSocket::from(
        WebRtcSocket::builder(room_url)
            .reconnect_attempts(reconnect_attempts)
            .add_ggrs_channel()
            .add_reliable_channel()
            .build(),
    ));
}

pub(crate) struct SocketPlugin;
impl Plugin for SocketPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(start_matchbox_socket.in_schedule(OnEnter(GameState::Matchmaking)));
    }
}
