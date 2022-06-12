use bevy::render::color::Color;

pub type Team = u8;

pub fn team_color(team: Team) -> Color {
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
        _ => panic!("The team number is too big!"),
    }
}
