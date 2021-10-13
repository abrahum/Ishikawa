use serde::Deserialize;
use std::{path::PathBuf, str::FromStr};

static FILE_NAME: &str = "ishi.host";
static URLS: [&str; 3] = [
    "https://1.0.0.1/dns-query",
    "https://1.1.1.1/dns-query",
    "https://cloudflare-dns.com/dns-query",
];
static GITHUBS: [&str; 35] = [
    "github.io",
    "github.com",
    "github.dev",
    "github.blog",
    "github.community",
    "api.github.com",
    "gist.github.com",
    "alive.github.com",
    "live.github.com",
    "central.github.com",
    "assets-cdn.github.com",
    "gist.github.com",
    "codeload.github.com",
    "github.githubassets.com",
    "desktop.githubusercontent.com",
    "camo.githubusercontent.com",
    "github.map.fastly.net",
    "github.global.ssl.fastly.net",
    "raw.githubusercontent.com",
    "user-images.githubusercontent.com",
    "favicons.githubusercontent.com",
    "avatars.githubusercontent.com",
    "avatars0.githubusercontent.com",
    "avatars1.githubusercontent.com",
    "avatars2.githubusercontent.com",
    "avatars3.githubusercontent.com",
    "avatars4.githubusercontent.com",
    "avatars5.githubusercontent.com",
    "github-cloud.s3.amazonaws.com",
    "github-com.s3.amazonaws.com",
    "github-production-release-asset-2e65be.s3.amazonaws.com",
    "github-production-user-asset-6210df.s3.amazonaws.com",
    "github-production-repository-file-5c1aeb.s3.amazonaws.com",
    "githubstatus.com",
    "media.githubusercontent.com",
];

#[tokio::main]
async fn main() {
    // check DNS server quality
    let mut rstr = String::default();
    let cli = reqwest::Client::new();
    println!("Quality checking...");
    let index = quality_check(&cli).await;
    println!("Quality check done, geting host from {}", URLS[index]);

    // get hosts single client don't pressure the server
    for url in GITHUBS {
        println!("Getting host for {}", url);
        let mut resp = get_host(&cli, URLS[index], url).await.unwrap();
        let a = resp.answer.remove(0);
        let mut h = a.data;
        while let Err(_) = std::net::Ipv4Addr::from_str(&h) {
            h = resp.answer.remove(0).data;
        }
        rstr = format!("{}{}\n", rstr, format_host(&h, &a.name));
    }

    // save the host data to file
    let file = PathBuf::from(FILE_NAME);
    std::fs::write(&file, rstr).unwrap();
    println!("All hosts get success");
    println!("{} has what you need", FILE_NAME);
}

async fn quality_check(cli: &reqwest::Client) -> usize {
    for i in 0..URLS.len() {
        if let Some(_) = get_host(cli, URLS[i], GITHUBS[0]).await {
            return i;
        }
    }
    panic!("All DNS server unreachable")
}

async fn get_host(cli: &reqwest::Client, dns: &'static str, url: &'static str) -> Option<Resp> {
    let req = cli
        .get(dns)
        .query(&[
            ("ct", "application/dns-json"),
            ("name", url),
            ("type", "A"),
            ("do", "false"),
            ("cd", "false"),
        ])
        .send();
    if let Ok(r) = tokio::time::timeout(tokio::time::Duration::from_secs(2), req).await {
        Some(r.unwrap().json::<Resp>().await.unwrap())
    } else {
        None
    }
}

fn format_host(host: &str, name: &str) -> String {
    let mut r = format!("{}", host);
    for _ in 0..18-host.len() {
        r.push(' ');
    }
    r.push_str(name);
    r
}

#[derive(Deserialize, Debug)]
pub struct Resp {
    #[serde(rename = "Status")]
    pub status: u32,
    #[serde(rename = "Answer")]
    pub answer: Vec<Hosts>,
}

#[derive(Deserialize, Debug)]
pub struct Hosts {
    pub name: String,
    pub data: String,
}
