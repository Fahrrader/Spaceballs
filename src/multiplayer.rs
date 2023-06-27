pub use bevy_ggrs::{GGRSPlugin, GGRSSchedule};

use crate::controls::CharacterActionInput;
use crate::ui::user_settings::{UserInputForm, UserSettings};
use crate::{info, GameState};
use bevy::core::{Pod, Zeroable};
use bevy::prelude::{Commands, Component, NextState, Res, ResMut, Resource};
use bevy::reflect::{FromReflect, Reflect};
use bevy_ggrs::ggrs::DesyncDetection;
use bevy_ggrs::{ggrs, Session};
use bevy_matchbox::prelude::{
    ChannelConfig, MatchboxSocket, PeerId, SingleChannel, WebRtcSocketBuilder,
};
use bevy_matchbox::CloseSocketExt;

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

#[derive(Resource)]
pub struct LocalPlayerHandle(pub usize);

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
    let room_url = if player_count.0 > 1 {
        format!(
            "{}/spaceballs?next={}",
            settings
                .get_string(UserInputForm::ServerUrl)
                .unwrap_or_default(),
            settings
                .get_string(UserInputForm::RoomName)
                .unwrap_or_default(),
        )
        .to_lowercase()
    } else {
        "".to_string()
    };
    let reconnect_attempts = if player_count.0 > 1 { Some(3) } else { None };
    info!("connecting to matchbox server: {:?}", room_url);
    commands.insert_resource(MatchboxSocket::<SingleChannel>::from(
        WebRtcSocketBuilder::new(room_url)
            .reconnect_attempts(reconnect_attempts)
            .add_channel(ChannelConfig::ggrs())
            .build(),
    ));
}

pub fn sever_connection(mut commands: Commands) {
    commands.close_socket::<SingleChannel>();
    commands.remove_resource::<Session<GGRSConfig>>();
    // ... and maybe more
}

/// Initialize the multiplayer session.
/// Having input systems in GGRS schedule will not execute them until a session is initialized.
pub fn wait_for_players(
    mut commands: Commands,
    mut socket: ResMut<MatchboxSocket<SingleChannel>>,
    player_count: Res<PlayerCount>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Check for new connections
    socket.update_peers();
    let players = socket.players();

    // todo:mp try players.len() (i.e. drop-in)
    // if there is not enough players, wait
    if players.len() < player_count.0 {
        /*if session.is_none() {
           // remove resource
           // unneeded, do drop-in, drop-out
        }*/
        return; // wait for more players
    }

    info!("All peers have joined, going in-game");

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

    for (i, player) in players.into_iter().enumerate() {
        session_builder = session_builder
            .add_player(player, i)
            .expect("failed to add player");

        if matches!(player, bevy_ggrs::ggrs::PlayerType::Local) {
            commands.insert_resource(LocalPlayerHandle(i));
        }
        // todo:mp add players here?
    }

    // move the channel out of the socket (required because GGRS takes ownership of it)
    let channel = socket.take_channel(0).unwrap();

    // start the GGRS session
    let ggrs_session = session_builder
        .start_p2p_session(channel)
        .expect("failed to start session");

    commands.insert_resource(Session::P2PSession(ggrs_session));
    next_state.set(GameState::InGame);
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
