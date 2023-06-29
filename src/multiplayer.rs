pub use bevy_ggrs::{GGRSPlugin, GGRSSchedule};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::controls::CharacterActionInput;
use crate::ui::user_settings::{UserInputForm, UserSettings};
use crate::{default, GameState};
use bevy::core::{Pod, Zeroable};
use bevy::log::{error, info, warn};
#[cfg(feature = "diagnostic")]
use bevy::prelude::Local;
use bevy::prelude::{
    Commands, Component, DetectChanges, EventReader, EventWriter, NextState, Res, ResMut, Resource,
};
use bevy::reflect::{FromReflect, Reflect};
use bevy::tasks::IoTaskPool;
use bevy::utils::HashMap;
pub use bevy_ggrs::ggrs::PlayerHandle;
use bevy_ggrs::ggrs::{DesyncDetection, PlayerType};
use bevy_ggrs::{ggrs, Session};
use bevy_matchbox::matchbox_socket::{MessageLoopFuture, WebRtcSocket};
use bevy_matchbox::prelude::{MultipleChannels, PeerId, PeerState};

// Common room address for all matches on the server! Oh it's going to be gloriously broken if left like this.
// pub const ROOM_NAME: &str = "spaceballs";

pub const MAINTAINED_FPS: usize = 60;
pub const MAX_PREDICTION_FRAMES: usize = 5;
pub const INPUT_DELAY: usize = 2;

/// Expected - and maximum - player count for the game session.
#[derive(Resource)]
pub struct PlayerCount(pub usize);

/// Struct onto which the GGRS config's types are mapped.
#[derive(Debug)]
pub struct GGRSConfig;

// todo:mp state sync with the host -- at least compare it once in a while to complain (debug only)
// will need it anyway for drop-in
impl ggrs::Config for GGRSConfig {
    type Input = GGRSInput;
    type State = u8;
    // Matchbox' WebRtcSocket addresses are called `PeerId`s
    type Address = PeerId;
}

#[derive(Resource, Debug, Clone)]
pub struct SpaceballSocket(pub Arc<RwLock<WebRtcSocket<MultipleChannels>>>);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PeerMessage {
    PlayerName { name: String },
    Chat { message: String },
}

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
    const GGRS_CHANNEL: usize = 0;
    const RELIABLE_CHANNEL: usize = 1;

    pub fn send_tcp_message(&mut self, peer: PeerId, message: PeerMessage) {
        let bytes = bincode::serialize(&message).expect("failed to serialize message");
        self.inner_mut()
            .channel(Self::RELIABLE_CHANNEL)
            .send(bytes.into(), peer);
    }

    pub fn broadcast_tcp_message(&mut self, message: PeerMessage) {
        let bytes = bincode::serialize(&message).expect("failed to serialize message");
        let peers = self.inner().connected_peers().collect::<Vec<_>>();
        for peer in peers {
            self.inner_mut()
                .channel(Self::RELIABLE_CHANNEL)
                .send(bytes.clone().into(), peer);
        }
    }

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

    pub fn inner(&self) -> RwLockReadGuard<'_, WebRtcSocket<MultipleChannels>> {
        // we don't care about handling lock poisoning
        self.0.read().expect("Failed to lock socket for reading!")
    }

    pub fn inner_mut(&mut self) -> RwLockWriteGuard<'_, WebRtcSocket<MultipleChannels>> {
        // we don't care about handling lock poisoning
        self.0.write().expect("Failed to lock socket for writing!")
    }
}

#[derive(Debug, Clone)]
pub struct PeerConnectionEvent {
    pub id: PeerId,
    pub state: PeerState,
}

/// Check for new connections and send out events.
///
/// NOTE: Exercise caution when calling `update_peers` on the socket, or the new connections will not be registered by one system or another.
pub fn update_peers(
    mut socket: ResMut<SpaceballSocket>,
    mut peer_updater: EventWriter<PeerConnectionEvent>,
    #[cfg(feature = "diagnostic")] mut n_recorded_peers: Local<usize>,
) {
    let new_peers = socket.inner_mut().update_peers();
    for (id, state) in new_peers {
        peer_updater.send(PeerConnectionEvent { id, state });
    }

    // todo some debug mode maybe?
    #[cfg(feature = "diagnostic")]
    {
        let n_connected_peers = socket.inner().connected_peers().count();
        if *n_recorded_peers != n_connected_peers {
            error!(
                "Someone hijacked our sweet peers! Peer update has been lost. Previous number of connections: {}, new number of connections: {}",
                *n_recorded_peers,
                n_connected_peers,
            );
        }
        *n_recorded_peers = n_connected_peers;
    }
}

#[derive(Resource)]
pub struct LocalPlayerHandle(pub PlayerHandle);

#[derive(Component)]
pub struct LocalPlayer;

// Having a load screen of just one frame helps with desync issues, some report.

// Bevy-Extremists host this match making service for us to use FOR FREE.
// So, use Johan's compatible matchbox.
// "wss://match-0-6.helsing.studio/bevy-ggrs-rapier-example?next=2";
// Check out their work on "Cargo Space", especially the blog posts, which are incredibly enlightening!
// https://johanhelsing.studio/cargospace

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
                settings
                    .get_string(UserInputForm::ServerUrl)
                    .unwrap_or_default(),
                settings
                    .get_string(UserInputForm::RoomName)
                    .unwrap_or_default(),
            )
            .to_lowercase(),
            Some(3),
        )
    } else {
        ("".to_string(), None)
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

pub fn sever_connection(mut commands: Commands) {
    commands.remove_resource::<SpaceballSocket>();
    commands.remove_resource::<Session<GGRSConfig>>();
    // ... and maybe more
}

/// Initialize the multiplayer session.
/// Having input systems in GGRS schedule will not execute them until a session is initialized.
pub fn wait_for_players(
    mut commands: Commands,
    mut socket: ResMut<SpaceballSocket>,
    player_count: Res<PlayerCount>,
    settings: Res<UserSettings>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Check for new players
    let players = socket.players();

    // if there is not enough players, wait
    if players.len() < player_count.0 {
        // wait for more players
        return;
    }

    if players.len() > player_count.0 {
        error!("You are trying to join an already full game! Exiting to main menu.");
        // todo test without when `update_peers` is called externally. Maybe that would let a spectator in.
        next_state.set(GameState::MainMenu);
        return;
    } else {
        info!("All peers have joined, going in-game");
    }

    // create a GGRS P2P session
    let mut session_builder = ggrs::SessionBuilder::<GGRSConfig>::new()
        .with_num_players(player_count.0)
        .with_fps(MAINTAINED_FPS)
        .expect("Invalid FPS")
        .with_max_prediction_window(MAX_PREDICTION_FRAMES)
        // just in case *shrug*
        .with_desync_detection_mode(DesyncDetection::On {
            interval: MAINTAINED_FPS as u32,
        })
        .with_input_delay(INPUT_DELAY);

    let mut peer_handles = PeerHandles::default();
    let mut player_registry = PlayerRegistry::default();

    for (i, player) in players.into_iter().enumerate() {
        session_builder = session_builder
            .add_player(player, i)
            .expect("failed to add player");

        match player {
            PlayerType::Remote(peer_id) => {
                player_registry.0.push(PlayerData::default());
                peer_handles.map.insert(peer_id, i);
            }
            PlayerType::Local => {
                player_registry.0.push(PlayerData {
                    name: settings
                        .get_string(UserInputForm::PlayerName)
                        .unwrap_or_default(),
                    ..default()
                });
                commands.insert_resource(LocalPlayerHandle(i));
            }
            PlayerType::Spectator(_) => {}
        };
    }

    commands.insert_resource(peer_handles);
    commands.insert_resource(player_registry);

    // move the channel out of the socket (required because GGRS takes ownership of it)
    let channel = socket
        .inner_mut()
        .take_channel(SpaceballSocket::GGRS_CHANNEL)
        .unwrap();

    // start the GGRS session
    let ggrs_session = session_builder
        .start_p2p_session(channel)
        .expect("failed to start session");

    commands.insert_resource(Session::P2PSession(ggrs_session));
    next_state.set(GameState::InGame);
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

#[derive(Resource, Debug, Default)]
pub struct PeerHandles {
    pub map: HashMap<PeerId, PlayerHandle>,
}

#[derive(Resource, Debug, Default)]
pub struct PeerNames {
    pub map: HashMap<PeerId, String>,
}

#[derive(Debug)]
pub struct ChatMessage {
    pub player_handle: Option<PlayerHandle>,
    pub message: String,
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

/// TEMPORARY
pub fn print_chat_messages(mut messenger: EventReader<ChatMessage>) {
    for message in messenger.iter() {
        info!(message.player_handle, message.message);
    }
}

#[derive(Debug, Default)]
pub struct PlayerData {
    name: String,
    // team?
    kills: usize,
    deaths: usize,
}

#[derive(Resource, Debug, Default)]
pub struct PlayerRegistry(pub Vec<PlayerData>);

impl PlayerRegistry {
    pub fn get(&self, handle: usize) -> Option<&PlayerData> {
        self.0.get(handle).or_else(|| {
            warn!("Could not find player by handle {}!", handle);
            None
        })
    }
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

// delete probably, it does not detect desync in the rolled-back components (transform)
pub fn detect_desync(session: ResMut<Session<GGRSConfig>>) {
    if let Session::P2PSession(p2p_session) = session.into_inner() {
        let events = p2p_session.events();
        if events.len() != 0 {
            println!("GGRS got {} events.", events.len());
            for event in events {
                /*if matches!(event, GGRSEvent::DesyncDetected { .. }) {
                    println!("Desync detected: {:?}", event);
                }*/
                println!("Hi, I'm {:?}", event);
            }
        }
    }
}

/// Players' input data structure, used and encoded by GGRS and exchanged over the internet.
#[derive(Copy, Clone, Debug, Default, PartialEq, Pod, Zeroable, Reflect, FromReflect)]
#[repr(C)]
pub struct GGRSInput {
    pub up: f32,
    pub right: f32,
    // bytemuck::Pod does not accept "padding"/uninit bytes,
    // therefore fields must make up a multiple of the byte size of the biggest field
    pub bit_flags: u32,
}

mod input_flags {
    pub const FIRE: u32 = 1 << 0;
    pub const RELOAD: u32 = 1 << 1;
    pub const INTERACT_1: u32 = 1 << 2;
    pub const INTERACT_2: u32 = 1 << 3;
}

impl Into<GGRSInput> for CharacterActionInput {
    fn into(self) -> GGRSInput {
        use input_flags::*;

        let mut input = GGRSInput::default();

        input.up = self.up;
        input.right = self.right;

        if self.fire {
            input.bit_flags |= FIRE;
        }
        if self.reload {
            input.bit_flags |= RELOAD;
        }
        if self.interact_1 {
            input.bit_flags |= INTERACT_1;
        }
        if self.interact_2 {
            input.bit_flags |= INTERACT_2;
        }

        input
    }
}

impl From<GGRSInput> for CharacterActionInput {
    fn from(value: GGRSInput) -> Self {
        use input_flags::*;

        CharacterActionInput {
            up: value.up,
            right: value.right,
            fire: value.bit_flags & FIRE != 0,
            reload: value.bit_flags & RELOAD != 0,
            interact_1: value.bit_flags & INTERACT_1 != 0,
            interact_2: value.bit_flags & INTERACT_2 != 0,
        }
    }
}
