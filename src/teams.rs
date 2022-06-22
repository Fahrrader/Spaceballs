use bevy::prelude::Component;
use bevy::render::color::Color;

pub type TeamNumber = u8;

/// The team number the player is created with.
pub const PLAYER_DEFAULT_TEAM: TeamNumber = 1;
/// The default team number of the AI enemies.
pub const AI_DEFAULT_TEAM: TeamNumber = 9;

#[derive(Component, Clone, Eq, PartialEq)]
pub struct Team(pub TeamNumber);

impl Team {
    pub fn color(&self) -> Color {
        team_color(self.0)
    }
}

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
        _ => panic!("The team number is too big!"),
    }
}
