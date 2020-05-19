use serde::{Deserialize, Serialize};
use unic_langid::LanguageIdentifier;

use crate::translation::DEFAULT_LANG;

#[derive(Deserialize, Serialize, Debug)]
pub struct GuildConfig {
    pub prefix: String,
    pub log_style: LogStyle,
    pub message_logs: MessageLogs,
    pub language: LanguageIdentifier,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MessageLogs {
    pub enabled: bool,
    pub ignored_users: Vec<u64>,
    pub ignored_channels: Vec<u64>,
    pub ignore_bots: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum LogStyle {
    Text,
    Embed,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LogChannelConfig {}

#[derive(Deserialize, Serialize, Debug)]
pub enum LogCategories {}

#[derive(Deserialize, Serialize, Debug)]
pub enum LogSubCategory {}

impl Default for GuildConfig {
    fn default() -> Self {
        GuildConfig {
            prefix: "!".to_string(),
            log_style: LogStyle::Text,
            message_logs: MessageLogs {
                enabled: false,
                ignored_users: vec![],
                ignored_channels: vec![],
                ignore_bots: true,
            },
            language: DEFAULT_LANG,
        }
    }
}
