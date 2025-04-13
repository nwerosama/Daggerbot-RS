use crate::{
  Error,
  controllers::sql::MpServers,
  internals::{
    ansi::Color,
    canvas::Canvas,
    config::BINARY_PROPERTIES,
    nats::MonicaNatsPayload,
    tasks::monica::{
      Collider,
      DssData,
      EMPTY_PLAYER_LIST_TEXT,
      EmbedPalette,
      SERVER_SEARCH_FILTERS,
      TASK_NAME,
      ac_serverlist,
      extract_ip_and_code,
      format_daytime,
      mod_page_url,
      playerlist_constructor
    }
  }
};

use {
  dashmap::DashMap,
  poise::{
    CreateReply,
    serenity_prelude::{
      CreateAttachment,
      CreateEmbed,
      CreateEmbedAuthor,
      CreateEmbedFooter,
      CreateMessage,
      CreatePoll,
      CreatePollAnswer,
      GenericChannelId,
      GetMessages,
      MessageId,
      RoleId
    }
  },
  serde_json::json,
  std::borrow::Cow
};

const CATEGORY_FILTER: [&str; 5] = ["PALLETS", "PALLETSILAGE", "BIGBAGS", "BIGBAGPALLETS", "IBC"];
const URL_EXTRACTION_FAILED: &str = "Couldn't parse the provided URL, please check and try again.";
const SERVER_OFFLINE_TEXT: &str = "**{{ name }}** is currently offline!";

trait IsVowel {
  fn is_vowel(&self) -> bool;
}

impl IsVowel for char {
  fn is_vowel(&self) -> bool { matches!(self, 'a' | 'e' | 'i' | 'o' | 'u') }
}

fn get_pallet_counts(data: &DssData) -> DashMap<Cow<'_, str>, usize> {
  let counts = DashMap::new();

  data
    .vehicles
    .iter()
    .filter_map(|pallet| {
      if !CATEGORY_FILTER.contains(&&pallet.category.as_ref()?[..]) {
        return None;
      }

      pallet.name.as_deref()
    })
    .for_each(|name| {
      *counts.entry(Cow::Borrowed(name)).or_insert(0) += 1;
    });

  counts
}

fn levenshtein(
  a: &str,
  b: &str
) -> usize {
  let (a, b) = if a.len() < b.len() { (b, a) } else { (a, b) };
  let mut prev_row = (0..=b.len()).collect::<Vec<_>>();
  let mut curr_row = vec![0; b.len() + 1];

  for (i, ca) in a.chars().enumerate() {
    curr_row[0] = i + 1;

    for (j, cb) in b.chars().enumerate() {
      curr_row[j + 1] = if ca == cb {
        prev_row[j]
      } else {
        1 + curr_row[j].min(prev_row[j]).min(prev_row[j + 1])
      };
    }

    std::mem::swap(&mut prev_row, &mut curr_row);
  }

  prev_row[b.len()]
}

fn normalize_string(s: &str) -> Cow<'_, str> {
  if s.chars().all(|c| c.is_ascii_alphanumeric()) {
    Cow::Borrowed(s)
  } else {
    Cow::Owned(s.chars().filter(|c| c.is_alphanumeric()).collect())
  }
}

async fn is_channel_allowed(ctx: super::PoiseContext<'_>) -> bool {
  let channel_id = ctx.channel_id().get();
  let whitelisted_channels = [BINARY_PROPERTIES.mp_channels.activeplayers];
  let whitelisted_roles = [BINARY_PROPERTIES.mp_mod_role, BINARY_PROPERTIES.mp_manager_role];

  if ctx.guild_id().unwrap().get() != BINARY_PROPERTIES.guild_id {
    return true
  }

  if let Some(member) = ctx.author_member().await {
    if member.roles.iter().any(|r| whitelisted_roles.contains(&r.get())) {
      return true
    }
  }

  if !whitelisted_channels.contains(&channel_id) {
    ctx
      .send(
        CreateReply::new()
          .ephemeral(true)
          .content("This command is not allowed to be used in this channel!")
      )
      .await
      .unwrap();
    return false
  }

  true
}

/// Retrieve specific information from FSMP server(s)
#[poise::command(slash_command, subcommands("players", "details", "pallets", "poll", "tools"))]
pub async fn mp(_: super::PoiseContext<'_>) -> Result<(), Error> { Ok(()) }

async fn data_warehouse(
  ctx: super::PoiseContext<'_>,
  server: String
) -> Result<DssData, Error> {
  let servers = MpServers::get_servers(&ctx.data().postgres).await?;
  let collider = Collider::encounter_random_object();
  let a_ = if collider.chars().next().unwrap().is_vowel() { "an" } else { "a" };

  let server = match servers.iter().find(|s| s.name == server) {
    Some(s) => s,
    None => {
      let closest_server = servers
        .iter()
        .filter(|s| s.is_active)
        .min_by(|a, b| {
          let a_dist = levenshtein(&a.name, &normalize_string(&server));
          let b_dist = levenshtein(&b.name, &normalize_string(&server));
          a_dist.cmp(&b_dist)
        })
        .unwrap();

      return Err(format!("**{server}** does not exist in database, closest server is **{}**.", closest_server.name).into());
    }
  };

  match ctx
    .data()
    .nats
    .publish(MonicaNatsPayload {
      identifier: "dss_only".to_string(),
      data:       json!({
        "name": server.name,
        "ip": server.ip,
        "code": server.code,
        "is_active": server.is_active
      })
    })
    .await
  {
    Ok(d) => {
      if d.data.is_null() || d.data["dss"].is_null() {
        eprintln!("DataWarehouse[Error] 'dss' field is nullified for {}", server.name);
        return Err("Monica didn't reply to the payload request in time, try again later!".to_string().into());
      }

      Ok(match serde_json::from_value::<DssData>(d.data["dss"].clone()) {
        Ok(dss) => dss,
        Err(y) => {
          eprintln!("DataWarehouse[Error] {y}");
          return Err("Monica returned an unexpected error!".to_string().into())
        }
      })
    },
    Err(y) => {
      eprintln!("DataWarehouse[Error] {y}");
      Err(format!("Ran into {a_} {collider} while trying to retrieve server data, please try again later.").into())
    }
  }
}

/// Fetches the list of players on the given server
#[poise::command(slash_command)]
async fn players(
  ctx: super::PoiseContext<'_>,
  #[description = "What server to get players from"]
  #[autocomplete = "ac_serverlist"]
  server: String
) -> Result<(), Error> {
  if !is_channel_allowed(ctx).await {
    return Ok(());
  }

  ctx.defer().await?;

  let server_clone = server.clone();
  let api = match data_warehouse(ctx, server.clone()).await {
    Ok(a) => a,
    Err(y) => {
      ctx.reply(y.to_string()).await?;
      return Ok(());
    }
  };
  let api_server = api.server.unwrap().clone();
  let api_slots = match api.slots {
    Some(s) => s,
    None => {
      ctx.reply(format!("**{server}**'s data structure is malformed, try again later.")).await?;
      return Ok(());
    }
  };

  let postgres = ctx.data().postgres.clone();
  let pd = MpServers::get_player_data(&postgres, server_clone.clone()).await?;
  let peak = MpServers::get_peak_players(&postgres, server_clone).await?;

  // Graph visibly displays the last 53 minutes worth of data,
  // each dot represents the data within 45 seconds apart
  let mut canvas = Canvas::new();
  canvas.render(pd.iter().map(|x| *x as f64).collect());
  let file = "Monica.jpg";

  let players = match api_slots.used {
    0 => EMPTY_PLAYER_LIST_TEXT.to_string(),
    _ => playerlist_constructor(api_slots.players)
  };

  let palette = EmbedPalette::new();
  let color = if api_slots.capacity == 0 && api_server.day_time == 0 {
    palette.red
  } else {
    match api_slots.used {
      0..=5 => palette.green,
      6..=10 => palette.yellow,
      11..=16 => palette.red,
      _ => palette.green
    }
  };

  let srv_name = match api_server.name.is_empty() {
    true => "Offline".to_string(),
    false => api_server.name
  };

  let embed = CreateEmbed::new()
    .color(color)
    .author(CreateEmbedAuthor::new(format!("{}/{} ({peak})", api_slots.used, api_slots.capacity)))
    .title(srv_name)
    .description(players)
    .image(format!("attachment://{file}"))
    .footer(CreateEmbedFooter::new(format!("Current time: {}", format_daytime(api_server.day_time))));

  ctx
    .send(
      CreateReply::default()
        .embed(embed)
        .attachment(CreateAttachment::bytes(canvas.export(), file))
    )
    .await?;

  Ok(())
}

/// Fetches the given server's information like password, map and so forth
#[poise::command(slash_command)]
async fn details(
  ctx: super::PoiseContext<'_>,
  #[description = "What server to get details from"]
  #[autocomplete = "ac_serverlist"]
  server: String
) -> Result<(), Error> {
  ctx.defer().await?;

  let api = {
    let data = match data_warehouse(ctx, server.clone()).await {
      Ok(d) => d,
      Err(y) => {
        ctx.reply(y.to_string()).await?;
        return Ok(());
      }
    };
    data.server.unwrap().clone()
  };
  let servers = MpServers::get_servers(&ctx.data().postgres).await?;
  let srv = servers.iter().find(|s| s.name == server).unwrap();

  if api.name.is_empty() && api.map_name.is_empty() {
    ctx.reply(SERVER_OFFLINE_TEXT.replace("{{ name }}", &srv.name).to_string()).await?;
    return Ok(());
  }

  let hide_mods = if srv.ip.starts_with("85.") {
    format!(
      "[Click here]({}) **|** [Direct link]({})",
      mod_page_url(srv, false),
      mod_page_url(srv, true)
    )
  } else {
    "<<private to members only>>".to_string()
  };

  let srv_details = [
    format!("**Name:** `{}`", api.name),
    format!("**Password:** `{}`", srv.game_password),
    format!("**Map:** `{}`", api.map_name),
    format!("**Mods:** {hide_mods}"),
    format!("**Filters:** [Click here]({SERVER_SEARCH_FILTERS})"),
    format!(
      "*Please see <#{}> for more additional information and rules*",
      BINARY_PROPERTIES.mp_channels.info
    )
  ];

  ctx
    .send(
      CreateReply::default().embed(
        CreateEmbed::new()
          .color(BINARY_PROPERTIES.embed_colors.primary)
          .author(CreateEmbedAuthor::new("Crossplay"))
          .description(srv_details.join("\n"))
      )
    )
    .await?;

  Ok(())
}

/// Fetches the given server's pallet count
#[poise::command(slash_command)]
async fn pallets(
  ctx: super::PoiseContext<'_>,
  #[description = "What server to get details from"]
  #[autocomplete = "ac_serverlist"]
  server: String
) -> Result<(), Error> {
  if !is_channel_allowed(ctx).await {
    return Ok(());
  }

  ctx.defer().await?;

  let (api, api_srv, api_veh) = {
    let data = match data_warehouse(ctx, server.clone()).await {
      Ok(d) => d,
      Err(y) => {
        ctx.reply(y.to_string()).await?;
        return Ok(());
      }
    };
    (data.clone(), data.server.unwrap().clone(), data.vehicles.clone())
  };
  let servers = MpServers::get_servers(&ctx.data().postgres).await?;
  let srv = servers.iter().find(|s| s.name == server).unwrap();

  if api_veh.is_empty() && api_srv.name.is_empty() {
    ctx.reply(SERVER_OFFLINE_TEXT.replace("{{ name }}", &srv.name).to_string()).await?;
    return Ok(());
  }

  let filter = api_veh
    .iter()
    .filter(|x| x.category.as_ref().map(|c| CATEGORY_FILTER.contains(&&c[..])).unwrap_or(false))
    .collect::<Vec<_>>();

  let rules = match filter.len() {
    1 => "single pallet",
    _ => "pallets"
  };

  if filter.is_empty() {
    ctx.reply(format!("Nothing to search on **{}**!", srv.name)).await?;
  } else {
    let mut pallet_counts = get_pallet_counts(&api).into_iter().collect::<Vec<_>>();
    pallet_counts.sort_by(|a, b| b.1.cmp(&a.1));
    let get_longest_name = pallet_counts.iter().map(|(k, _)| k.len()).max().unwrap();

    let pallet_details = pallet_counts
      .iter()
      .map(|(k, v)| {
        let width = get_longest_name + 3;
        let padding = format!("{k:<width$}");
        format!("{}{}", Color::Blue.bold().paint(&padding), Color::Yellow.bold().paint(&v.to_string()))
      })
      .collect::<Vec<String>>()
      .join("\n");

    ctx
      .reply(format!(
        "There are currently **{}** {rules} on **{}**. Here's the breakdown:\n```ansi\n{pallet_details}\n```",
        filter.len(),
        srv.name
      ))
      .await?;
  }

  Ok(())
}

async fn poll_perm_check(ctx: super::PoiseContext<'_>) -> Result<bool, Error> {
  match ctx
    .author_member()
    .await
    .unwrap()
    .roles
    .contains(&RoleId::new(BINARY_PROPERTIES.mp_mod_role))
  {
    true => Ok(true),
    false => Ok(false)
  }
}

async fn poll_webhook_embed(ctx: super::PoiseContext<'_>) -> Result<poise::serenity_prelude::Embed, Error> {
  let suggestion_pool = {
    let mp_moderators = GenericChannelId::new(BINARY_PROPERTIES.mp_channels.mod_chat);
    let hook_msg = MessageId::new(BINARY_PROPERTIES.mp_channels.suggestion_pool_msg);

    match ctx.http().get_message(mp_moderators, hook_msg).await {
      Ok(m) => m,
      Err(y) => {
        ctx.reply(format!("Error while fetching the webhook: {y}")).await?;
        return Ok(poise::serenity_prelude::Embed::default());
      }
    }
  };

  Ok(suggestion_pool.embeds[0].clone())
}

/// Poll system
#[poise::command(slash_command, subcommands("start", "end", "maps"), check = "poll_perm_check")]
pub async fn poll(_: super::PoiseContext<'_>) -> Result<(), Error> { Ok(()) }

/// Start a map poll
#[poise::command(slash_command)]
async fn start(ctx: super::PoiseContext<'_>) -> Result<(), Error> {
  ctx.defer().await?;
  let mp_announcements = GenericChannelId::new(BINARY_PROPERTIES.mp_channels.announcements);

  let mpa_msgs = match mp_announcements.messages(ctx.http(), GetMessages::new().limit(5)).await {
    Ok(m) => m,
    Err(y) => {
      ctx.reply(y.to_string()).await?;
      return Ok(());
    }
  };

  if let Some(live_poll) = mpa_msgs
    .iter()
    .find(|m| m.poll.as_ref().is_some_and(|p| !p.results.as_ref().is_some_and(|r| r.is_finalized)))
  {
    ctx
      .reply(format!(
        "There's already a [map vote currently in progress](<{}>), finish that first!",
        live_poll.link()
      ))
      .await?;
    return Ok(());
  }

  let mut poll_choices: Vec<CreatePollAnswer> = vec![];
  let map_pool = poll_webhook_embed(ctx).await.unwrap().description.unwrap();

  // Validate the sequential list of maps
  let lines: Vec<&str> = map_pool.lines().collect();
  let mut expected_num = 1;
  for line in &lines {
    if !line.trim().starts_with(&format!("{expected_num}.")) {
      ctx
        .reply(format!(
          "Line validation failed at: {line}\nExpected line to start with '{expected_num}.' \nFormat should be: \n```1. [Map Name](Mod Link)\n2. \
           [Map Name](Mod Link)\netc...\n```"
        ))
        .await?;
      return Ok(());
    }
    expected_num += 1;
  }

  // If validation passes, run the regex and extract the map names
  let re = regex::Regex::new(r"\d+\.\s*\[([^\]]+)\](?:\(.*?\))?").unwrap();
  for caps in re.captures_iter(&map_pool) {
    poll_choices.push(CreatePollAnswer::new().text(caps.get(1).unwrap().as_str()));
  }

  match mp_announcements
    .send_message(
      ctx.http(),
      CreateMessage::new().content(format!("<@&{}>", BINARY_PROPERTIES.mp_players_role)).poll(
        CreatePoll::new()
          .question("Vote for the next map!")
          .answers(poll_choices)
          .duration(std::time::Duration::from_secs(259200)) // 3 days
      )
    )
    .await
  {
    Ok(m) => ctx.reply(format!("[Poll](<{}>) started!", m.link())).await?,
    Err(y) => ctx.reply(format!("Error while sending the poll: {y}")).await?
  };

  Ok(())
}

/// End the map poll early
#[poise::command(slash_command)]
async fn end(ctx: super::PoiseContext<'_>) -> Result<(), Error> {
  let mp_announcements = GenericChannelId::new(BINARY_PROPERTIES.mp_channels.announcements);

  let messages = match mp_announcements.messages(ctx.http(), GetMessages::new().limit(5)).await {
    Ok(m) => m,
    Err(y) => {
      ctx.reply(y.to_string()).await?;
      return Ok(());
    }
  };

  let poll_msg = messages.iter().find(|m| m.poll.is_some()).unwrap();

  match ctx.http().expire_poll(mp_announcements, poll_msg.id).await {
    Ok(_) => ctx.reply(format!("[Poll](<{}>) ended early!", poll_msg.link())).await?,
    Err(y) => ctx.reply(y.to_string()).await?
  };

  Ok(())
}

/// Retrieve the list from the suggestion pool
#[poise::command(slash_command)]
async fn maps(ctx: super::PoiseContext<'_>) -> Result<(), Error> {
  ctx
    .send(CreateReply::default().embed(CreateEmbed::from(poll_webhook_embed(ctx).await?)))
    .await?;

  Ok(())
}

async fn tools_perm_check(ctx: super::PoiseContext<'_>) -> Result<bool, Error> {
  let member = ctx.author_member().await.unwrap();
  let roles = member.roles.clone();
  let perms = member.permissions.unwrap();

  Ok(roles.contains(&RoleId::new(BINARY_PROPERTIES.mp_manager_role)) || perms.administrator())
}

/// MP Manager tools for Monica
#[poise::command(slash_command, check = "tools_perm_check", subcommands("list", "add", "delete", "update"))]
async fn tools(_: super::PoiseContext<'_>) -> Result<(), Error> { Ok(()) }

/// List all available servers in the database
#[poise::command(slash_command)]
async fn list(ctx: super::PoiseContext<'_>) -> Result<(), Error> {
  let servers = MpServers::get_servers(&ctx.data().postgres).await?;
  let mut server_list = Vec::new();

  if servers.is_empty() {
    ctx.reply("No servers found in the database!").await?;
    return Ok(());
  }

  for server in servers {
    let active_flag = if server.is_active { "Yes" } else { "No" };

    server_list.push(format!("- **{}**", server.name));
    server_list.push(format!("  - IP: `{}`", server.ip));
    server_list.push(format!("  - Code: `{}`", server.code));
    server_list.push(format!("  - Password: `{}`", server.game_password));
    server_list.push(format!("  - Active: {active_flag}"));
  }

  ctx.reply(server_list.join("\n")).await?;

  Ok(())
}

/// Add a new server to the database
#[poise::command(slash_command)]
async fn add(
  ctx: super::PoiseContext<'_>,
  #[description = "Friendly name (for autocomplete and autorefresh embeds)"] name: String,
  #[description = "Server URL (DSS/CSG link)"] url: String,
  #[description = "Game password (optional, default is -)"] password: Option<String>,
  #[description = "Active status (optional, default is true)"] active: Option<bool>
) -> Result<(), Error> {
  let extracted = match extract_ip_and_code(&url) {
    Some(e) => e,
    None => {
      ctx.reply(URL_EXTRACTION_FAILED).await?;
      return Ok(());
    }
  };

  let pw = password.unwrap_or("-".to_string());
  let act = active.unwrap_or(true);

  match MpServers::create_server(&ctx.data().postgres, name.clone(), extracted.ip, extracted.code, pw, act).await {
    Ok(_) => {
      ctx.data().redis.del(TASK_NAME).await?;
      ctx.reply(format!("**{name}** successfully added!")).await?;
    },
    Err(y) => {
      ctx.reply(format!("Error while adding server: {y}")).await?;
    }
  }

  Ok(())
}

/// Delete a server from the database
#[poise::command(slash_command)]
async fn delete(
  ctx: super::PoiseContext<'_>,
  #[description = "Server name"]
  #[autocomplete = "ac_serverlist"]
  name: String
) -> Result<(), Error> {
  if MpServers::get_server(&ctx.data().postgres, name.clone()).await?.is_none() {
    ctx.reply(format!("**{name}** doesn't exist in database!")).await?;
    return Ok(());
  }

  match MpServers::delete_server(&ctx.data().postgres, name.clone()).await {
    Ok(_) => {
      ctx.data().redis.del(TASK_NAME).await?;
      ctx.reply(format!("**{name}** successfully deleted!")).await?;
    },
    Err(y) => {
      ctx.reply(format!("Error while deleting server: {y}")).await?;
    }
  }

  Ok(())
}

/// Update an existing server in the database
#[poise::command(slash_command)]
async fn update(
  ctx: super::PoiseContext<'_>,
  #[description = "Server to update"]
  #[autocomplete = "ac_serverlist"]
  name: String,
  #[description = "Server URL (DSS/CSG link)"] url: Option<String>,
  #[description = "Game password (If password is none, put a hyphen instead)"] password: Option<String>,
  #[description = "Active status"] active: Option<bool>
) -> Result<(), Error> {
  if url.is_none() && password.is_none() && active.is_none() {
    ctx.reply("Please provide atleast one field to update.").await?;
    return Ok(());
  }

  let existing_server = MpServers::get_server(&ctx.data().postgres, name.clone()).await?;

  if existing_server.is_none() {
    ctx.reply(format!("**{name}** doesn't exist in database!")).await?;
    return Ok(());
  }

  if let Some(server) = existing_server {
    let new_url = url.clone();
    let (ip, code) = if let Some(u) = new_url {
      match extract_ip_and_code(&u) {
        Some(e) => (e.ip, e.code),
        None => {
          ctx.reply(URL_EXTRACTION_FAILED).await?;
          return Ok(());
        }
      }
    } else {
      (server.ip, server.code)
    };

    let new_password = password.unwrap_or(server.game_password);
    let new_active = active.unwrap_or(server.is_active);

    match MpServers::update_server(&ctx.data().postgres, name.clone(), new_active, ip, code, new_password).await {
      Ok(_) => {
        ctx.data().redis.del(TASK_NAME).await?;
        ctx.reply(format!("**{name}**'s information successfully updated!")).await?;
      },
      Err(y) => {
        ctx.reply(format!("Error while updating server: {y}")).await?;
      }
    }
  }

  Ok(())
}
