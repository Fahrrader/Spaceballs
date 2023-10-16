use crate::network::ggrs_config::GGRSConfig;
use crate::network::peers::{PeerConnectionEvent, PeerHandles};
use crate::network::players::{PlayerData, PlayerRegistry};
use crate::network::socket::SpaceballSocket;
use crate::network::PlayerHandle;
use crate::ui::user_settings::UserSettings;
use crate::GameState;
use bevy::log::prelude::*;
#[cfg(feature = "diagnostic")]
use bevy::prelude::Local;
use bevy::prelude::{
    in_state, not, App, Commands, Component, EventWriter, IntoSystemAppConfig, IntoSystemConfig,
    NextState, OnExit, Plugin, Res, ResMut, Resource,
};
use bevy_ggrs::ggrs::PlayerType;
#[cfg(feature = "diagnostic")]
use bevy_ggrs::GGRSSchedule;
use bevy_ggrs::Session;

// Bevy-Extremists host this match making service for us to use FOR FREE.
// So, use Johan's compatible matchbox.
// "wss://match-0-6.helsing.studio/bevy-ggrs-rapier-example?next=2";
// Check out their work on "Cargo Space", especially the blog posts, which are incredibly enlightening!
// https://johanhelsing.studio/cargospace

// pub const ROOM_NAME: &str = "spaceballs";

/// Expected - and maximum - player count for the game session.
#[derive(Resource)]
pub struct PlayerCount(pub usize);

impl std::ops::Deref for PlayerCount {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Resource containing a player handle from the GGRS session as having [`PlayerType::Local`].
#[derive(Resource)]
pub struct LocalPlayerHandle(pub PlayerHandle);

/// Marker component indicating that this entity should be used for some peer-side-only systems, like HUD.
#[derive(Component)]
pub struct LocalPlayer;

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

/// Initialize the multiplayer session.
/// Having input systems in GGRS schedule will not execute them until a session is initialized.
/// Will wait until all players have joined.
pub fn build_session(
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
        // test without when `update_peers` is called externally. Maybe that would let a spectator in.
        next_state.set(GameState::MainMenu);
        return;
    } else {
        info!("All peers have joined, going in-game");
    }

    // create a GGRS P2P session
    let mut session_builder = GGRSConfig::new_builder().with_num_players(player_count.0);

    let mut peer_handles = PeerHandles::default();
    let mut player_registry = PlayerRegistry::default();

    for (i, player) in players.into_iter().enumerate() {
        session_builder = session_builder
            .add_player(player, i)
            .expect("failed to add player");

        // todo ensure consistency of team ordering, i.e the order of players joining? Nah.
        // maybe implement ability to choose own color or join a specific team later, for now tis will do.
        match player {
            PlayerType::Remote(peer_id) => {
                player_registry
                    .0
                    .push(PlayerData::default().with_team_from_handle(i));
                peer_handles.map.insert(peer_id, i);
            }
            PlayerType::Local => {
                player_registry.0.push(
                    PlayerData::default()
                        .with_team_from_handle(i)
                        .with_name(settings.player_name.clone()),
                );
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

pub fn sever_connection(mut commands: Commands) {
    commands.remove_resource::<SpaceballSocket>();
    commands.remove_resource::<Session<GGRSConfig>>();
    // ... and maybe more
}

#[allow(unused)]
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

pub(crate) struct SessionPlugin;
impl Plugin for SessionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_peers.run_if(not(in_state(GameState::MainMenu))))
            .add_system(build_session.run_if(in_state(GameState::Matchmaking)))
            .add_system(sever_connection.in_schedule(OnExit(GameState::InGame)));

        #[cfg(feature = "diagnostic")]
        app.add_system(
            detect_desync
                .in_schedule(GGRSSchedule)
                .run_if(in_state(GameState::InGame)),
        );
    }
}
