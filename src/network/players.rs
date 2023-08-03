use crate::network::peers::{PeerHandles, PeerNames};
use crate::network::{PlayerHandle, MAINTAINED_FPS_F64};
use crate::teams::{Team, TeamNumber, PLAYER_DEFAULT_TEAM};
use crate::{GameState, MenuState, PlayerCount};
use bevy::prelude::*;
use std::slice::Iter;
use std::time::Duration;

#[derive(Default, Debug)]
pub struct PlayerData {
    pub name: String,
    pub team: Team,
    pub kills: usize,
    pub deaths: usize,
}

impl PlayerData {
    pub fn from_player_handle(player_handle: PlayerHandle) -> Self {
        Self {
            team: Team(PLAYER_DEFAULT_TEAM + player_handle as TeamNumber),
            ..default()
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }
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
    pub killed_by: Option<PlayerHandle>,
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

impl std::ops::Deref for PlayerRegistry {
    type Target = [PlayerData];

    fn deref(&self) -> &Self::Target {
        &self.0[..]
    }
}

impl<'a> IntoIterator for &'a PlayerRegistry {
    type Item = <Self::IntoIter as Iterator>::Item;

    type IntoIter = Iter<'a, PlayerData>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[derive(Resource, Debug)]
pub struct MatchTime(pub Timer);

impl Default for MatchTime {
    fn default() -> Self {
        Self(Timer::new(Duration::from_secs(1 * 60), TimerMode::Once))
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

pub fn send_new_players_joined(
    players: Res<PlayerRegistry>,
    mut player_teller: EventWriter<PlayerJoined>,
    mut previous_players_len: Local<usize>,
    state: Res<State<GameState>>,
) {
    if state.is_changed() {
        *previous_players_len = 0;
    }
    if *previous_players_len != players.len() {
        // maybe should compare to all existing players, see if their PlayerControlled characters exist
        // for drop-in
        for index in *previous_players_len..players.len() {
            player_teller.send(PlayerJoined {
                player_handle: index,
            });
        }
        *previous_players_len = players.len();
    }
}

fn reset_match_time_in_multiplayer(mut commands: Commands, player_count: Res<PlayerCount>) {
    if player_count.0 > 1 {
        commands.init_resource::<MatchTime>();
    }
}

pub fn handle_match_time(
    mut commands: Commands,
    match_time: Option<ResMut<MatchTime>>,
    mut menu_state: ResMut<NextState<MenuState>>,
) {
    if match_time.is_none() {
        return;
    }

    let mut match_time = match_time.unwrap();
    // GGRS fixed ticks
    match_time
        .0
        .tick(Duration::from_secs_f64(1. / MAINTAINED_FPS_F64));

    if match_time.0.finished() {
        menu_state.set(MenuState::MatchEnd);
        commands.remove_resource::<MatchTime>();
    }
}

pub(crate) struct OnlinePlayerPlugin;
impl Plugin for OnlinePlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerRegistry>()
            .add_event::<PlayerJoined>()
            .add_event::<PlayerDied>()
            .add_system(update_player_names.run_if(in_state(GameState::InGame)))
            .add_system(reset_match_time_in_multiplayer.in_schedule(OnEnter(GameState::InGame)));
    }
}
