use crate::{
  Error,
  GIT_COMMIT_BRANCH,
  GIT_COMMIT_HASH,
  internals::{
    config::BINARY_PROPERTIES,
    tasks,
    utils::BOT_VERSION
  }
};

use {
  poise::serenity_prelude::{
    Context,
    GenericChannelId,
    Ready,
    builder::{
      CreateEmbed,
      CreateEmbedAuthor,
      CreateMessage
    },
    gateway::ActivityData
  },
  serde::{
    Deserialize,
    Serialize
  },
  std::{
    fs,
    sync::{
      Arc,
      atomic::{
        AtomicBool,
        Ordering
      }
    },
    thread::current
  }
};

/// The static path to the TOML config file for bot's presence data
pub const TOML_FILE: &str = if cfg!(feature = "production") {
  "presence.toml"
} else {
  "src/internals/assets/presence.toml"
};

static READY_ONCE: AtomicBool = AtomicBool::new(false);

#[derive(Serialize, Deserialize)]
pub struct Activity {
  pub name: String,
  pub url:  String
}

#[derive(Serialize, Deserialize)]
pub struct Presence {
  pub activities: Vec<Activity>
}

#[derive(Serialize, Deserialize)]
pub struct TomlConfig {
  pub presence: Presence
}

fn read_config() -> TomlConfig {
  let content = fs::read_to_string(TOML_FILE).expect("[TomlConfig] Error loading config file");
  let config: TomlConfig = toml::from_str(&content).expect("[TomlConfig] Error parsing config file");
  config
}

async fn ready_once(
  ctx: &Context,
  ready: &Ready
) -> Result<(), Error> {
  #[cfg(not(feature = "production"))]
  {
    println!("Event[Ready:Notice] Detected a development environment!");
    let gateway = ctx.http.get_bot_gateway().await?;
    let session = gateway.session_start_limit;
    println!("Event[Ready:Notice] Session limit: {}/{}", session.remaining, session.total);
  }

  println!("Event[Ready] Build version: {} ({GIT_COMMIT_HASH}:{GIT_COMMIT_BRANCH})", *BOT_VERSION);
  println!("Event[Ready] Connected to API as {}", ready.user.name);

  let ready_embed = CreateEmbed::new()
    .color(BINARY_PROPERTIES.embed_colors.primary)
    .thumbnail(ready.user.avatar_url().unwrap_or_default())
    .author(CreateEmbedAuthor::new(format!("{} is ready!", ready.user.name)).clone());

  GenericChannelId::new(BINARY_PROPERTIES.ready_notify)
    .send_message(&ctx.http, CreateMessage::new().add_embed(ready_embed))
    .await?;

  Ok(())
}

pub async fn on_ready(
  ctx: &Context,
  ready: &Ready
) -> Result<(), Error> {
  if !READY_ONCE.swap(true, Ordering::Relaxed) {
    ready_once(ctx, ready).await.expect("Failed to call on_ready method");
  }

  let thread_id = format!("{:?}", current().id());
  let thread_num: String = thread_id.chars().filter(|c| c.is_ascii_digit()).collect();
  println!("Event[Ready] Task Scheduler launched on thread {thread_num}");

  let tconf = read_config();
  let activity = tconf.presence.activities.first().unwrap();

  ctx.set_activity(Some(ActivityData::streaming(activity.name.clone(), activity.url.clone()).unwrap()));

  tasks::run_task(Arc::new(ctx.clone()), tasks::monica, "Monica").await;

  Ok(())
}
