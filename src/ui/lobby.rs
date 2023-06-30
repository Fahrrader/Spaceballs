use crate::network::peers::PeerNames;
use crate::ui::menu::handle_menu_actions;
use crate::{GameState, MenuState, PlayerCount};
use bevy::prelude::*;

#[derive(Component)]
pub struct PeerWaitingText {
    pub number_section_idx: usize,
    pub plurality_section_idx: usize,
}

fn handle_peer_waiting_text(
    mut peer_waiting_text_query: Query<(&mut Text, &PeerWaitingText)>,
    desired_player_count: Res<PlayerCount>,
    peers: Res<PeerNames>,
) {
    if !(peers.is_changed() || desired_player_count.is_changed()) {
        return;
    }

    let remaining_peers = desired_player_count.0.saturating_sub(1 + peers.map.len());
    peer_waiting_text_query.for_each_mut(|(mut text, peer_waiting)| {
        if let Some(text) = text.sections.get_mut(peer_waiting.number_section_idx) {
            text.value = remaining_peers.to_string();
        }
        if let Some(text) = text.sections.get_mut(peer_waiting.plurality_section_idx) {
            text.value = {
                if remaining_peers != 1 {
                    "s"
                } else {
                    ""
                }
            }
            .to_string();
        }
    })
}

fn intercept_matchmaking_menu_state(
    mut next_menu_state: ResMut<NextState<MenuState>>,
    player_count: Option<Res<PlayerCount>>,
) {
    if let (Some(MenuState::MatchmakingLobby), Some(1)) =
        (next_menu_state.0, player_count.map(|count| count.0))
    {
        next_menu_state.set(MenuState::Disabled);
    }
}

fn disable_matchmaking_menu_state(mut menu_state: ResMut<NextState<MenuState>>) {
    menu_state.set(MenuState::Disabled);
}

pub(crate) struct LobbyPlugin;
impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(handle_peer_waiting_text.run_if(in_state(MenuState::MatchmakingLobby)))
            .add_system(disable_matchmaking_menu_state.in_schedule(OnExit(GameState::Matchmaking)))
            .add_system(
                intercept_matchmaking_menu_state
                    .run_if(in_state(GameState::MainMenu))
                    .after(handle_menu_actions),
            );
    }
}
