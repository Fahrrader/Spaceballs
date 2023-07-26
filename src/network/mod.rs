pub mod controls;
pub mod ggrs_config;
pub mod peers;
pub mod players;
pub mod session;
pub mod socket;

use bevy::app::PluginGroupBuilder;
pub use bevy_ggrs::{GGRSPlugin, GGRSSchedule};
pub use controls::GGRSInput;
pub use ggrs_config::{GGRSConfig, PeerId, PlayerHandle};
pub use players::{PlayerDied, PlayerJoined, PlayerRegistry};
pub use session::PlayerCount;

use bevy::prelude::PluginGroup;
use peers::OnlinePeerPlugin;
use players::OnlinePlayerPlugin;
use session::SessionPlugin;
use socket::SocketPlugin;

// Having a load screen of just one frame helps with desync issues, some report.

pub struct MultiplayerPlugins;
impl PluginGroup for MultiplayerPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(SocketPlugin)
            .add(SessionPlugin)
            .add(OnlinePeerPlugin)
            .add(OnlinePlayerPlugin)
    }
}
