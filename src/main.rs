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
  asahi::{
    Probe,
    error,
    info,
    utils::database::{
      AsahiDatabaseConfig,
      AsahiDatabaseKind,
      connect,
      prepare_tables
    }
  },
  bridges::LuaSerenityBridge,
  dag_grpc::MonicaClient,
  errors::{
    BotError,
    BotResult
  },
  internals::{
    invite_data::InviteCache,
    presence::read_config,
    seasonal::SeasonalTheme,
    utils::{
      discord_token,
      token_path
    }
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
    sync::Arc
  }
};

struct BotData {
  redis:           Arc<controllers::cache::RedisController>,
  postgres:        sqlx::PgPool,
  serenity_bridge: Arc<LuaSerenityBridge>,
  invite_data:     Arc<InviteCache>,
  grpc:            MonicaClient
}

struct Database(String);

impl AsahiDatabaseConfig for Database {
  fn uri(&self) -> &str { &self.0 }

  fn app_name(&self) -> &str { "Daggerbot" }

  fn kind(&self) -> AsahiDatabaseKind { AsahiDatabaseKind::Postgres }

  fn max_connections(&self) -> u32 { 26 }
}

async fn init_serenity_bridge(
  lua: Arc<Lua>,
  serenity_http: Arc<Http>
) -> BotResult<LuaSerenityBridge> {
  let bridge = LuaSerenityBridge::new(lua, serenity_http);
  bridge.register_all()?;
  Ok(bridge)
}

#[tokio::main]
async fn main() {
  asahi::log_init();

  let health_probe = Arc::new(Probe::new());
  health_probe.spawn_server(9000);

  let tconf = read_config();
  let activity = tconf.presence.activities.first().unwrap();

  let postgres = match connect(&Database(token_path().await.postgres_uri)).await {
    Ok(p) => {
      info!("Database connection established");
      p
    },
    Err(e) => {
      error!("Database connection error: {e}");
      std::process::exit(1);
    }
  };

  let _ = prepare_tables(&postgres, "schemas").await;

  let grpc = MonicaClient::new();
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

  asahi::spawn(SeasonalTheme);

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
  .event_handler(events::DiscordEvents {
    probe: Arc::clone(&health_probe)
  })
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
