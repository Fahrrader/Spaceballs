use crate::network::controls::GGRSInput;
use bevy_ggrs::ggrs::{Config, DesyncDetection, SessionBuilder};

pub use bevy_ggrs::ggrs::PlayerHandle;
pub use bevy_matchbox::prelude::PeerId;

/// Struct onto which the GGRS config's types are mapped.
#[derive(Debug)]
pub struct GGRSConfig;

// todo:mp state sync with the host -- at least compare it once in a while to complain (debug only)
// will need it anyway for drop-in
impl Config for GGRSConfig {
    type Input = GGRSInput;
    type State = u8;
    // Matchbox' WebRtcSocket addresses are called `PeerId`s
    type Address = PeerId;
}
pub const MAINTAINED_FPS: usize = 60;
pub const MAINTAINED_FPS_F64: f64 = MAINTAINED_FPS as f64;
pub const MAX_PREDICTION_FRAMES: usize = 5;
pub const INPUT_DELAY: usize = 2;

impl GGRSConfig {
    pub fn new_builder() -> SessionBuilder<Self> {
        SessionBuilder::<Self>::new()
            .with_fps(MAINTAINED_FPS)
            .expect("Invalid FPS")
            .with_max_prediction_window(MAX_PREDICTION_FRAMES)
            // just in case *shrug*
            .with_desync_detection_mode(DesyncDetection::On {
                interval: MAINTAINED_FPS as u32,
            })
            .with_input_delay(INPUT_DELAY)
    }
}
