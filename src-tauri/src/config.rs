use std::collections::HashMap;

use anyhow::Result;
use dirs::home_dir;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub application_profiles: HashMap<String, ApplicationProfile>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationProfile {
    pub bindings: Vec<(Action, Command)>,
}

impl ApplicationProfile {
    pub fn get_binding(&self, action: &Action) -> Option<Command> {
        self.bindings
            .iter()
            .find(|(a, _)| a == action)
            .map(|(_, b)| b.clone())
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Command {
    pub display_name: String,
    pub operations: Option<Vec<Operation>>,
    pub radial_menu_items: Option<Vec<RadialMenuItem>>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RadialMenuItem {
    pub label: String,
    pub command: Command,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum Action {
    ButtonPress { id: u8 },
    EncoderDecrement { id: u8 },
    EncoderIncrement { id: u8 },
    // Not for use in config
    #[default]
    None,
    ButtonRelease { id: u8 },
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Operation {
    KeyPress { key: String },
    KeyTap { key: String },
    Delay { ms: u64 },
    Repeat { 
        times: u64, 
        operations: Vec<Operation>
    },
    // Not for use in config
    #[default]
    None,
    KeyRelease { key: String },
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
