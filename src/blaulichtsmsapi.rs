use std::string::ToString;
use std::time::Duration;
use chrono::FixedOffset;
use crossbeam_channel::{Sender, unbounded};
use generic_runtime::executor::Executor;
use generic_runtime::handler::Handler;
use generic_runtime::module_runner::ModuleRunner;
use lazy_static::lazy_static;
use log::{info, warn};
use rand::Rng;
use tokio::runtime::{Handle, Runtime};
use tokio::task::spawn_blocking;
use tracing_subscriber::fmt::format;
use crate::config::Config;
use crate::message::Message;
use chrono::prelude::DateTime;

lazy_static! {
    static ref BLAULICHTSMS_ENDPOINT: String = "https://api.blaulichtsms.net/blaulicht/api/".to_string();
    static ref ALARM_URL: String = format!("{}alarm/v1/dashboard/", *BLAULICHTSMS_ENDPOINT);
}

#[derive(Debug, Clone)]
pub struct BlaulichtSMSAPI {
    last_connection: u32,
    config: Config,
    sender: Sender<Message>
}

impl Executor<Message> for BlaulichtSMSAPI {
    fn execute(&self) -> Option<Message> {
        let api = self.config.blau_licht_sms_api.clone();
        let sender = self.sender.clone();
        info!("Checking...");
        Handle::current().spawn(async move {
            let body = reqwest::get(format!("https://api.blaulichtsms.net/blaulicht/api/alarm/v1/dashboard/{}",api)).await;
            if let Ok(body) = body {
                if let Ok(json) = body.json::<serde_json::Value>().await {
                    if let Some(alarms) = json["alarms"].as_array() {
                        let mut most_recent_datetime = chrono::offset::Utc::now() - Duration::from_secs(1*60*60);
                        //info!("One hour ago: {}", most_recent_datetime);
                        //info!("Current: {}", chrono::offset::Utc::now());
                        for alarm in alarms {
                            let alarm = DateTime::parse_from_rfc3339(alarm["alarmDate"].as_str().expect("Could not parse datetime!")).unwrap().to_utc();
                            if (alarm > most_recent_datetime) {
                                most_recent_datetime = alarm;
                            }
                        }
                        if most_recent_datetime > (chrono::offset::Utc::now() - Duration::from_secs(5*60)) {
                            //return Message::EventOccured;
                            sender.send(Message::EventOccured).unwrap();
                        }
                    }
                    //info!("{:#?}", json["alarms"][0]["alarmDate"])
                }
            }
        });
        None
    }
}

impl BlaulichtSMSAPI {
    pub fn new(config: Config, sender: Sender<Message>) -> Self {
        Self {
            last_connection: 0,
            config,
            sender
        }
    }
}