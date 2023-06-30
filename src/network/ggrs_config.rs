use crate::network::controls::GGRSInput;
use bevy_ggrs::ggrs;

pub use bevy_ggrs::ggrs::PlayerHandle;
pub use bevy_matchbox::prelude::PeerId;

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
