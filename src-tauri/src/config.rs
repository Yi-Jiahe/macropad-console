use std::fs;
use std::collections::{HashMap, HashSet};

use anyhow::Result;
use dirs::home_dir;

use serde::{ser, de, Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub application_profiles: HashMap<String, ApplicationProfile>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationProfile {
    pub bindings: Vec<(KeyCombination, Command)>,
}

impl ApplicationProfile {
    pub fn get_binding(&self, key_combination: &KeyCombination) -> Option<Command> {
        self.bindings
            .iter()
            .find(|(a, _)| a == key_combination)
            .map(|(_, b)| b.clone())
    }
}

#[derive(Debug, Clone)]
pub struct KeyCombination {
    // Button ids
    pub modifiers: Option<HashSet<u8>>,
    pub action: Action,
}

impl Serialize for KeyCombination {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = "".to_string();

        if let Some(modifiers) = &self.modifiers {
            let mut m = modifiers.clone().into_iter().collect::<Vec<u8>>();
            m.sort();
            let modifiers = m.iter().map(|x| format!("BTN_{}+", x)).collect::<Vec<String>>().join("");
            s = modifiers;
        }

        match self.action {
            Action::ButtonPress { id } => return serializer.serialize_str(&format!("{}BTN_{}", s, id)),
            Action::EncoderDecrement { id } => return serializer.serialize_str(&format!("{}ENC_{}_DEC", s, id)),
            Action::EncoderIncrement { id } => return serializer.serialize_str(&format!("{}ENC_{}_INC", s, id)),
            _ => return Err(ser::Error::custom("Invalid action")) 
        }
    }
}

impl<'de> Deserialize<'de> for KeyCombination {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let keys = s.split("+").collect::<Vec<&str>>();
        let n = keys.len();

        let mut c = match keys[n - 1].split("_").collect::<Vec<&str>>()[..] {
            ["BTN", x] => KeyCombination { modifiers: None, action: Action::ButtonPress { id: x.parse().unwrap() } },
            ["ENC", x, "DEC"] => KeyCombination { modifiers: None, action: Action::EncoderDecrement { id: x.parse().unwrap() } },
            ["ENC", x, "INC"] => KeyCombination { modifiers: None, action: Action::EncoderIncrement { id: x.parse().unwrap() } }, 
            _ => return Err(de::Error::custom("Invalid action")),
        };

        if n == 1 {
            return Ok(c);
        }

        let mut modifiers = HashSet::new();
        for k in keys.iter().take(n - 1) {
            let id = match k.split("_").collect::<Vec<&str>>()[..] {
                ["BTN", x] => x.parse().unwrap(),
                _ => return Err(de::Error::custom("Invalid key")),
            };
            modifiers.insert(id);
        }

        c.modifiers = Some(modifiers);
        Ok(c)
    }
}

impl PartialEq for KeyCombination {
    fn eq(&self, other: &Self) -> bool {
        self.modifiers == other.modifiers && self.action == other.action
    }
}

impl Eq for KeyCombination {}

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
pub struct Command {
    pub display_name: String,
    pub operations: Option<Vec<Operation>>,
    pub radial_menu_items: Option<Vec<RadialMenuItem>>,
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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RadialMenuItem {
    pub label: String,
    pub command: Command,
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
    if !fs::metadata(&config_path).is_ok() {
        // Create directory if it doesn't exist
        fs::create_dir_all(config_path.parent().unwrap())?;
        // Create config file
        fs::File::create(&config_path)?;
        // Write default config
        fs::write(&config_path, serde_json::to_string(&AppConfig::default()).unwrap()).unwrap();
    }
    let config = std::fs::read_to_string(config_path)?;
    Ok(serde_json::from_str(&config)?)
}
