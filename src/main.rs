use clap::{Parser, Subcommand};
use copypasta::ClipboardProvider;
use rand::Rng;
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};
use url::Url;

#[derive(Debug, Parser)]
pub struct Arguments {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand, EnumIter, EnumString, Display)]
pub enum Command {
    ConfigEspanso,

    #[strum(serialize = "binary-decode")]
    BinaryDecode,

    #[strum(serialize = "binary-encode")]
    BinaryEncode,

    #[strum(serialize = "dedupe")]
    Dedupe,

    #[strum(serialize = "nato-decode")]
    NatoDecode,

    #[strum(serialize = "format-json")]
    FormatJson,

    #[strum(serialize = "ip")]
    Ip,

    #[strum(serialize = "password")]
    Password,

    #[strum(serialize = "reddit-top")]
    RedditTop,

    #[strum(serialize = "sort")]
    Sort,

    #[strum(serialize = "spongebob")]
    Spongebob,

    #[strum(serialize = "timestamp")]
    Timestamp,

    #[strum(serialize = "uuid4")]
    Uuid4,

    #[strum(serialize = "uuid7")]
    Uuid7,
}

fn config_espanso() -> String {
    let exec_path = std::env::current_exe().unwrap();

    Command::iter()
        .filter_map(|item| match item {
            Command::ConfigEspanso => None,
            item => Some(format!(
                "
  - trigger: \";{1}\"
    replace: \"{{{{output}}}}\"
    vars:
      - name: output
        type: script
        params:
          args:
            - {0}
            - {1}
",
                exec_path.display(),
                item
            )),
        })
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn now() -> jiff::Zoned {
    jiff::Timestamp::now()
        .in_tz("UTC")
        .expect("Unable to generate timestamp")
}

pub fn binary_decode(input: &str) -> String {
    input
        .trim()
        .split(' ')
        .map(|chunk| u8::from_str_radix(chunk, 2).unwrap() as char)
        .collect::<_>()
}

pub fn binary_encode(input: &str) -> String {
    input
        .bytes()
        .map(|b| format!("{:0>8}", format!("{b:b}")))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn dedupe(input: &str) -> String {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    for term in input.split(',') {
        let normalized = term
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if !normalized.is_empty() && seen.insert(normalized.clone()) {
            result.push(normalized);
        }
    }
    result.join(", ")
}

pub fn nato_decode(input: &str) -> String {
    input
        .trim()
        .to_lowercase()
        .split_whitespace()
        .map(|word| {
            word.chars()
                .next()
                .unwrap_or_else(|| panic!("Could not get first character of a word: {input}"))
                .to_string()
        })
        .collect()
}

pub fn format_json(input: &str) -> String {
    let json = serde_json::from_str::<serde_json::Value>(input)
        .expect("Unable to parse json, check input");
    serde_json::to_string_pretty(&json).expect("Unable to generate json")
}

pub fn gen_password(input: &str) -> String {
    let length = input.parse().unwrap_or(32);

    rand::rng()
        .sample_iter(&rand::distr::Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub fn reddit_top(input: &str) -> String {
    let url = Url::parse(input).unwrap();
    let origin = url.origin().ascii_serialization();

    let segments: Vec<&str> = url
        .path_segments()
        .map(|c| c.filter(|s| !s.is_empty()).collect())
        .unwrap_or_default();

    if let Some(pos) = segments.iter().position(|&s| s == "u" || s == "user")
        && let Some(&username) = segments.get(pos + 1)
    {
        return format!("{}/u/{}/submitted?sort=top&t=all", origin, username);
    }

    if let Some(pos) = segments.iter().position(|&s| s == "r")
        && let Some(&subreddit) = segments.get(pos + 1)
    {
        return format!("{}/r/{}/top?sort=top&t=all", origin, subreddit);
    }

    input.to_string()
}

pub fn sort_list(input: &str) -> String {
    let mut result = Vec::new();
    for term in input.split(',') {
        let normalized = term
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if !normalized.is_empty() {
            result.push(normalized);
        }
    }
    result.sort();
    result.join(", ")
}

pub fn get_ip_address() -> String {
    ureq::get("https://ipv4.icanhazip.com/")
        .call()
        .expect("unable to request ip address")
        .body_mut()
        .read_to_string()
        .expect("unable to parse body into utf8 string")
}

pub fn spongebob(input: &str) -> String {
    input
        .chars()
        .enumerate()
        .map(|(i, c)| match i.rem_euclid(2) == 0 {
            true => c.to_uppercase().to_string(),
            false => c.to_lowercase().to_string(),
        })
        .collect()
}

pub fn get_iso_timestamp() -> String {
    now().timestamp().to_string()
}

pub fn gen_uuid4() -> String {
    uuid::Uuid::new_v4().as_hyphenated().to_string()
}

pub fn gen_uuid7() -> String {
    let timestamp = now().timestamp();

    uuid::Uuid::new_v7(uuid::Timestamp::from_unix(
        uuid::NoContext,
        timestamp.as_second() as _,
        timestamp.subsec_nanosecond() as _,
    ))
    .as_hyphenated()
    .to_string()
}

pub fn main() {
    let args = Arguments::parse();

    let mut clipboard =
        copypasta::ClipboardContext::new().expect("Unable to build clipboard context");

    let input = clipboard
        .get_contents()
        .expect("Unable to get contents of clipboard");

    let result = match &args.command {
        Command::ConfigEspanso => config_espanso(),
        Command::BinaryDecode => binary_decode(&input),
        Command::BinaryEncode => binary_encode(&input),
        Command::Dedupe => dedupe(&input),
        Command::NatoDecode => nato_decode(&input),
        Command::FormatJson => format_json(&input),
        Command::Ip => get_ip_address(),
        Command::Password => gen_password(&input),
        Command::RedditTop => reddit_top(&input),
        Command::Sort => sort_list(&input),
        Command::Spongebob => spongebob(&input),
        Command::Timestamp => get_iso_timestamp(),
        Command::Uuid4 => gen_uuid4(),
        Command::Uuid7 => gen_uuid7(),
    };

    print! {"{}", result.trim()};
    clipboard
        .set_contents(result)
        .expect("Unable to set contents of clipboard")
}
