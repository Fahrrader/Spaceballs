use bevy::prelude::Component;
use bevy::render::color::Color;

pub type TeamNumber = u8;

pub const PLAYER_DEFAULT_TEAM: TeamNumber = 0;
pub const AI_DEFAULT_TEAM: TeamNumber = 8;
pub const NONEXISTENT_TEAM: TeamNumber = 255;

#[derive(Component, Clone, Eq, PartialEq)]
pub struct Team(pub TeamNumber);

impl Team {
    pub fn color(&self) -> Color {
        team_color(self.0)
    }
}

pub fn team_color(team: TeamNumber) -> Color {
    match team {
        0 => Color::CYAN,
        1 => Color::CRIMSON,
        2 => Color::LIME_GREEN,
        3 => Color::GOLD,
        4 => Color::PURPLE,
        5 => Color::SEA_GREEN,
        6 => Color::ORANGE_RED,
        7 => Color::INDIGO,
        8 => Color::SILVER,
        NONEXISTENT_TEAM | _ => panic!("The team number is too big!"),
    }
}
