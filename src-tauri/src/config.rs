use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct EncoderConfig {
    pub sensitivity: f32,
    pub up: char,
    pub down: char,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ApplicationProfile {
    pub encoder: Option<EncoderConfig>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub application_profiles: HashMap<String, ApplicationProfile>,
}
