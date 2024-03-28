use serde::{Deserialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub blau_licht_sms_api: String
}