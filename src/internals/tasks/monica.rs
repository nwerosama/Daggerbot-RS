use {
  super::{
    super::{
      config::BINARY_PROPERTIES,
      utils::discord_token
    },
    task_err,
    task_info
  },
  crate::{
    BotData,
    BotError,
    commands::PoiseContext,
    controllers::{
      cache::RedisController,
      sql::{
        MpServers,
        Webhooks
      }
    }
  }
};

use {
  dag_grpc::FetchRequest,
  image::Rgba,
  lazy_static::lazy_static,
  num_format::{
    Locale,
    ToFormattedString
  },
  poise::serenity_prelude::{
    AutocompleteChoice,
    Context,
    CreateAutocompleteResponse,
    CreateEmbed,
    CreateEmbedAuthor,
    CreateEmbedFooter,
    EditMessage,
    EditWebhookMessage,
    EmbedField,
    ExecuteWebhook,
    GenericChannelId,
    Http,
    MessageId,
    ThreadId,
    Timestamp,
    Webhook,
    WebhookId
  },
  regex::Regex,
  serde::{
    Deserialize,
    Serialize
  },
  serde_json::Value,
  std::{
    collections::HashMap,
    sync::{
      Arc,
      atomic::{
        AtomicI32,
        Ordering::{
          Acquire,
          Release
        }
      }
    }
  },
  tokio::time::{
    Duration,
    interval
  }
};

pub static TASK_NAME: &str = "Monica";
static NO_SERVERS_TEXT: &str = "No servers are available at this time";
static REFRESH_TEXT: &str = "Refreshes every {{ refresh.timer }} seconds";
static REFRESH_TIMER_SECS: u64 = 40;
pub static EMPTY_PLAYER_LIST_TEXT: &str = "*Nobody is playing*";
pub static SERVER_SEARCH_FILTERS: &str = "https://discord.com/channels/468835415093411861/468835769092669461/1331780599228399636";

const UNKNOWN_SLOT_SYSTEM: &str = "`[UNKNOWN_SLOT_SYSTEM]`";
const UNKNOWN_TIME: &str = "`[UNKNOWN_TIME]`";

/// Due to be changed down the road,
/// but it was found to have 3500 slots.<br>
/// - Nwero, 12/11/24
const CONSOLE_SLOT_LIMIT: i32 = 3500;

lazy_static! {
  static ref PREVIOUS_DAY_TIME: AtomicI32 = AtomicI32::new(0);
  static ref SETTINGS_TXT_MAP: HashMap<TxtMapKey, HashMap<&'static str, &'static str>> = {
    [
      (TxtMapKey::GenericBools, [("false", "Off"), ("true", "On")].iter().cloned().collect()),
      (
        TxtMapKey::GrowthMode,
        [("1", "Yes"), ("2", "No"), ("3", "Growth paused")].iter().cloned().collect()
      ),
      (
        TxtMapKey::EconomicDifficulty,
        [("EASY", "Easy"), ("NORMAL", "Normal"), ("HARD", "Hard")].iter().cloned().collect()
      ),
      (
        TxtMapKey::DisasterDestructionState,
        [("ENABLED", "Enabled"), ("VISUALS_ONLY", "Visuals Only"), ("DISABLED", "Disabled")]
          .iter()
          .cloned()
          .collect()
      ),
      (
        TxtMapKey::FuelUsage,
        [("1", "Low"), ("2", "Normal"), ("3", "High")].iter().cloned().collect()
      ),
      (
        TxtMapKey::DirtInterval,
        [("1", "Off"), ("2", "Slow"), ("3", "Normal"), ("4", "Fast")].iter().cloned().collect()
      )
    ]
    .iter()
    .cloned()
    .collect()
  };
}

pub struct EmbedPalette {
  pub green:  u32,
  pub yellow: u32,
  pub red:    u32
}

impl EmbedPalette {
  pub fn new() -> Self {
    Self {
      green:  BINARY_PROPERTIES.embed_colors.green,
      yellow: BINARY_PROPERTIES.embed_colors.yellow,
      red:    BINARY_PROPERTIES.embed_colors.red
    }
  }

  pub fn rgba(
    &self,
    color: u32
  ) -> Rgba<u8> {
    let r = ((color >> 16) & 0xFF) as u8;
    let g = ((color >> 8) & 0xFF) as u8;
    let b = (color & 0xFF) as u8;

    Rgba([r, g, b, 255])
  }
}

// Parts of Farming Simulator API is used, so we
// need to implement the structs for it to be usable
// - Nwero, 6/7/24

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DssData {
  pub server:   Option<DssServer>,
  pub slots:    Option<DssSlots>,
  pub vehicles: Vec<DssVehicle>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CsgData {
  settings:    Option<CsgSettings>,
  #[serde(rename = "slotSystem")]
  slot_system: Option<CsgSlotSystem>
}

// DSS section start

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DssServer {
  #[serde(rename = "dayTime")]
  pub day_time: i32,
  #[serde(rename = "mapName")]
  pub map_name: String,
  pub name:     String,
  pub version:  String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DssSlots {
  pub capacity: i8,
  pub used:     i8,
  pub players:  Vec<DssPlayer>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DssPlayer {
  #[serde(rename = "isUsed")]
  pub is_used:  Option<bool>,
  #[serde(rename = "isAdmin")]
  pub is_admin: Option<bool>,
  pub uptime:   Option<i32>,
  pub name:     Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DssVehicle {
  pub name:     Option<String>,
  pub category: Option<String>
}

// DSS section end

// CSG section start

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CsgSettings {
  #[serde(rename = "mapTitle")]
  map_title:                  String,
  #[serde(rename = "growthMode")]
  growth_mode:                i8,
  #[serde(rename = "fruitDestruction")]
  fruit_destruction:          bool,
  #[serde(rename = "plowingRequiredEnabled")]
  plowing_required_enabled:   bool,
  #[serde(rename = "stonesEnabled")]
  stones_enabled:             bool,
  #[serde(rename = "weedsEnabled")]
  weeds_enabled:              bool,
  #[serde(rename = "limeRequired")]
  lime_enabled:               bool,
  #[serde(rename = "fuelUsage")]
  fuel_usage:                 i8,
  #[serde(rename = "economicDifficulty")]
  economic_difficulty:        String,
  #[serde(rename = "disasterDestructionState")]
  disaster_destruction_state: String,
  #[serde(rename = "dirtInterval")]
  dirt_interval:              i8,
  #[serde(rename = "timeScale")]
  time_scale:                 f32,
  #[serde(rename = "autoSaveInterval")]
  auto_save_interval:         f32
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CsgSlotSystem {
  #[serde(rename = "slotUsage")]
  slot_usage: String
}

// CSG section end

impl DssData {
  fn is_valid(&self) -> bool {
    match &self.server {
      Some(server) => server.day_time > 0,
      None => true
    }
  }
}

impl CsgData {
  fn is_valid(&self) -> bool {
    if let Some(slot_system) = &self.slot_system {
      if !slot_system.slot_usage.is_empty() {
        if let Some(settings) = &self.settings {
          return !settings.map_title.is_empty() && !settings.time_scale.is_normal();
        }
      }
    }
    false
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SavegameSettingsFields {
  fields: Vec<EmbedField>
}

impl SavegameSettingsFields {
  fn new(fields: Vec<EmbedField>) -> Self { Self { fields } }

  fn has_changed(
    &self,
    other: &Self
  ) -> bool {
    // Compare fields and check if they're different
    if self.fields.len() != other.fields.len() {
      return true;
    }

    for (a, b) in self.fields.iter().zip(other.fields.iter()) {
      if a.name != b.name || a.value != b.value || a.inline != b.inline {
        return true;
      }
    }

    // If we reach here and the fields are the same, oh well.
    false
  }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
enum TxtMapKey {
  GenericBools,
  GrowthMode,
  EconomicDifficulty,
  DisasterDestructionState,
  FuelUsage,
  DirtInterval
}

pub async fn ac_serverlist<'a>(
  ctx: PoiseContext<'a>,
  _partial: &'a str
) -> CreateAutocompleteResponse<'a> {
  let servers = MpServers::get_servers(&ctx.data().postgres).await.unwrap();
  CreateAutocompleteResponse::new().set_choices(
    servers
      .iter()
      .filter(|s| s.is_active)
      .map(|s| AutocompleteChoice::new(s.name.clone(), s.name.clone()))
      .collect::<Vec<AutocompleteChoice<'a>>>()
  )
}

pub struct IpCodePair {
  pub ip:   String,
  pub code: String
}

/// Uses regex to extract the IP and code from
/// the provided DSS/CSG url string and returns it as a tuple.<br>
/// If regex fails to match, it returns None.
pub fn extract_ip_and_code(url: &str) -> Option<IpCodePair> {
  let re = Regex::new(
    r"http://(\d{1,3}(?:\.\d{1,3}){3}:\d+)/feed/(?:dedicated-server-stats\.(?:xml|json)|dedicated-server-savegame\.html)\?code=([a-zA-Z0-9]+)"
  )
  .unwrap();

  if let Some(caps) = re.captures(url) {
    let ip = caps.get(1).unwrap().as_str().to_string();
    let code = caps.get(2).unwrap().as_str().to_string();
    Some(IpCodePair { ip, code })
  } else {
    None
  }
}

pub fn mod_page_url(
  server: &MpServers,
  all_mods: bool
) -> String {
  let mut url = String::new();
  let url_tmpl = format!("http://{}", server.ip);

  match all_mods {
    true => url.push_str(&format!("{url_tmpl}/all_mods_download?onlyActive=true")),
    false => url.push_str(&format!("{url_tmpl}/mods.html"))
  }

  url
}

async fn cache_servers(
  redis: &RedisController,
  servers: Vec<MpServers>
) -> Result<(), BotError> {
  let serialized_servers = serde_json::to_string(&servers)?;
  redis.set(TASK_NAME, serialized_servers.as_str()).await?;
  redis.expire(TASK_NAME, 900).await?;

  Ok(())
}

pub async fn monica(ctx: Arc<Context>) -> Result<(), BotError> {
  let mut interval = interval(Duration::from_secs(REFRESH_TIMER_SECS));
  let bot_data = ctx.data::<BotData>();
  let redis = bot_data.redis.clone();
  let postgres = bot_data.postgres.clone();
  let mut showing_no_servers = false;

  let mp_info = GenericChannelId::new(BINARY_PROPERTIES.mp_channels.info);
  let mp_info_msg = MessageId::new(BINARY_PROPERTIES.mp_channels.info_msg);

  loop {
    interval.tick().await;
    let servers = MpServers::get_servers(&postgres).await?;
    let mut embeds: Vec<CreateEmbed<'_>> = Vec::with_capacity(servers.len());

    let servers_in_cache: String = match redis.get(TASK_NAME).await {
      Ok(v) => v.unwrap_or_default(),
      Err(e) => {
        task_err(TASK_NAME, &format!("[monica] Redis failed to get servers: {e}"));
        continue;
      }
    };
    let cached_servers: Vec<MpServers>;

    if servers_in_cache.trim().is_empty() {
      #[cfg(not(feature = "production"))]
      task_info(TASK_NAME, "Redis cache must've expired, pulling fresh data from database...");
      cached_servers = servers.clone();
      cache_servers(&redis, servers.clone()).await?;
    } else {
      #[cfg(not(feature = "production"))]
      task_info(TASK_NAME, "Cache hit, using cached data...");
      cached_servers = serde_json::from_str(&servers_in_cache)?;
    }

    for server in &cached_servers {
      if !server.is_active {
        continue;
      }

      let data = match bot_data
        .grpc
        .clone()
        .fetch_data(FetchRequest {
          server_name: server.name.clone(),
          server_ip:   server.ip.clone(),
          server_code: server.code.clone(),
          is_active:   server.is_active,
          fetch_type:  "both".to_string()
        })
        .await
      {
        Ok(d) => {
          let response_data = d.into_inner().data;

          if response_data.is_empty() {
            embeds.push(
              CreateEmbed::new()
                .color(EmbedPalette::new().red)
                .title(server.name.to_string())
                .description(":no_entry_sign: **Monica passed empty data!**")
                .timestamp(Timestamp::now())
            );
            continue;
          }

          response_data
        },
        Err(e) => {
          if e.message().contains("request timed out: deadline has elapsed") {
            continue;
          }

          eprintln!("gRPC[Error] Monica reported an error: {e}");
          embeds.push(
            CreateEmbed::new()
              .color(EmbedPalette::new().red)
              .title(server.name.to_string())
              .description(":no_entry_sign: **Monica is currently unavailable!**")
              .timestamp(Timestamp::now())
          );
          continue;
        }
      };

      let json_value: Value = match serde_json::from_str(&data) {
        Ok(val) => val,
        Err(e) => {
          task_err(TASK_NAME, &format!("[monica:debug_dump] Invalid JSON structure: {e}"));
          embeds.push(
            CreateEmbed::new()
              .color(EmbedPalette::new().red)
              .title(server.name.to_string())
              .description(":no_entry_sign: **Monica sent invalid structure**")
              .timestamp(Timestamp::now())
          );
          continue;
        }
      };

      let dss_data = json_value.get("dss");
      let csg_data = json_value.get("csg");

      let (dss, csg): (DssData, CsgData) = {
        if dss_data.is_none() || csg_data.is_none() {
          task_err(
            TASK_NAME,
            &format!(
              "[monica:debug_dump] Missing dss/csg fields for \"{}\": dss={dss_data:?}, csg={csg_data:?}",
              server.name
            )
          );
          embeds.push(
            CreateEmbed::new()
              .color(EmbedPalette::new().red)
              .title(server.name.to_string())
              .description(":no_entry_sign: **DSS/CSG data missing some fields, check terminal**")
              .timestamp(Timestamp::now())
          );
          continue;
        }

        if dss_data.unwrap().is_null() || csg_data.unwrap().is_null() {
          task_info(TASK_NAME, &format!("[monica:debug_dump] Received nullified data from {}", server.name));
          embeds.push(
            CreateEmbed::new()
              .color(EmbedPalette::new().yellow)
              .title(server.name.to_string())
              .description(":hourglass: **Server temporarily unavailable**")
              .footer(CreateEmbedFooter::new(
                "Please ping Nwero if this still continues for more than a minute!"
              ))
              .timestamp(Timestamp::now())
          );
          continue;
        }

        match (
          serde_json::from_value(dss_data.unwrap().clone()),
          serde_json::from_value(csg_data.unwrap().clone())
        ) {
          (Ok(dss), Ok(csg)) => (dss, csg),
          (Err(d_e), Err(c_e)) => {
            task_info(TASK_NAME, &format!("[monica:debug_dump] Raw data for {}: {data:?}", server.name));
            task_err(TASK_NAME, &format!("[monica:debug_dump]      dss: {d_e} | csg: {c_e}"));
            embeds.push(
              CreateEmbed::new()
                .color(EmbedPalette::new().red)
                .title(server.name.to_string())
                .description(":no_entry_sign: **Request failed ─ Dead server**")
                .timestamp(Timestamp::now())
            );
            continue;
          },
          (..) => {
            embeds.push(
              CreateEmbed::new()
                .color(EmbedPalette::new().yellow)
                .title(server.name.to_string())
                .description(":warning: **Empty data**")
                .timestamp(Timestamp::now())
            );
            continue;
          }
        }
      };

      let used_slots = dss.slots.clone().unwrap().used as i32;
      let reset_result = MpServers::reset_peak_players(&postgres, server.name.clone()).await?; // Reset peak players count every 72 hours
      let update_result = MpServers::update_peak_players(&postgres, server.name.clone(), used_slots).await?;
      MpServers::update_player_data(&postgres, server.name.clone(), used_slots).await?;

      // Server-specific webhook in each channel
      savegame_settings_webhook(server, ctx.clone(), &json_value).await;
      // Time drift logger
      time_drift_webhook(server, ctx.clone(), &json_value).await;

      if !dss.server.clone().unwrap().name.is_empty() && !dss.is_valid() && !csg.is_valid() {
        task_err(
          TASK_NAME,
          &format!("[monica] Partial data received for \"{}\", not displaying in Discord", server.name)
        );
        println!("[monica:invalid_data_received_embed] {dss:?}"); // Debug trace, this section occurs when server gets rebooted.
        embeds.push(
          CreateEmbed::new()
            .color(EmbedPalette::new().red)
            .title(server.name.to_string())
            .description(":no_entry_sign: **Invalid data received**")
            .timestamp(Timestamp::now())
        );
        continue;
      }

      let peak_players = MpServers::get_peak_players(&postgres, server.name.clone()).await?;
      if reset_result || update_result {
        const PEAK_PLRS_TXT: &str = "Peak players value for";
        if reset_result {
          task_info(
            TASK_NAME,
            &format!("{PEAK_PLRS_TXT} \"{}\" has passed 72 hours and now since reset", server.name)
          );
          cache_servers(&redis, servers.clone()).await?;
        } else {
          task_info(TASK_NAME, &format!("{PEAK_PLRS_TXT} \"{}\" has been updated", server.name));
          cache_servers(&redis, servers.clone()).await?;
        }
      }

      let players = match dss.slots.clone().unwrap().used {
        0 => EMPTY_PLAYER_LIST_TEXT.to_string(),
        _ => playerlist_constructor(dss.slots.clone().unwrap().players)
      };

      let slot_usage = csg.slot_system.map_or_else(
        || {
          task_err(TASK_NAME, &format!("[csg:slot_system] Slot system data missing for \"{}\"", server.name));
          UNKNOWN_SLOT_SYSTEM.to_string()
        },
        |slot_system| {
          slot_system.slot_usage.parse::<i32>().map_or_else(
            |e| {
              task_err(TASK_NAME, &format!("[csg:slot_system] Invalid slot usage value: {e}"));
              UNKNOWN_SLOT_SYSTEM.to_string()
            },
            |current_usage| {
              let formatted_usage = current_usage.to_formatted_string(&Locale::en_AU);
              let limit_str = CONSOLE_SLOT_LIMIT.to_formatted_string(&Locale::en_AU);
              format!("**{formatted_usage}**/**{limit_str}**")
            }
          )
        }
      );

      let time_scale = csg.settings.clone().map_or_else(|| 0.0, |settings| settings.time_scale);

      let embed = CreateEmbed::new()
        .color(BINARY_PROPERTIES.embed_colors.primary)
        .title(dss.server.clone().unwrap().name.to_string())
        .description(players)
        .fields(vec![
          (
            "Time",
            format!("{} ({time_scale}x)", format_daytime(dss.server.clone().unwrap().day_time)),
            true
          ),
          ("Map", dss.server.clone().unwrap().map_name, true),
          ("Slot Usage", slot_usage, true),
        ])
        .author(CreateEmbedAuthor::new(format!(
          "{}/{} ({peak_players})",
          dss.slots.clone().unwrap().used,
          dss.slots.clone().unwrap().capacity
        )))
        .footer(CreateEmbedFooter::new(format!(
          "Autosave: {} mins ∙ Version: {}",
          csg.settings.unwrap().auto_save_interval,
          dss.server.clone().unwrap().version
        )))
        .timestamp(Timestamp::now());

      let embed = if dss.server.unwrap().name.is_empty() {
        CreateEmbed::new()
          .color(EmbedPalette::new().red)
          .title(format!("{} is offline", server.name))
          .timestamp(Timestamp::now())
      } else {
        embed
      };

      embeds.push(embed);
    }

    if embeds.is_empty() && !showing_no_servers {
      task_info(TASK_NAME, "[monica] No embeds to update message with");
      if let Err(y) = mp_info
        .edit_message(&ctx.http, mp_info_msg, EditMessage::new().content(NO_SERVERS_TEXT).embeds(vec![]))
        .await
      {
        task_err(TASK_NAME, &format!("[monica] Error editing message: {y}"));
      }
      showing_no_servers = true;
      continue;
    }

    showing_no_servers = false;

    if let Err(y) = mp_info
      .edit_message(
        &ctx.http,
        mp_info_msg,
        EditMessage::new()
          .content(REFRESH_TEXT.replace("{{ refresh.timer }}", &REFRESH_TIMER_SECS.to_string()))
          .embeds(embeds)
      )
      .await
    {
      task_err(TASK_NAME, &format!("[monica] Error editing message: {y}"));
      continue;
    }
  }
}

pub fn format_daytime(day_time: i32) -> String {
  let hours = day_time / 3600000;
  let mins = (day_time % 3600000) / 60000;

  format!("{hours:02}:{mins:02}")
}

fn format_player_uptime(uptime: i32) -> String {
  let mins: i32;
  let mut hrs: i32 = 0;

  if uptime >= 60 {
    hrs = uptime / 60;
    mins = uptime % 60;
  } else {
    mins = uptime;
  }

  format!(
    "{}{}",
    if hrs > 0 { format!("{hrs} h ") } else { "".to_string() },
    if mins > 0 { format!("{mins} m") } else { "".to_string() }
  )
}

pub fn playerlist_constructor(players: Vec<DssPlayer>) -> String {
  let mut builder = String::new();

  for player in players.into_iter().filter(|p| p.is_used.unwrap_or(false)) {
    let icon = icon_factory(&player);

    let uptime = match player.uptime {
      Some(0) => "Just joined".to_string(),
      Some(uptime) => format!("Playing for {}", format_player_uptime(uptime)),
      None => UNKNOWN_TIME.to_string()
    };

    if let Some(name) = player.name {
      builder.push_str(&format!("**{name}{icon}**\n{uptime}\n\n"));
    }
  }

  builder
}

fn icon_factory(player: &DssPlayer) -> String {
  struct IconCondition<'a> {
    condition: Box<dyn Fn(&DssPlayer) -> bool + 'a>,
    icon:      &'a str
  }

  let icon_conditions = vec![
    IconCondition {
      condition: Box::new(|p| p.is_admin.unwrap_or(false)),
      icon:      ":detective:"
    },
    IconCondition {
      condition: Box::new(|p| p.name.as_ref().is_some_and(|n| n.contains("Nwero"))),
      icon:      "<:NeuroLoad:1334279559889293344>"
    },
    IconCondition {
      condition: Box::new(|p| p.name.as_ref().is_some_and(|n| n.contains("Daggerwin"))),
      icon:      "<:Daggerwin:549283056079339520>"
    },
  ];

  let mut builder = String::new();
  for pair in icon_conditions {
    if (pair.condition)(player) {
      builder.push_str(pair.icon)
    }
  }

  builder
}

async fn savegame_settings_webhook(
  server: &MpServers,
  ctx: Arc<Context>,
  data: &Value
) {
  if !server.is_active {
    return;
  }

  fn get_mapped_value(
    map_key: &TxtMapKey,
    key: &str
  ) -> &'static str {
    SETTINGS_TXT_MAP.get(map_key).and_then(|map| map.get(key)).unwrap_or(&"Unknown Value")
  }

  let csg: CsgData = match serde_json::from_value(data["csg"].clone()) {
    Ok(c) => c,
    Err(e) => {
      task_err(TASK_NAME, &format!("[savegame_settings_webhook] Failed to deserialize CSG data: {e}"));
      return;
    }
  };

  let csg_settings = match &csg.settings {
    Some(settings) => settings,
    None => {
      task_err(
        TASK_NAME,
        &format!("[savegame_settings_webhook] CSG settings not found for \"{}\"", server.name)
      );
      return;
    }
  };

  let redis = &ctx.data::<BotData>().redis;
  let cache_key = format!("{TASK_NAME}:savegame_settings:{}", server.name);
  let csg_settings__ = csg.settings.is_some();

  let efields = if let Some(csg_settings) = &csg.settings {
    vec![
      (
        "Seasonal Growth",
        get_mapped_value(&TxtMapKey::GrowthMode, &csg_settings.growth_mode.to_string()),
        true
      ),
      (
        "Crop Destruction",
        get_mapped_value(&TxtMapKey::GenericBools, &csg_settings.fruit_destruction.to_string()),
        true
      ),
      (
        "Periodic Plowing",
        get_mapped_value(&TxtMapKey::GenericBools, &csg_settings.plowing_required_enabled.to_string()),
        true
      ),
      (
        "Stones",
        get_mapped_value(&TxtMapKey::GenericBools, &csg_settings.stones_enabled.to_string()),
        true
      ),
      (
        "Lime",
        get_mapped_value(&TxtMapKey::GenericBools, &csg_settings.lime_enabled.to_string()),
        true
      ),
      (
        "Weeds",
        get_mapped_value(&TxtMapKey::GenericBools, &csg_settings.weeds_enabled.to_string()),
        true
      ),
      (
        "Economic Difficulty",
        get_mapped_value(&TxtMapKey::EconomicDifficulty, &csg_settings.economic_difficulty),
        true
      ),
      (
        "Disaster Destruction",
        get_mapped_value(&TxtMapKey::DisasterDestructionState, &csg_settings.disaster_destruction_state),
        true
      ),
      (
        "Fuel Usage",
        get_mapped_value(&TxtMapKey::FuelUsage, &csg_settings.fuel_usage.to_string()),
        true
      ),
      (
        "Dirt Interval",
        get_mapped_value(&TxtMapKey::DirtInterval, &csg_settings.dirt_interval.to_string()),
        true
      ),
    ]
  } else {
    task_info(
      TASK_NAME,
      &format!("[savegame_settings_webhook] CSG settings temporarily unavailable for \"{}\"", server.name)
    );

    vec![("Status", "Savegame settings are unavailable due to fresh save!", false)]
  };

  let current_fields = SavegameSettingsFields::new(efields.iter().map(|(n, v, i)| EmbedField::new(*n, *v, *i)).collect());
  let previous_fields: Option<SavegameSettingsFields> = redis
    .get(&cache_key)
    .await
    .ok()
    .flatten()
    .and_then(|data: String| serde_json::from_str(&data).ok());

  if (csg_settings__ || previous_fields.is_none()) && previous_fields.as_ref().is_some_and(|pf| !current_fields.has_changed(pf)) {
    return;
  }

  let bot_http = Http::new(discord_token().await);
  let hookdb = match Webhooks::get_hooks(&ctx.data::<BotData>().postgres).await {
    Ok(hooks) => hooks,
    Err(e) => {
      task_err(TASK_NAME, &format!("[savegame_settings_webhook] Failed to get webhooks: {e}"));
      return;
    }
  };

  if let Some(hook) = hookdb.into_iter().find(|h| h.name == server.name) {
    let webhook_id = match hook.id.parse::<u64>() {
      Ok(id) => WebhookId::new(id),
      Err(e) => {
        task_err(TASK_NAME, &format!("[savegame_settings_webhook:{}] Invalid webhook ID: {e}", server.name));
        return;
      }
    };

    let message_id = match hook.message_id.parse::<u64>() {
      Ok(id) => MessageId::new(id),
      Err(e) => {
        task_err(TASK_NAME, &format!("[savegame_settings_webhook:{}] Invalid message ID: {e}", server.name));
        return;
      }
    };

    match Webhook::from_id_with_token(&bot_http, webhook_id, &hook.token).await {
      Ok(webhook) => {
        let embed = CreateEmbed::default()
          .color(if csg_settings__ {
            BINARY_PROPERTIES.embed_colors.primary
          } else {
            BINARY_PROPERTIES.embed_colors.yellow
          })
          .title(format!("Savegame Settings - {}", csg_settings.map_title))
          .fields(efields)
          .footer(CreateEmbedFooter::new(&server.name))
          .timestamp(Timestamp::now());

        match webhook
          .edit_message(&bot_http, message_id, EditWebhookMessage::default().content(String::new()).embed(embed))
          .await
        {
          Ok(_) => {
            if let Err(e) = redis.set(&cache_key, &serde_json::to_string(&current_fields).unwrap()).await {
              task_err(
                TASK_NAME,
                &format!("[savegame_settings_webhook:{}] Redis failed to set cache: {e}", server.name)
              );
            }
          },
          Err(e) => {
            task_err(
              TASK_NAME,
              &format!("[savegame_settings_webhook:{}] Webhook failed to edit message: {e}", server.name)
            );
          }
        }
      },
      Err(e) => {
        task_err(
          TASK_NAME,
          &format!("[savegame_settings_webhook:{}] Webhook doesn't exist: {e}", server.name)
        );
      }
    }
  }
}

async fn time_drift_webhook(
  server: &MpServers,
  ctx: Arc<Context>,
  data: &Value
) {
  if !server.is_active {
    return;
  }

  let dss: DssData = match serde_json::from_value(data["dss"].clone()) {
    Ok(c) => c,
    Err(e) => {
      task_err(
        TASK_NAME,
        &format!("[time_drift_webhook:{}] Failed to deserialize DSS data: {e}", server.name)
      );
      return;
    }
  };

  let dss_slots = match dss.slots.clone() {
    Some(s) => s,
    None => {
      task_err(TASK_NAME, "[time_drift_webhook:{}] DSS slots data is empty");
      return;
    }
  };

  let player_names = dss_slots
    .players
    .iter()
    .filter(|p| p.is_used.unwrap_or(false))
    .filter_map(|p| p.name.clone().map(|name| format!("{name}{}", icon_factory(p))));

  let plist = match dss.slots.clone().unwrap().used {
    0 => EMPTY_PLAYER_LIST_TEXT.to_string(),
    _ => player_names.collect::<Vec<String>>().join("\n")
  };

  if dss.server.as_ref().is_none_or(|s| s.name.is_empty() && dss.is_valid()) {
    return;
  }

  /// 00:03
  const MIDNIGHT: i32 = 220000;
  /// 09:30
  const MORNING: i32 = 34249800;
  /// 17:35
  const EVENING: i32 = 63333710;

  let redis = ctx.data::<crate::BotData>().redis.clone();
  let redis_webhook_sent = format!("{TASK_NAME}:time_drift:{}:webhook_sent", server.name);

  let current_time = dss.server.as_ref().map(|s| s.day_time).unwrap_or_default();
  let previous_time = PREVIOUS_DAY_TIME.load(Acquire);
  if previous_time == 0 {
    PREVIOUS_DAY_TIME.store(current_time, Release);
    return;
  }

  PREVIOUS_DAY_TIME.store(current_time, Release);

  // Check if webhook was already sent
  match redis.get(&redis_webhook_sent).await {
    Ok(Some(webhook_sent)) => {
      if webhook_sent == "true" {
        // Reset the flag if the current time is approaching 17:35
        if current_time >= EVENING {
          if let Err(e) = redis.set(&redis_webhook_sent, "false").await {
            task_err(
              TASK_NAME,
              &format!("[time_drift_webhook:{}] Failed to reset webhook sent flag: {e}", server.name)
            );
          }
        }
        return;
      }
    },
    Ok(None) => {
      if let Err(e) = redis.set(&redis_webhook_sent, "false").await {
        task_err(
          TASK_NAME,
          &format!("[time_drift_webhook:{}] Failed to set initial webhook sent flag: {e}", server.name)
        );
        return;
      }
    },
    Err(e) => {
      task_err(TASK_NAME, &format!("[time_drift_webhook] Redis failed to get webhook_sent: {e}"));
      return;
    }
  }

  let bot_http = Http::new(discord_token().await);
  let hookdb = match Webhooks::get_hooks(&ctx.data::<BotData>().postgres).await {
    Ok(hooks) => hooks,
    Err(e) => {
      task_err(TASK_NAME, &format!("[time_drift_webhook] Failed to get webhooks: {e}"));
      return;
    }
  };

  if (MIDNIGHT..=MORNING).contains(&current_time) {
    for hook in hookdb {
      if hook.name != server.name {
        continue;
      }

      let webhook = match Webhook::from_id_with_token(&bot_http, WebhookId::new(hook.id.parse().unwrap_or_default()), &hook.token).await {
        Ok(webhook) => webhook,
        Err(e) => {
          task_err(TASK_NAME, &format!("[time_drift_webhook:{}] Webhook doesn't exist: {e}", server.name));
          continue;
        }
      };

      let server_name = dss
        .server
        .as_ref()
        .map(|s| s.name.clone())
        .unwrap_or_else(|| "Unknown Server".to_string());

      match webhook
        .execute(
          &bot_http,
          true,
          ExecuteWebhook::new()
            .in_thread(ThreadId::new(hook.thread_id.parse::<u64>().unwrap_or_default()))
            .embed(
              CreateEmbed::new()
                .color(BINARY_PROPERTIES.embed_colors.primary)
                .title(format!("Time difference - {server_name}"))
                .description(format!(
                  "It is the new day, previously it was **{}** and it's now **{}**.\nList of players that were on:\n**{plist}**",
                  format_daytime(previous_time),
                  format_daytime(current_time)
                ))
            )
        )
        .await
      {
        Ok(_) => {
          if (redis.set(&redis_webhook_sent, "true").await).is_err() {
            continue;
          }
        },
        Err(e) => {
          if e.to_string().contains("Unknown Channel") {
            task_err(
              TASK_NAME,
              &format!(
                "[time_drift_webhook:{}] Webhook's parent channel is not where the thread is located",
                server.name
              )
            );
            task_err(TASK_NAME, &format!("[time_drift_webhook:{}] Raw error: {e}", server.name));
            continue;
          }
          task_err(TASK_NAME, &format!("[time_drift_webhook:{}] Webhook failed to send: {e}", server.name));
        }
      }
    }
  }
}

mod error_humor {
  const OBJECTS: [&str; 37] = [
    "tree",
    "rock",
    "wall",
    "fence",
    "sign",
    "car",
    "bus",
    "bike",
    "tractor",
    "pedestrian",
    "animal",
    "cow",
    "sheep",
    "bench",
    "table",
    "chair",
    "house",
    "building",
    "barn",
    "skyscraper",
    "statue",
    "monument",
    "lamp post",
    "street light",
    "traffic light",
    "stop sign",
    "bus stop",
    "bridge",
    "fountain",
    "dumpster",
    "mailbox",
    "trash can",
    "mailbox",
    "parking meter",
    "glass door",
    "window",
    "door"
  ];

  pub struct Collider;

  impl Collider {
    pub fn encounter_random_object() -> &'static str { OBJECTS[rand::random::<u32>() as usize % OBJECTS.len()] }
  }
}

pub use error_humor::Collider;
