use crate::core::context::stats::BotStats;
use crate::core::GuildConfig;
use crate::translation::Translations;
use crate::utils::LogType;
use crate::EncryptionKey;

use aes_gcm::aead::generic_array::GenericArray;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use deadpool_postgres::Pool;
use std::sync::RwLock;
use tokio::sync::mpsc::UnboundedSender;

use twilight::cache::InMemoryCache;
use twilight::gateway::Cluster;
use twilight::http::Client as HttpClient;
use twilight::model::{channel::Message, id::GuildId, user::CurrentUser};

pub struct Context {
    pub cache: InMemoryCache,
    pub cluster: Cluster,
    pub http: HttpClient,
    pub stats: BotStats,
    pub status_type: RwLock<u16>,
    pub status_text: RwLock<String>,
    pub bot_user: CurrentUser,
    configs: DashMap<GuildId, GuildConfig>,
    pub pool: Pool,
    pub translations: Translations,
    __static_master_key: Option<Vec<u8>>,
    log_pumps: DashMap<GuildId, UnboundedSender<(DateTime<Utc>, LogType)>>,
}

impl Context {
    pub fn new(
        cache: InMemoryCache,
        cluster: Cluster,
        http: HttpClient,
        bot_user: CurrentUser,
        pool: Pool,
        translations: Translations,
        key: Option<Vec<u8>>,
    ) -> Self {
        Context {
            cache,
            cluster,
            http,
            stats: BotStats::default(),
            status_type: RwLock::new(3),
            status_text: RwLock::new(String::from("the commands turn")),
            bot_user,
            configs: DashMap::new(),
            pool,
            translations,
            __static_master_key: key,
            log_pumps: DashMap::new(),
        }
    }

    /// Returns if a message was sent by us.
    pub fn is_own(&self, other: &Message) -> bool {
        self.bot_user.id == other.author.id
    }

    fn __get_master_key(&self) -> Option<&EncryptionKey> {
        if let Some(mk_bytes) = &self.__static_master_key {
            let key = GenericArray::from_slice(mk_bytes);
            Some(key)
        } else {
            None
        }
    }
}

mod cache;
mod database;
mod logpump;
mod permissions;
mod stats;
