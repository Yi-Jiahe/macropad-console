use serde::{Deserialize, Serialize};

use crate::config;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ShowRadialMenu {
    pub items: Vec<config::RadialMenuItem>,
}

pub type SelectedRadialMenuItem = config::RadialMenuItem;
