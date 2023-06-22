use crate::ui::text_input::TextInput;
use bevy::prelude::*;
use bevy::utils::HashMap;

/// Resource encapsulating a hash map of the application's user settings.
/// Each setting is identified by a `UserInputForm` and holds a `SettingValue`.
#[derive(Resource, Debug)]
pub struct UserSettings(HashMap<UserInputForm, SettingValue>);

/// Enum representing different forms of user input, each associated with a unique setting.
/// Placed on an entity as a component, will eventually record other input components' values under the corresponding setting.
#[derive(Component, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum UserInputForm {
    PlayerName,
    ServerUrl,
    RoomName,
}

/// An enum representing possible value types for settings.
#[derive(Debug, Clone)]
pub enum SettingValue {
    String(String),
    Uint(usize),
    Bool(bool),
}

impl SettingValue {
    /// Transforms the `SettingValue` into a string, if it is a `String`.
    pub fn as_string(&self) -> Option<String> {
        match self {
            SettingValue::String(value) => Some(value.to_string()),
            _ => {
                debug!("You picked the wrong setting value type, fool!");
                None
            }
        }
    }

    /// Converts the `SettingValue` into a `usize` integer, if it is a `Uint`.
    pub fn as_usize(&self) -> Option<usize> {
        match self {
            SettingValue::Uint(value) => Some(*value),
            _ => {
                debug!("You picked the wrong setting value type, fool!");
                None
            }
        }
    }

    /// Converts the `SettingValue` into a `bool`, if it is a `Bool`.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            SettingValue::Bool(value) => Some(*value),
            _ => {
                debug!("You picked the wrong setting value type, fool!");
                None
            }
        }
    }
}

impl Into<SettingValue> for String {
    fn into(self) -> SettingValue {
        SettingValue::String(self)
    }
}

impl Into<SettingValue> for &str {
    fn into(self) -> SettingValue {
        SettingValue::String(self.to_string())
    }
}

impl Into<SettingValue> for usize {
    fn into(self) -> SettingValue {
        SettingValue::Uint(self)
    }
}

impl Into<SettingValue> for bool {
    fn into(self) -> SettingValue {
        SettingValue::Bool(self)
    }
}

impl Default for UserSettings {
    fn default() -> Self {
        Self(HashMap::from([
            (UserInputForm::PlayerName, "Player".into()),
            (UserInputForm::ServerUrl, "ws://localhost:3536".into()),
            (UserInputForm::RoomName, "".into()),
        ]))
    }
}

impl UserSettings {
    /// Sets the value of the setting specified by `setting` to `value` in the `UserSettings` hash map.
    pub fn set<T: Into<SettingValue>>(&mut self, setting: UserInputForm, value: T) {
        self.0.insert(setting, value.into());
    }

    /// Retrieves the `SettingValue` for the setting specified by `setting` from the `UserSettings` hash map.
    pub fn get(&self, setting: UserInputForm) -> Option<&SettingValue> {
        self.0.get(&setting)
    }

    /// Retrieves the `SettingValue` for the setting specified by `setting` as a `String`.
    #[allow(unused)]
    pub fn get_string(&self, setting: UserInputForm) -> Option<String> {
        self.get(setting)?.as_string()
    }

    /// Retrieves the `SettingValue` for the setting specified by `setting` as a `usize`.
    #[allow(unused)]
    pub fn get_usize(&self, setting: UserInputForm) -> Option<usize> {
        self.get(setting)?.as_usize()
    }

    /// Retrieves the `SettingValue` for the setting specified by `setting` as a `bool`.
    #[allow(unused)]
    pub fn get_bool(&self, setting: UserInputForm) -> Option<bool> {
        self.get(setting)?.as_bool()
    }
}

/// System, responsible for recording data from text inputs into settings.
/// It is supposed to be called on arbitrary conditions, for example, at the end of the text input's lifespan.
pub fn transfer_setting_from_text_input(
    mut settings: ResMut<UserSettings>,
    input_query: Query<(&TextInput, &UserInputForm)>,
) {
    for (text_input, input_form) in input_query.iter() {
        let mut setting_value = text_input.text.clone();
        if setting_value.is_empty() {
            setting_value = UserSettings::default()
                .get_string(*input_form)
                .unwrap_or_default();
        }
        settings.set(*input_form, setting_value);
    }
}

/// Plugin handling the [`UserSettings`] resource insertion.
pub struct UserSettingsPlugin;
impl Plugin for UserSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UserSettings::default());
    }
}
