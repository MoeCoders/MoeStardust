use poise::serenity_prelude as serenity;
use regex::Regex;
use serde::{Deserialize, Serialize};
//use tracing::info;

use super::Context;
use super::Error;

#[allow(non_snake_case)]
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

#[allow(non_snake_case)]
pub struct SetuParams {
    r18: u8,
    keyword: Option<String>,
    tags: Option<Vec<String>>,
    num: Option<u8>,
    uids: Option<Vec<u64>>,
    proxy: String,
    excludeAI: bool,
}

impl Default for SetuParams {
    fn default() -> Self {
        SetuParams {
            r18: 0,
            keyword: None,
            tags: None,
            uids: None,
            excludeAI: false,
            num: None,
            proxy: "pixiv.yxlr.link".to_string(),
        }
    }
}

impl SetuParams {
    async fn to_url(&self) -> String {
        let mut url = format!(
            "https://api.lolicon.app/setu/v2?r18={}&proxy={}&excludeAI={}",
            self.r18, self.proxy, self.excludeAI
        );
        if let Some(k) = &self.keyword {
            url = format!("{}&keyword={}", url, k);
        }
        if let Some(tags) = &self.tags {
            for t in tags.iter() {
                url = format!("{}&tag={}", url, t);
            }
        }
        if let Some(uids) = &self.uids {
            for u in uids.iter() {
                url = format!("{}&uid={}", url, u);
            }
        }
        if let Some(n) = &self.num {
            url = format!("{}&num={}", url, n);
        }
        url
    }
}

#[derive(poise::ChoiceParameter)]
pub enum R18type {
    #[name = "R18！"]
    True,
    #[name = "全年龄"]
    False,
    #[name = "随机！！"]
    Rand,
}

/// 获取色图
#[poise::command(slash_command, prefix_command)]
pub async fn setu(
    ctx: Context<'_>,
    #[description = "是否r18"] r18: Option<R18type>,
    #[description = "关键字"] keyword: Option<String>,
    #[description = "标签（多个请用空格分隔）"] tags: Option<String>,
    #[description = "数量，最多10"]
    #[min = 1]
    #[max = 10]
    num: Option<u8>,
    #[description = "指定画师uid（多个用空格分隔）"] uids: Option<String>,
) -> Result<(), Error> {
    if let Some(u) = &uids {
        let uids_pattern = r"^(\d+\s)*\d+$";

        // Create a Regex object with the pattern
        let re = Regex::new(uids_pattern).unwrap();

        // Check if the input string matches the pattern
        if !re.is_match(u.as_str()) {
            ctx.say("Uid参数格式不正确！").await?;
            return Ok(());
        }
    }
    let r18 = match r18 {
        Some(r) => match r {
            R18type::True => 1,
            R18type::False => 0,
            R18type::Rand => 2,
        },
        _ => 0,
    };
    let mut params = SetuParams {
        num: num,
        r18: r18,
        keyword: keyword,
        ..SetuParams::default()
    };
    params.tags = match tags {
        Some(t) => Some(t.split_whitespace().map(|t| t.to_string()).collect()),
        None => None,
    };
    params.uids = match uids {
        Some(u) => Some(u.split_whitespace().map(|u| u.parse().unwrap()).collect()),
        _ => None,
    };
    let embeds = get_setu(params).await?;
    // let b = format!("{:?}",embeds);
    // info!(b);
    let mut replay = poise::CreateReply::default();
    replay.embeds = embeds;
    ctx.send(replay).await?;
    Ok(())
}

pub async fn get_setu(
    params: SetuParams,
) -> Result<Vec<serenity::CreateEmbed>, Box<dyn std::error::Error + Send + Sync>> {
    let url = params.to_url().await;
    let resp = reqwest::get(&url).await?.json::<Setu>().await?;
    let mut embeds: Vec<poise::serenity_prelude::CreateEmbed> = vec![];
    if resp.error != "" {
        return Err(format!("api返回了错误：{}", resp.error).into());
    }
    if resp.data.len() == 0 {
        return Err("Api返回了空值".into());
    }
    for data in resp.data.iter() {
        let tags = format!("{:?}", &data.tags);

        let author = serenity::CreateEmbedAuthor::new(&data.author)
            .url(format!("https://www.pixiv.net/users/{}", &data.uid))
            .icon_url("https://i.imgur.com/pECIFHB.png");
        let footer = serenity::CreateEmbedFooter::new("Powerd by api.lolicon.app");
        let embed = serenity::CreateEmbed::default()
            .author(author)
            .title(&data.title)
            .url(format!("https://www.pixiv.net/artworks/{}", &data.pid))
            .image(&data.urls.original)
            .color(serenity::model::Color::DARK_BLUE)
            .field("R18", format!("{}", &data.r18), false)
            .field("Size", format!("{}x{}", &data.height, &data.width), false)
            .field(
                "Tags",
                tags.chars()
                    .skip(1)
                    .take(tags.len() - 4)
                    .collect::<String>(),
                false,
            )
            .footer(footer);
        embeds.push(embed);
    }
    Ok(embeds)
}
