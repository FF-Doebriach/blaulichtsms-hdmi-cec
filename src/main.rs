use std::fs::File;
use std::io::BufReader;
use std::process::Command;
use tokio::time::{sleep, Duration};
use clap::Parser;
use clap_derive::Parser;
use crossbeam_channel::unbounded;
use generic_runtime::handler::Handler;
use generic_runtime::module_runner::ModuleRunner;
use log::{error, info};
use crate::blaulichtsmsapi::BlaulichtSMSAPI;
use crate::config::Config;
use crate::message::Message;

mod blaulichtsmsapi;
mod message;
mod config;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    path: String,
    #[arg(short, long)]
    debug: bool
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config_file = File::open(args.path).expect("Config file not found!");
    let config_reader = BufReader::new(config_file);
    let config: Config = serde_json::from_reader(config_reader).unwrap();
    tracing_subscriber::fmt::init(); // enable logging of async
    let (sender, receiver) = unbounded::<Message>();
    let mut handler: Handler<Message> = Handler::new::<>();
    let api_runner = BlaulichtSMSAPI::new(config.clone(), sender.clone());
    let runner = ModuleRunner::new(Box::new(api_runner), Duration::from_secs(config.checking_interval_secs), sender.clone());
    handler.spawn(0, runner);
    let mut turned_on = false;
    let mut last_alarm = chrono::offset::Utc::now();
    loop {
        receiver.try_iter().for_each(|message| {
            match message {
                Message::EventOccured(occured_at) => {
                    if occured_at != last_alarm {
                        if config.use_hdmi_cec {
                            let cmd = Command::new("sh")
                                .arg("-c")
                                .arg("echo 'on 0.0.0.0' | cec-client -s -d 1")
                                .output();
                            if cmd.is_err() {
                                error!("Could not use cec-client!")
                            } else {
                                info!("Turned on TV...")
                            }
                            turned_on = true;
                            last_alarm = occured_at;
                        }
                    }
                }
            }
        });
        if last_alarm + Duration::from_secs(config.turn_off_interval_secs) < chrono::offset::Utc::now() && turned_on && config.turn_off_interval_enabled {
            turned_on = false;
            if config.use_hdmi_cec {
                let cmd = Command::new("sh")
                    .arg("-c")
                    .arg("echo 'standby 0.0.0.0' | cec-client -s -d 1 #turn in standby")
                    .output();
                if cmd.is_err() {
                    error!("Could not use cec-client!")
                } else {
                    info!("Turned off tv...")
                }
            }
        }
        sleep(Duration::from_secs(5)).await;
    }
}
