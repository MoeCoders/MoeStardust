use poise::serenity_prelude as serenity;
use tracing::info;
use poise::structs::PrefixFrameworkOptions;
use dotenv::dotenv;
use poise::serenity_prelude::prelude::GatewayIntents;
mod event_handler;
use event_handler::event_handler;

mod commands;
pub struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv().unwrap();
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![commands::setu::setu()],
            prefix_options: PrefixFrameworkOptions {
                prefix: Some("bot".to_string()),
                ..PrefixFrameworkOptions::default()
            },
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        //.event_handler(Handler {})
        .await;
    info!("机器人启动！");
    client.unwrap().start().await.unwrap();
}
