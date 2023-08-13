use std::fmt::Write;

use reqwest;
use serde::{Deserialize, Serialize};
use serenity::builder::CreateEmbed;
use serenity::model::prelude::Message;
use serenity::prelude::Context;
use serenity::utils::Color;
use serenity::Error as SerenityError;
#[derive(Serialize, Deserialize)]
struct Data {
    pid: i64,
    p: i64,
    uid: i64,
    title: String,
    author: String,
    r18: bool,
    width: i64,
    height: i64,
    tags: Vec<String>,
    ext: String,
    aiType: i64,
    uploadDate: i64,
    urls: Urls,
}

#[derive(Serialize, Deserialize)]
struct Setu {
    error: String,
    data: Vec<Data>,
}

#[derive(Serialize, Deserialize)]
struct Urls {
    original: String,
}

pub mod command {
    use super::get_setu_json;
    use super::SerenityError;
    use super::Setu;
    use super::SetuParams;
    use serenity::builder::CreateApplicationCommand;
    use serenity::builder::CreateEmbed;
    use serenity::model::application::interaction::InteractionResponseType;
    use serenity::model::prelude::command::CommandOptionType;
    use serenity::model::prelude::prelude::interaction::application_command::ApplicationCommandInteraction;
    use serenity::prelude::Context;
    use serenity::utils::Color;

    pub async fn run(
        ctx: &Context,
        cmd: &ApplicationCommandInteraction,
    ) -> Result<(), serenity::Error> {
        let mut params = SetuParams::default();
        let options = &cmd.data.options;
        for option in options {
            match option.name.as_str() {
                "r18" => {
                    params.r18 = match &option.value {
                        Some(s) => s.as_i64(),
                        _ => None,
                    }
                }
                "keyword" => {
                    params.keyword = match &option.value {
                        Some(k) => match k {
                            serenity::json::Value::String(v) => Some(v.clone()),
                            _ => None,
                        },
                        _ => None,
                    }
                }
                "tags" => {
                    params.tags = match &option.value {
                        Some(t) => match t {
                            serde_json::Value::String(s) => {
                                s.split(' ').map(|s| s.parse::<String>().ok()).collect()
                            }
                            _ => vec![],
                        },
                        None => vec![],
                    }
                }
                "num" => {
                    params.num = match &option.value {
                        Some(n) => match n {
                            serde_json::Value::Number(i) => match i.as_u64() {
                                Some(i) => {
                                    if i > 10 {
                                        Some(10)
                                    } else {
                                        Some(i)
                                    }
                                }
                                _ => Some(1),
                            },
                            _ => None,
                        },
                        _ => None,
                    }
                }
                "uids" => {
                    params.uids = match &option.value {
                        Some(u) => match u {
                            serde_json::Value::String(s) => {
                                s.split(" ").map(|b| b.parse::<i64>().ok()).collect()
                            }
                            _ => vec![],
                        },
                        None => vec![],
                    }
                }
                _ => (),
            }
        }
        let data: Setu;
        match get_setu_json(params).await {
            Ok(v) => data = v,
            Err(e) => {
                println!("请求api错误，{}", e.to_string());
                return Err(SerenityError::from(e));
            }
        };
        let mut embed: Vec<CreateEmbed> = vec![];
        for v in data.data.iter() {
            let tags = format!("{:?}", &v.tags);
            embed.push(
                CreateEmbed::default()
                    .author(|a| {
                        a.name(&v.author)
                            .url(format!("https://www.pixiv.net/users/{}", v.uid))
                            .icon_url("https://i.imgur.com/pECIFHB.png")
                    })
                    .title(&v.title)
                    .url(format!("https://www.pixiv.net/artworks/{}", v.pid))
                    .image(&v.urls.original)
                    .color(Color::DARK_BLUE)
                    .field("R18", format!("{}", &v.r18), false)
                    .field("Size", format!("{}x{}", &v.height, &v.width), false)
                    .field(
                        "Tags",
                        tags.chars()
                            .skip(1)
                            .take(tags.len() - 3)
                            .collect::<String>(),
                        false,
                    )
                    .footer(|f| f.text("Powerd by api.lolicon.app"))
                    .clone(),
            );
        }

        let response = cmd.create_interaction_response(&ctx.http, |r| {
            r.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|d| d.add_embeds(embed))
        });

        if let Err(why) = response.await {
            return Err(why);
        }
        Ok(())
    }

    pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name("get_setu")
            .description("获取色图")
            .create_option(|option| {
                option
                    .name("r18")
                    .description("是否R18")
                    .kind(CommandOptionType::Integer)
                    .add_int_choice("是😍", 1)
                    .add_int_choice("不是", 0)
                    .add_int_choice("随机", 2)
                    .required(false)
            })
            .create_option(|option| {
                option
                    .name("keyword")
                    .description("关键字")
                    .kind(CommandOptionType::String)
                    .required(false)
            })
            .create_option(|option| {
                option
                    .name("tags")
                    .description("标签（多个请用空格分隔）")
                    .kind(CommandOptionType::String)
                    .required(false)
            })
            .create_option(|option| {
                option
                    .name("num")
                    .description("数量，最多10")
                    .kind(CommandOptionType::Integer)
                    .required(false)
            })
            .create_option(|option| {
                option
                    .name("uids")
                    .description("指定画师uid（多个用空格分隔）")
                    .kind(CommandOptionType::String)
                    .required(false)
            })
    }
}

pub async fn get_setu(ctx: Context, msg: Message) -> Result<(), SerenityError> {
    let data: Setu;
    match get_setu_json(SetuParams {
        num: Some(1),
        ..Default::default()
    })
    .await
    {
        Ok(v) => data = v,
        Err(e) => {
            println!("请求api错误，{}", e.to_string());
            return Err(SerenityError::from(e));
        }
    };

    for v in data.data.iter() {
        let mut embed = CreateEmbed::default();
        embed
            .author(|a| {
                a.name(&v.author)
                    .url(format!("https://www.pixiv.net/users/{}", v.uid))
                    .icon_url("https://i.imgur.com/pECIFHB.png")
            })
            .title(&v.title)
            .url(format!("https://www.pixiv.net/artworks/{}", v.pid))
            .image(&v.urls.original)
            .color(Color::DARK_BLUE)
            .field("R18", format!("{}", &v.r18), false)
            .field("Size", format!("{}x{}", &v.height, &v.width), false)
            .field("Tags", format!("{:?}", &v.tags), false)
            .footer(|f| f.text("Powerd by api.lolicon.app"));

        let reply = msg.channel_id.send_message(&ctx.http, |m| {
            m.reference_message(&msg);
            m.embed(|e| {
                e.clone_from(&embed);
                e
            })
        });
        // 如果发送回复消息失败，则打印错误信息
        if let Err(why) = reply.await {
            println!("Failed to send reply: {:?}", why);
            return Err(why);
        }
    }
    Ok(())
}

macro_rules! append_opt {
    ($url:expr, $key:expr, $val:expr) => {{
        if let Some(v) = $val {
            if $url.ends_with('?') {
                write!($url, "{}={}", $key, v).unwrap();
            } else {
                write!($url, "&{}={}", $key, v).unwrap();
            }
        }
    }};
}

struct SetuParams {
    r18: Option<i64>,
    keyword: Option<String>,
    tags: Vec<Option<String>>,
    num: Option<u64>,
    uids: Vec<Option<i64>>,
    proxy: Option<String>,
    excludeAI: Option<bool>,
}

impl SetuParams {
    async fn to_url(&self) -> String {
        let mut url = String::from("https://api.lolicon.app/setu/v2?");
        append_opt!(url, "r18", &self.r18);
        append_opt!(url, "keyword", &self.keyword);
        for tag in &self.tags {
            append_opt!(url, "tag", tag);
        }
        append_opt!(url, "num", &self.num);
        for uid in &self.uids {
            append_opt!(url, "uid", uid);
        }
        append_opt!(url, "proxy", &self.proxy);
        append_opt!(url, "excludeAI", &self.excludeAI);

        url
    }
}

impl Default for SetuParams {
    fn default() -> Self {
        SetuParams {
            r18: Some(0),
            keyword: None,
            tags: vec![None],
            uids: vec![None],
            excludeAI: Some(true),
            num: None,
            proxy: Some("pixiv.yxlr.link".to_string()),
        }
    }
}

async fn get_setu_json(params: SetuParams) -> Result<Setu, std::io::Error> {
    let url = params.to_url().await;
    let resp = reqwest::get(url)
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    if resp.status().is_success() {
        let data: Setu = resp
            .json()
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        if data.error != "" {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("api返回了错误 {}", data.error),
            ))
        } else {
            Ok(data)
        }
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("api返回了：{}", resp.status()),
        ))
    }
}
