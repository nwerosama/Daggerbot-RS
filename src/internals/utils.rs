use super::tsclient::TSClient;

use {
  poise::serenity_prelude::{
    Token,
    UserId
  },
  std::{
    str::FromStr,
    sync::LazyLock
  },
  tokenservice_client::TokenServiceApi,
  tokio::sync::Mutex
};

pub static BOT_VERSION: LazyLock<String> = LazyLock::new(|| {
  let cargo_version = cargo_toml::Manifest::from_str(include_str!("../../Cargo.toml"))
    .unwrap()
    .package
    .unwrap()
    .version
    .unwrap();
  format!("v{cargo_version}")
});

static TSCLIENT: LazyLock<Mutex<TSClient>> = LazyLock::new(|| Mutex::new(TSClient::new()));

pub async fn token_path() -> TokenServiceApi { TSCLIENT.lock().await.get().await.unwrap() }

pub async fn discord_token() -> Token { Token::from_str(&token_path().await.main).expect("Serenity couldn't parse the bot token!") }

pub fn mention_dev(ctx: poise::Context<'_, crate::BotData, crate::BotError>) -> Option<String> {
  let devs = super::config::BINARY_PROPERTIES.developers.clone();
  let app_owners = ctx.framework().options().owners.clone();

  let mut mentions = Vec::new();

  for dev in devs {
    if app_owners.contains(&UserId::new(dev)) {
      mentions.push(format!("<@{dev}>"));
    }
  }

  if mentions.is_empty() { None } else { Some(mentions.join(", ")) }
}
