use crate::network::PlayerHandle;
use crate::{App, GameState};
use bevy::prelude::{in_state, info, not, EventReader, IntoSystemConfig, Plugin};

#[derive(Debug)]
pub struct ChatMessage {
    pub player_handle: Option<PlayerHandle>,
    pub message: String,
}

/// TEMPORARY
pub fn print_chat_messages(mut messenger: EventReader<ChatMessage>) {
    for message in messenger.iter() {
        info!(message.player_handle, message.message);
    }
}

pub struct ChatPlugin;
impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ChatMessage>()
            .add_system(print_chat_messages.run_if(not(in_state(GameState::MainMenu))));
    }
}
