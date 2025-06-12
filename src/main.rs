mod bridges;
mod commands;
mod controllers;
mod errors;
mod events;
mod internals;
mod shutdown;
// https://cdn.toast-server.net/RustFSHiearachy.png
// Using the new filesystem hierarchy

use {
  bridges::LuaSerenityBridge,
  dag_grpc::MonicaGRPCClient,
  errors::BotError,
  internals::{
    invite_data::InviteCache,
    presence::read_config,
    seasonal::SeasonalTheme,
    utils::{
      discord_token,
      token_path
    }
  }
};

use {
  asahi::{
    error,
    info
  },
  mlua::Lua,
  poise::serenity_prelude::{
    ActivityData,
    ClientBuilder,
    CreateAllowedMentions,
    GatewayIntents,
    Http,
    RoleId
  },
  std::{
    borrow::Cow,
    sync::Arc,
    time::Duration
  }
};

struct BotData {
  redis:           Arc<controllers::cache::RedisController>,
  postgres:        sqlx::PgPool,
  serenity_bridge: Arc<LuaSerenityBridge>,
  invite_data:     Arc<InviteCache>,
  grpc:            MonicaGRPCClient
}

#[cfg(feature = "production")]
pub static GIT_COMMIT_HASH: &str = env!("GIT_COMMIT_HASH");
pub static GIT_COMMIT_BRANCH: &str = env!("GIT_COMMIT_BRANCH");

#[cfg(not(feature = "production"))]
pub static GIT_COMMIT_HASH: &str = "devel";

async fn init_serenity_bridge(
  lua: Arc<Lua>,
  serenity_http: Arc<Http>
) -> Result<LuaSerenityBridge, BotError> {
  let bridge = LuaSerenityBridge::new(lua, serenity_http);
  bridge.register_all()?;
  Ok(bridge)
}

#[tokio::main]
async fn main() {
  asahi::log_init();

  let tconf = read_config();
  let activity = tconf.presence.activities.first().unwrap();

  let postgres = {
    info!("Preparing to connect to database...");
    match sqlx::postgres::PgPoolOptions::new()
      .max_connections(28)
      .max_lifetime(Some(Duration::from_secs(600))) // 10 minutes
      .idle_timeout(Some(Duration::from_secs(360))) // 6 minutes
      .connect(&token_path().await.postgres_uri)
      .await
    {
      Ok(p) => {
        info!("Database connection established");
        p
      },
      Err(e) => {
        error!("Database connection error: {e}");
        std::process::exit(1);
      }
    }
  };

  let grpc = MonicaGRPCClient::default();
  let lua = Arc::new(Lua::new());
  let http = Arc::new(Http::new(discord_token().await));

  let serenity_bridge = Arc::new(
    init_serenity_bridge(Arc::clone(&lua), Arc::clone(&http))
      .await
      .expect("Error initializing LuaSerenityBridge")
  );

  let bot_data = Arc::new(BotData {
    redis: Arc::new(controllers::cache::RedisController::new().await.unwrap()),
    postgres,
    serenity_bridge,
    invite_data: Arc::new(InviteCache::new()),
    grpc
  });

  asahi::spawn(SeasonalTheme, Arc::clone(&bot_data));

  #[cfg(feature = "automod")]
  asahi::spawn(controllers::automod::MaliciousDomains, Arc::clone(&bot_data));

  let prefix = if cfg!(feature = "production") {
    Some(Cow::Borrowed("!!_"))
  } else {
    Some(Cow::Borrowed("."))
  };

  let commands = commands::collect!();
  let framework = poise::Framework::builder()
    .options(poise::FrameworkOptions {
      commands,
      pre_command: |ctx| {
        Box::pin(async move {
          let guild_name: Cow<'_, str> = match ctx.guild() {
            Some(guild) => Cow::Owned(guild.name.clone().into()),
            None => Cow::Borrowed("Unknown Guild")
          };
          let guild_channel_name = match ctx.channel().await {
            Some(channel) => format!("in #{}", channel.guild().unwrap_or_default().base.name),
            None => String::from("")
          };
          let prefix = match ctx.command().prefix_action {
            Some(_) => ctx.framework().options.prefix_options.prefix.as_ref().unwrap(),
            None => "/"
          };

          info!(
            "Discord[{guild_name}] {} ran {prefix}{} {guild_channel_name}",
            ctx.author().name,
            ctx.command().qualified_name
          );
        })
      },
      prefix_options: poise::PrefixFrameworkOptions {
        prefix,
        mention_as_prefix: false,
        case_insensitive_commands: true,
        ignore_bots: true,
        ..Default::default()
      },
      on_error: |error| Box::pin(async move { errors::fw_errors(error).await }),
      allowed_mentions: Some(
        CreateAllowedMentions::default()
          .roles(Cow::Owned(vec![RoleId::new(1155760735612305408)]))
          .empty_users()
      ),
      initialize_owners: true,
      ..Default::default()
    })
    .build();

  let mut client = ClientBuilder::new(
    discord_token().await,
    GatewayIntents::GUILDS
      | GatewayIntents::GUILD_INVITES
      | GatewayIntents::GUILD_MEMBERS
      | GatewayIntents::GUILD_MESSAGES
      | GatewayIntents::GUILD_MODERATION
      | GatewayIntents::MESSAGE_CONTENT
      | GatewayIntents::DIRECT_MESSAGES
  )
  .event_handler(events::DiscordEvents)
  .framework(framework)
  .data(bot_data)
  .activity(ActivityData::streaming(activity.name.clone(), activity.url.clone()).unwrap())
  .await
  .expect("Error creating client");

  let exit_signal = tokio::spawn(async move { shutdown::gracefully_shutdown().await });

  tokio::select! {
    client_result = client.start() => {
      if let Err(why) = client_result {
        error!("Client error: {why:?}");
      }
    },
    shutdown = exit_signal => {
      if shutdown.unwrap() {
        std::process::exit(0);
      }
    }
  }
}
