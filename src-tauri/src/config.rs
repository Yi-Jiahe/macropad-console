use std::collections::HashMap;

use anyhow::Result;
use dirs::home_dir;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ApplicationAction {
    OpenRadialMenu { items: Vec<Box<RadialMenuItem>> },
    KeyPress { key: String },
    KeyTap { key: String },
    // Should not include OpenRadialMenu
    MacroTap { actions: Vec<ApplicationAction> },
    Delay { ms: u64 },
    // Not for use in config
    #[default]
    None,
    KeyRelease { key: String },
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
    pub action: ApplicationAction,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum Action {
    ButtonPress { button: u8 },
    EncoderDecrement { id: u8 },
    EncoderIncrement { id: u8 },
    // Not for use in config
    #[default]
    None,
    ButtonRelease { button: u8 },
}

pub fn get_config_path() -> std::path::PathBuf {
    home_dir()
        .unwrap()
        .join(".macropad-console")
        .join("config.json")
}

pub fn load_config() -> Result<AppConfig> {
    let config_path = get_config_path();
    dbg!(&config_path);
    let config = std::fs::read_to_string(config_path)?;
    Ok(serde_json::from_str(&config)?)
}
