use crate::{
  BotData,
  BotError,
  GIT_COMMIT_BRANCH,
  GIT_COMMIT_HASH,
  internals::{
    config::BINARY_PROPERTIES,
    monica::Monica,
    threadtimer::ThreadTimer,
    utils::BOT_VERSION
  }
};

use {
  asahi::{
    info,
    spawn
  },
  poise::serenity_prelude::{
    Context,
    GenericChannelId,
    Ready,
    builder::{
      CreateEmbed,
      CreateEmbedAuthor,
      CreateMessage
    }
  },
  std::sync::{
    Arc,
    atomic::{
      AtomicBool,
      Ordering
    }
  }
};

static READY_ONCE: AtomicBool = AtomicBool::new(false);

async fn ready_once(
  ctx: &Context,
  ready: &Ready
) -> Result<(), BotError> {
  #[cfg(not(feature = "production"))]
  {
    info!("Detected a development environment!");
    let gateway = ctx.http.get_bot_gateway().await?;
    let session = gateway.session_start_limit;
    info!("Gateway session limit: {}/{}", session.remaining, session.total);
  }

  info!("Build version: {} ({GIT_COMMIT_HASH}:{GIT_COMMIT_BRANCH})", *BOT_VERSION);
  info!("Connected to API as {}", ready.user.name);

  let ready_embed = CreateEmbed::new()
    .color(BINARY_PROPERTIES.embed_colors.primary())
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
) -> Result<(), BotError> {
  if !READY_ONCE.swap(true, Ordering::Relaxed) {
    ready_once(ctx, ready).await.expect("Failed to call on_ready method");
  }

  let ctx_clone = Arc::new(ctx.clone());
  let bot_data = Arc::clone(&ctx.data::<BotData>());

  spawn(Monica { ctx: Arc::clone(&ctx_clone) }, Arc::clone(&bot_data));
  spawn(ThreadTimer { ctx: Arc::clone(&ctx_clone) }, Arc::clone(&bot_data));

  Ok(())
}
