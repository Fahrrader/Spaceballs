use bevy::prelude::{Component, FromReflect, Reflect};
use bevy::render::color::Color;

/// Number of teams is limited by 256.
pub type TeamNumber = u8;

/// The team number the player is created with.
pub const PLAYER_DEFAULT_TEAM: TeamNumber = 1;
/// The default team number of the AI enemies.
pub const AI_DEFAULT_TEAM: TeamNumber = 9;

/// Marker holding the character's (or anything's) allegiance.
#[derive(Component, Clone, Debug, Eq, PartialEq, Reflect, FromReflect)]
pub struct Team(pub TeamNumber);

impl Team {
    /// Get the color associated with the team.
    pub fn color(&self) -> Color {
        team_color(self.0)
    }
}

/// Get the color associated with the team.
pub fn team_color(team: TeamNumber) -> Color {
    match team {
        1 => Color::CYAN,
        2 => Color::CRIMSON,
        3 => Color::LIME_GREEN,
        4 => Color::GOLD,
        5 => Color::PURPLE,
        6 => Color::SEA_GREEN,
        7 => Color::ORANGE_RED,
        8 => Color::INDIGO,
        9 => Color::SILVER,
        _ => panic!("The team number is out of bounds!"),
    }
}
