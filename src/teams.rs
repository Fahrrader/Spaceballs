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
pub fn try_team_color(team: TeamNumber) -> Option<Color> {
    match team {
        1 => Some(Color::CYAN),
        2 => Some(Color::CRIMSON),
        3 => Some(Color::LIME_GREEN),
        4 => Some(Color::GOLD),
        5 => Some(Color::PURPLE),
        6 => Some(Color::SEA_GREEN),
        7 => Some(Color::ORANGE_RED),
        8 => Some(Color::INDIGO),
        9 => Some(Color::SILVER),
        _ => None,
    }
}

pub fn team_color(team: TeamNumber) -> Color {
    try_team_color(team).expect("The team number is out of bounds!")
}
