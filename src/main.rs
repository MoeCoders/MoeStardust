use std::env;

mod modules;
mod utils;

use dotenv;
use serenity::{
    async_trait,
    framework::standard::{macros::group, StandardFramework},
    model::{
        application::{
            command::Command,
            interaction::{Interaction, InteractionResponseType},
        },
        channel::Message,
        prelude::{Ready, ResumedEvent},
    },
    prelude::*,
};

#[group]
struct General;

struct Handler {}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        if msg.author.id == 1006899111020204082 && msg.content == "删除命令" {
            match Command::get_global_application_commands(&ctx.http).await {
                Ok(commands) => {
                    for command in commands {
                        println!("{}", command.name);
                        match Command::delete_global_application_command(&ctx.http, command.id)
                            .await
                        {
                            Ok(_) => {
                                if let Err(e) = msg.reply(&ctx, "删除命令成功").await {
                                    eprintln!("发送消息错误 {}", e);
                                }
                            }
                            Err(error) => eprintln!("无法删除全局Slash命令：{:?}", error),
                        }
                    }
                }
                Err(error) => eprintln!("无法获取全局Slash命令列表: {:?}", error),
            }
            // 获取 bot 加入的所有服务器的 ID 列表
            let guild_ids = ctx.cache.guilds();

            // 删除每个服务器中的 Slash 命令
            for guild_id in guild_ids {
                match ctx.http.get_guild_application_commands(guild_id.0).await {
                    Ok(commands) => {
                        for command in commands {
                            // 删除 Slash 命令
                            match ctx
                                .http
                                .delete_guild_application_command(guild_id.0, command.id.0)
                                .await
                            {
                                Ok(_) => println!(
                                    "Successfully deleted command {} in guild {}",
                                    command.name, guild_id.0
                                ),
                                Err(e) => println!(
                                    "Failed to delete command {} in guild {}: {:?}",
                                    command.name, guild_id.0, e
                                ),
                            }
                        }
                    }
                    Err(e) => {
                        println!("Failed to get commands in guild {}: {:?}", guild_id.0, e);
                    }
                }
            }
        }

        if msg.content == "我要色图" {
            if let Err(e) = modules::setu::get_setu(ctx, msg).await {
                println!("发送消息错误：{}", e);
            };
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            match command.data.name.as_str() {
                "get_setu" => {
                    if let Err(e) = modules::setu::command::run(&ctx, &command).await {
                        if let Err(why) = command
                            .create_interaction_response(&ctx.http, |response| {
                                response
                                    .kind(InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|message| {
                                        message.content(format!("获取色图失败(▼皿▼#) ：{}", e))
                                    })
                            })
                            .await
                        {
                            println!("发送消息失败: {}", why);
                        }
                    }
                }
                "img2color" => {
                    if let Err(why) = modules::img2color::run(&ctx, &command).await {
                        if let Err(why) = command
                            .create_interaction_response(&ctx.http, |response| {
                                response
                                    .kind(InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|message| {
                                        message.content(format!("获取主题色失败(▼皿▼#) ：{}", why))
                                    })
                            })
                            .await
                        {
                            println!("发送消息失败: {}", why);
                        }
                    }
                }
                _ => {
                    if let Err(why) = command
                        .create_interaction_response(&ctx.http, |response| {
                            response
                                .kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|message| {
                                    message.content("获取slash失败(怒｀Д´怒)")
                                })
                        })
                        .await
                    {
                        println!("发送消息失败 : {}", why);
                    }
                }
            };
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} 上线了！", ready.user.name);
        let _ = Command::set_global_application_commands(&ctx.http, |commands| {
            commands
                .create_application_command(|command| modules::setu::command::register(command))
                .create_application_command(|command| modules::img2color::register(command))
        })
        .await;
    }

    async fn resume(&self, c: Context, _: ResumedEvent) {
        println!("{} 已重连", c.cache.current_user().name);
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let framework = StandardFramework::new().group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler {})
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
