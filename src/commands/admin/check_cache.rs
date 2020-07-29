use crate::core::CommandContext;
use crate::parser::Parser;
use crate::CommandResult;
use log::info;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use twilight::builders::embed::EmbedBuilder;
use twilight::model::id::{GuildId, UserId};

pub async fn check_cache(ctx: CommandContext, _: Parser) -> CommandResult {
    if ctx.message.author.id.0 == 106354106196570112 {
        let mut counts: HashMap<UserId, Vec<GuildId>> = HashMap::new();
        for guild in ctx
            .bot_context
            .cache
            .guilds
            .read()
            .expect("Global guild cache got poisoned!")
            .values()
        {
            for member in guild
                .members
                .read()
                .expect("Guild inner members cache got poisoned!")
                .values()
            {
                let mut list = match counts.get(&member.user_id) {
                    Some(list) => list.clone(),
                    None => vec![],
                };
                list.push(guild.id);
                counts.insert(member.user_id, list);
            }
        }

        let mut out = String::from("");
        let mut think_no_servers = 0;
        let mut no_servers = 0;

        for user in ctx
            .bot_context
            .cache
            .users
            .read()
            .expect("Global users cache got corrupted!")
            .values()
        {
            let tracked = user.mutual_servers.load(Ordering::SeqCst) as usize;
            let empty = vec![];
            let real = counts.get(&user.id).unwrap_or(&empty);
            if tracked == 0 {
                think_no_servers += 1;
            }
            if real.is_empty() {
                no_servers += 1;
            }
            if tracked != real.len() {
                out += &format!(
                    "\n {} is in {} mutual but thinks they are in {})",
                    user.id,
                    real.len(),
                    tracked.to_string() /*,
                                        real.iter()
                                            .map(|id| id.0.to_string())
                                            .collect::<Vec<String>>()
                                            .join(", "),
                                        user.mutual_servers
                                            .read()
                                            .expect("User mutuals server list got poisoned")
                                            .iter()
                                            .map(|id| id.0.to_string())
                                            .collect::<Vec<String>>()
                                            .join(", ")*/
                );
            }
        }
        if out.is_empty() {
            out = String::from("All user mutual counts are correct")
        }
        if out.len() > 2000 {
            info!("{}", out);
            out = String::from("Too long, see console");
        }
        let e = EmbedBuilder::new()
            .add_field(
                "Unique users metric",
                ctx.bot_context.stats.user_counts.unique.get().to_string(),
            )
            .commit()
            .add_field(
                "Unique users in cache",
                ctx.bot_context
                    .cache
                    .users
                    .read()
                    .expect("Global users cache got corrupted!")
                    .len()
                    .to_string(),
            )
            .commit()
            .add_field(
                "Total users metric",
                ctx.bot_context.stats.user_counts.total.get().to_string(),
            )
            .commit()
            .add_field(
                "Total users in cache",
                ctx.bot_context
                    .cache
                    .guilds
                    .read()
                    .expect("Global guilds cache got corrupted!")
                    .values()
                    .map(|guild| {
                        guild
                            .members
                            .read()
                            .expect("Guild inner members cache got poisoned!")
                            .len()
                    })
                    .sum::<usize>()
                    .to_string(),
            )
            .commit()
            .add_field(
                "Members without properly cached users",
                ctx.bot_context
                    .cache
                    .guilds
                    .read()
                    .expect("Global guild cache got poisoned!")
                    .values()
                    .map(|guild| {
                        guild
                            .members
                            .read()
                            .expect("Guild inner members cache got poisoned!")
                            .values()
                            .map(|member| {
                                if ctx
                                    .bot_context
                                    .cache
                                    .users
                                    .read()
                                    .expect("Global users cache got poisoned!")
                                    .contains_key(&member.user_id)
                                {
                                    0
                                } else {
                                    1
                                }
                            })
                            .sum::<usize>()
                    })
                    .sum::<usize>()
                    .to_string(),
            )
            .commit()
            .add_field("Users who think they have no mutuals", think_no_servers.to_string())
            .commit()
            .add_field("Users without mutual servers", no_servers.to_string())
            .commit()
            .build();
        ctx.reply_raw_with_embed(out, e).await?;
    }
    Ok(())
}
