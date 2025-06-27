use serde::{Deserialize, Serialize};

use crate::config;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ShowRadialMenu {
    pub location: (i32, i32),
    pub items: Vec<config::RadialMenuItem>,
}

pub type SelectedRadialMenuItem = config::RadialMenuItem;
