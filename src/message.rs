use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub enum Message {
    EventOccured(DateTime<Utc>)
}