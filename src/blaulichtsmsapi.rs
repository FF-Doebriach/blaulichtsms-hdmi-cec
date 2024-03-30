
use std::time::Duration;


use chrono::prelude::DateTime;
use crossbeam_channel::{Sender};
use generic_runtime::executor::Executor;


use log::{info};
use tokio::runtime::{Handle};



use crate::config::Config;
use crate::message::Message;

#[derive(Debug, Clone)]
pub struct BlaulichtSMSAPI {
    config: Config,
    sender: Sender<Message>,
}

impl Executor<Message> for BlaulichtSMSAPI {
    fn execute(&self) -> Option<Message> {
        let api = self.config.blau_licht_sms_api.clone();
        let sender = self.sender.clone();
        let checking_interval_secs = self.config.checking_interval_secs;
        info!("Checking...");
        Handle::current().spawn(async move {
            let body = reqwest::get(format!("https://api.blaulichtsms.net/blaulicht/api/alarm/v1/dashboard/{}", api)).await;
            if let Ok(body) = body {
                if let Ok(json) = body.json::<serde_json::Value>().await {
                    if let Some(alarms) = json["alarms"].as_array() {
                        let mut most_recent_datetime = chrono::offset::Utc::now() - Duration::from_secs(1 * 60 * 60);
                        //info!("One hour ago: {}", most_recent_datetime);
                        //info!("Current: {}", chrono::offset::Utc::now());
                        for alarm in alarms {
                            let alarm = DateTime::parse_from_rfc3339(alarm["alarmDate"].as_str().expect("Could not parse datetime!")).unwrap().to_utc();
                            if alarm > most_recent_datetime {
                                most_recent_datetime = alarm;
                            }
                        }
                        if most_recent_datetime > (chrono::offset::Utc::now() - Duration::from_secs(5 * checking_interval_secs)) {
                            //return Message::EventOccured;
                            sender.send(Message::EventOccured(most_recent_datetime)).unwrap();
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
            config,
            sender,
        }
    }
}