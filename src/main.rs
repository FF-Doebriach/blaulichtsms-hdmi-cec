use std::fs::File;
use std::io::BufReader;
use std::process::Command;
use std::time::Duration;
use clap::Parser;
use clap_derive::Parser;
use crossbeam_channel::unbounded;
use generic_runtime::handler::Handler;
use generic_runtime::module_runner::ModuleRunner;
use log::error;
use crate::blaulichtsmsapi::BlaulichtSMSAPI;
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
    tracing_subscriber::fmt::init(); // enable logging of async
    let (sender, receiver) = unbounded::<Message>();
    let mut handler: Handler<Message> = Handler::new::<>();
    let api_runner = BlaulichtSMSAPI::new(serde_json::from_reader(config_reader).unwrap(), sender.clone());
    let runner = ModuleRunner::new(Box::new(api_runner), Duration::from_secs(1*60), sender.clone());
    handler.spawn(0, runner);
    loop {
        receiver.try_iter().for_each(|message| {
            match message {
                Message::EventOccured => {
                    let cmd = Command::new("sh")
                        .arg("-c")
                        .arg("echo 'on 0.0.0.0' | cec-client -s -d 1")
                        .output();
                    if cmd.is_err() {
                        error!("Could not use cec-client!")
                    }
                }
            }
        })

    }
}