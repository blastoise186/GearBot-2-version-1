use std::convert::TryFrom;
use std::error;
use std::sync::Arc;

use log::debug;
use tokio::stream::StreamExt;
use twilight::cache::twilight_cache_inmemory::config::{
    EventType as CacheEventType, InMemoryConfigBuilder,
};
use twilight::cache::InMemoryCache;
use twilight::command_parser::{CommandParserConfig, Parser};
use twilight::gateway::cluster::config::ShardScheme;
use twilight::gateway::cluster::Event;
use twilight::gateway::{Cluster, ClusterConfig};
use twilight::http::Client as HttpClient;
use twilight::model::gateway::GatewayIntents;

use crate::core::handlers::{cache, commands::{self, COMMAND_LIST}, general};
use crate::core::{BotConfig, Context};
use crate::{gearbot_error, gearbot_info, Error};

pub struct GearBot;

impl GearBot {
    pub async fn run(
        config: &BotConfig,
        http: HttpClient,
    ) -> Result<(), Box<dyn error::Error + Send + Sync>> {
        // gearbot_info!("GearBot startup initiated!");
        let sharding_scheme = ShardScheme::try_from((0..2, 2)).unwrap();

        let intents = Some(
            GatewayIntents::GUILDS
                | GatewayIntents::GUILD_MEMBERS
                | GatewayIntents::GUILD_BANS
                | GatewayIntents::GUILD_EMOJIS
                | GatewayIntents::GUILD_INVITES
                | GatewayIntents::GUILD_VOICE_STATES
                | GatewayIntents::GUILD_MESSAGES
                | GatewayIntents::GUILD_MESSAGE_REACTIONS
                | GatewayIntents::DIRECT_MESSAGES
                | GatewayIntents::DIRECT_MESSAGE_REACTIONS,
        );

        let cluster_config = ClusterConfig::builder(&config.tokens.discord)
            .shard_scheme(sharding_scheme)
            .intents(intents)
            .build();

        let cache_config = InMemoryConfigBuilder::new()
            .event_types(
                CacheEventType::MESSAGE_CREATE
                    | CacheEventType::MESSAGE_DELETE
                    | CacheEventType::MESSAGE_DELETE_BULK
                    | CacheEventType::MESSAGE_UPDATE,
            )
            .build();

        let cache = InMemoryCache::from(cache_config);

        //TODO: autogen and move to own section
        let cmd_parser = {
            let mut commands_config = CommandParserConfig::new();
            commands_config.add_prefix("?");
            for cmd in &COMMAND_LIST {
                commands_config.command(*cmd).case_insensitive().add()
            }
            Parser::new(commands_config)
        };

        gearbot_info!("The cluster is going online!");
        let cluster = Cluster::new(cluster_config);
        cluster.up().await?;

        let context = Arc::new(Context::new(cmd_parser, cache, cluster, http));

        // TODO: Look into splitting this into two streams:
        // One for user messages, and the other for internal bot things
        let mut bot_events = context.cluster.events().await;

        // context.cluster.command()
        while let Some(event) = bot_events.next().await {
            context.cache.update(&event.1).await?;

            if let Err(e) = tokio::spawn(handle_event(event, context.clone())).await {
                gearbot_error!("{}", e);
                context.stats.had_error().await
            }
        }

        Ok(())
    }
}

async fn handle_event(event: (u64, Event), ctx: Arc<Context<'_>>) -> Result<(), Error> {
    // Process anything that uses the event ID that we care about, aka shard events
    // TODO: Why doesn't this print?
    debug!(
        "Got a {:?} event on shard {}",
        event.1.event_type(),
        event.0
    );
    cache::handle_event(event.0, &event.1, ctx.clone()).await?;
    general::handle_event(event.0, &event.1).await?;
    commands::handle_event(&event.1, ctx.clone()).await?;

    // Since we handled anything with a id we care about, we can make the
    // next match simpler.
    let event = event.1;
    // Bot stat handling "hooks"
    match &event {
        Event::MessageCreate(msg) => ctx.stats.new_message(&ctx, msg).await,
        Event::GuildDelete(_) => ctx.stats.left_guild().await,
        _ => {}
    }

    Ok(())
}
