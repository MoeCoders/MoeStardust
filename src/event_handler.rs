use poise::serenity_prelude::{self as serenity, CreateMessage};
use crate::commands::setu::{get_setu, SetuParams};

use super::{Data,Error};

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }
        serenity::FullEvent::Message { new_message } => {
            if new_message.author.bot {
                return Ok(());
            }
            tracing::info!("{:?}",new_message.content);
            if new_message.content == "来份色图" {
                let embed = get_setu(SetuParams::default()).await?;
                let message = CreateMessage::new().add_embeds(embed);
                new_message.channel_id.send_message(&ctx.http, message)
                    .await?;
            }
        }
        _ => {}
    }
    Ok(())
}