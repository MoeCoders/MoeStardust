use std::io;

use super::super::utils::img;

use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::prelude::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOptionValue,
};
use serenity::prelude::Context;
use serenity::Error as SerenityError;

pub async fn run(
    ctx: &Context,
    cmd: &ApplicationCommandInteraction,
) -> Result<(), serenity::Error> {
    let mut url: Option<&str> = None;
    let options = &cmd.data.options;
    for option in options {
        match option.name.as_str() {
            "img" => {
                url = match &option.resolved {
                    Some(o) => match o {
                        CommandDataOptionValue::Attachment(a) => match &a.content_type {
                            Some(t) => {
                                if t.starts_with("image/") {
                                    Some(&a.url)
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        },
                        _ => None,
                    },
                    _ => None,
                }
            }
            _ => {
                return Err(SerenityError::from(io::Error::new(
                    io::ErrorKind::Other,
                    "传入参数错误",
                )));
            }
        }
    }
    let color: String;
    match url {
        Some(s) => {
            color = match img::color::get_theme_color(s, true).await {
                Ok(v) => format!("图片的主题色是 {}", v),
                Err(e) => e.to_string(),
            };
        }
        None => {
            return Err(SerenityError::from(io::Error::new(
                io::ErrorKind::Other,
                "读取url错误",
            )))
        }
    }
    let resp = cmd.create_interaction_response(&ctx.http, |r| {
        r.kind(serenity::model::prelude::InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|f| f.content(color))
    });
    if let Err(why) = resp.await {
        return Err(why);
    }
    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("img2color")
        .description("获取图片主题色")
        .create_option(|option| {
            option
                .name("img")
                .description("需要提取主题色的图片")
                .kind(CommandOptionType::Attachment)
                .required(true)
        })
}
