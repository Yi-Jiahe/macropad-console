use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ApplicationAction {
    OpenRadialMenu { items: Vec<Box<RadialMenuItem>> },
    KeyPress { key: String },
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ApplicationProfile {
    pub actions: Vec<(Action, ApplicationAction)>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub application_profiles: HashMap<String, ApplicationProfile>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RadialMenuItem {
    pub label: String,
    pub actions: Vec<ApplicationAction>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum Action {
    ButtonPress { button: u8 },
    // Not for use in config
    ButtonRelease { button: u8 },
}
