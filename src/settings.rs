use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(rename = "mapping")]
    pub instruments_mapping: HashMap<String, String>,

    #[serde(rename = "target_exchange")]
    pub target_exchange: String
}