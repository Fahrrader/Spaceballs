use crate::ui::text_input::TextInput;
use bevy::prelude::*;

/// Resource encapsulating a set of the application's user settings.
/// Each setting is identified by a `UserInputForm` and could be retrieved with it.
#[derive(Resource, Debug)]
pub struct UserSettings {
    pub player_name: String,
    pub server_url: String,
    pub room_name: String,
}

/// Enum representing different forms of user input, each associated with a unique setting.
/// Placed on an entity as a component, will eventually record other input components' values under the corresponding setting.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserInputForm {
    PlayerName,
    ServerUrl,
    RoomName,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            player_name: "Player".into(),
            server_url: "wss://match-0-6.helsing.studio".into(),
            room_name: "".into(),
        }
    }
}

impl UserSettings {
    /// Sets the value of the setting specified by `UserInputForm` to `value`.
    pub fn set(&mut self, setting: UserInputForm, value: String) {
        match setting {
            UserInputForm::PlayerName => self.player_name = value,
            UserInputForm::ServerUrl => self.server_url = value,
            UserInputForm::RoomName => self.room_name = value,
        }
    }

    /// Retrieves the setting value specified by `UserInputForm`. For specific queries, just reference the value directly.
    pub fn get(&self, setting: UserInputForm) -> String {
        match setting {
            UserInputForm::PlayerName => self.player_name.clone(),
            UserInputForm::ServerUrl => self.server_url.clone(),
            UserInputForm::RoomName => self.room_name.clone(),
        }
    }
}

/// System responsible for recording data from text inputs into settings.
/// Supposed to be called on arbitrary conditions - for example, at the end of the text input's lifespan.
pub fn transfer_setting_from_text_input(
    mut settings: ResMut<UserSettings>,
    input_query: Query<(&TextInput, &UserInputForm)>,
) {
    for (text_input, input_form) in input_query.iter() {
        let mut setting_value = text_input.text.clone();
        if setting_value.is_empty() {
            setting_value = UserSettings::default().get(*input_form);
        }
        settings.set(*input_form, setting_value);
    }
}

/// Plugin handling the [`UserSettings`] resource insertion.
pub(crate) struct UserSettingsPlugin;
impl Plugin for UserSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UserSettings::default());
    }
}
