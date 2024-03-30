use serde::{Deserialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub blau_licht_sms_api: String,
    pub checking_interval_secs: u64,
    pub turn_off_interval_enabled: bool,
    pub turn_off_interval_secs: u64,
    pub use_hdmi_cec: bool,
    pub use_wake_on_lan: bool,
    pub wake_on_lan_mac_addr: String,
}