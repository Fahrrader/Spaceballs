pub use bevy_ggrs::{GGRSPlugin, GGRSSchedule};

use crate::controls::CharacterActionInput;
use crate::{info, GameState};
use bevy::prelude::{Commands, NextState, Res, ResMut, Resource};
use bevy_ggrs::{ggrs, Session};
use bevy_matchbox::prelude::{MatchboxSocket, PeerId, SingleChannel};

/// Expected - and maximum - player count for the game session.
#[derive(Resource)]
pub struct PlayerCount(pub usize);

/// Struct onto which the GGRS config's types are mapped.
pub struct GGRSConfig;

// todo:mp state sync with the host -- at least compare it once in a while to complain (debug only)
// will need it anyway for drop-in
impl ggrs::Config for GGRSConfig {
    // 4 directions, fire, reload and 2 interact actions fit perfectly in a single byte
    // but todo:mp rework for a more complex struct
    type Input = GGRSInput;
    type State = u8;
    // Matchbox' WebRtcSocket addresses are called `PeerId`s
    type Address = PeerId;
}

/// Initialize a socket for connecting to the matchbox server.
pub fn start_matchbox_socket(mut commands: Commands) {
    let room_url = "ws://127.0.0.1:3536/extreme_bevy?next=2";
    info!("connecting to matchbox server: {:?}", room_url);
    commands.insert_resource(MatchboxSocket::new_ggrs(room_url));
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
        .with_input_delay(2);

    for (i, player) in players.into_iter().enumerate() {
        session_builder = session_builder
            .add_player(player, i)
            .expect("failed to add player");
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

/// Players' input data structure, used and encoded by GGRS and exchanged over the internet.
pub type GGRSInput = u8;

const INPUT_UP: u8 = 1 << 0;
const INPUT_DOWN: u8 = 1 << 1;
const INPUT_LEFT: u8 = 1 << 2;
const INPUT_RIGHT: u8 = 1 << 3;
const INPUT_FIRE: u8 = 1 << 4;
const INPUT_RELOAD: u8 = 1 << 5;
const INPUT_ENV_1: u8 = 1 << 6;
const INPUT_ENV_2: u8 = 1 << 7;

impl Into<GGRSInput> for CharacterActionInput {
    fn into(self) -> GGRSInput {
        let mut input = GGRSInput::default();
        if self.up > 0.0 {
            input |= INPUT_UP;
        }
        if self.up < 0.0 {
            input |= INPUT_DOWN;
        }
        if self.right > 0.0 {
            input |= INPUT_RIGHT;
        }
        if self.right < 0.0 {
            input |= INPUT_LEFT;
        }
        if self.fire {
            input |= INPUT_FIRE;
        }
        if self.reload {
            input |= INPUT_RELOAD;
        }
        if self.use_environment_1 {
            input |= INPUT_ENV_1;
        }
        if self.use_environment_2 {
            input |= INPUT_ENV_2;
        }
        input
    }
}

impl From<GGRSInput> for CharacterActionInput {
    fn from(value: GGRSInput) -> Self {
        CharacterActionInput {
            up: (value & INPUT_UP) as f32 - (value & INPUT_DOWN) as f32,
            right: (value & INPUT_RIGHT) as f32 - (value & INPUT_LEFT) as f32,
            fire: value & INPUT_FIRE != 0,
            reload: value & INPUT_RELOAD != 0,
            use_environment_1: value & INPUT_ENV_1 != 0,
            use_environment_2: value & INPUT_ENV_2 != 0,
        }
    }
}
