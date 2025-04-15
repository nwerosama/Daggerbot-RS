use crate::{
  BotError,
  controllers::sql::{
    ProhibitedUrls,
    ProhibitedWords,
    Sanctions
  },
  internals::{
    config::BINARY_PROPERTIES,
    utils::format_duration
  }
};

use {
  parse_duration::parse,
  poise::{
    CreateReply,
    serenity_prelude::{
      AutocompleteChoice,
      CreateAttachment,
      CreateAutocompleteResponse,
      CreateEmbed,
      CreateMessage,
      EditMessage,
      GenericChannelId,
      GetMessages,
      Member,
      Mentionable,
      Timestamp,
      User
    }
  },
  std::time::{
    Duration,
    SystemTime,
    UNIX_EPOCH
  },
  tokio::{
    fs::File,
    io::AsyncWriteExt
  }
};

// If action is done through Discord's mod tools or AutoMod,
// the ActionType value should have "External" in front of the action name.
// and we grab the partial data from the audit log and add new entry to sanctions table.
// We need to ignore the action done by this bot and backup moderation bot (Dyno) as well
// if we want to gather info from external source and not actioned internally.
//
// We can rely on GuildBanAddition and GuildBanRemoval gateway events
// to make this idea work and manage the sanctions table accordingly.
#[derive(Debug, Clone)]
pub enum ActionType {
  Ban,
  Softban,
  Unban,
  Warn,
  Kick,
  Mute,
  Unmute
}

impl std::fmt::Display for ActionType {
  fn fmt(
    &self,
    f: &mut std::fmt::Formatter<'_>
  ) -> std::fmt::Result {
    let action = match self {
      Self::Ban => "Ban",
      Self::Softban => "Softban",
      Self::Unban => "Unban",
      Self::Warn => "Warn",
      Self::Kick => "Kick",
      Self::Mute => "Mute",
      Self::Unmute => "Unmute"
    };

    write!(f, "{action}")
  }
}

pub enum Target {
  User(User),
  Member(Member)
}

pub enum LogChannel {
  BansAndKicks,
  BotLog
}

impl LogChannel {
  fn id(&self) -> u64 {
    match self {
      Self::BansAndKicks => BINARY_PROPERTIES.bans_kicks_log,
      Self::BotLog => BINARY_PROPERTIES.bot_log
    }
  }

  pub fn to_discord(&self) -> GenericChannelId { GenericChannelId::new(self.id()) }
}

pub async fn generate_id(pool: &sqlx::PgPool) -> Result<i32, BotError> {
  let q: Option<i32> = sqlx::query_scalar("SELECT MAX(case_id) FROM sanctions").fetch_one(pool).await?;

  match q {
    Some(id) => Ok(id + 1),
    None => Ok(1)
  }
}

fn is_bkl(ctx: super::PoiseContext<'_>) -> bool { ctx.channel_id().get() == BINARY_PROPERTIES.bans_kicks_log }

/// Send a notification to a user about a moderation action
pub async fn send_notification(
  ctx: &super::PoiseContext<'_>,
  target: &Target,
  action: &ActionType,
  reason: &str,
  case_id: i32,
  duration: Option<u64>
) -> Result<bool, BotError> {
  let user = match target {
    Target::User(user) => user,
    Target::Member(mem) => &mem.user
  };

  let description = format!(
    "You've been **{}** in **{}** for:```\n{reason}\n```",
    match action {
      ActionType::Ban => "banned",
      ActionType::Kick => "kicked",
      ActionType::Mute => "timed out",
      ActionType::Softban => "softbanned",
      ActionType::Warn => "warned",
      _ => ""
    },
    ctx.guild_id().unwrap().to_partial_guild(ctx.http()).await?.name
  );

  let mut fields = vec![("Case ID", case_id.to_string(), true)];

  if let Some(duration) = duration {
    let d = parse_duration::parse(&duration.to_string()).unwrap();
    fields.insert(1, ("Duration", format_duration(d.as_secs()), true));
  }

  let embed = CreateEmbed::new()
    .color(BINARY_PROPERTIES.embed_colors.primary)
    .title("Notice from moderation team")
    .fields(fields)
    .description(description);

  match user.id.direct_message(ctx.http(), CreateMessage::new().embed(embed)).await {
    Ok(_) => Ok(true),
    Err(e) => {
      eprintln!("[moderation::send_notification] Send DM failed with error: {e}");
      Ok(false)
    }
  }
}

fn formate_dm_status(b: bool) -> String {
  match b {
    true => "dm sent".to_string(),
    false => "dm failed".to_string()
  }
}

#[allow(clippy::too_many_arguments)]
async fn log_entry(
  ctx: super::PoiseContext<'_>,
  case_id: i32,
  moderator: Member,
  target: Target,
  action: ActionType,
  reason: &str,
  duration: Option<i64>,
  channel: LogChannel
) -> Result<bool, BotError> {
  let db = ctx.data().postgres.clone();
  let existing_sanctions = Sanctions::load_data(&db, case_id).await?;

  if existing_sanctions.is_some() {
    eprintln!(
      "Moderation[Error] {} tried to create a case entry but Postgres already has it, dropping this one!",
      moderator.user.name
    );
    return Ok(false)
  }

  let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("System time is lagging behind or is in the future")
    .as_secs() as i64;

  let target = match target {
    Target::User(user) => user,
    Target::Member(member) => member.user
  };

  let sanctions = Sanctions {
    case_id,
    case_type: action.to_string(),
    member_name: target.name.clone().into(),
    member_id: target.id.to_string(),
    moderator_name: moderator.user.name.clone().into(),
    moderator_id: moderator.user.id.to_string(),
    timestamp,
    end_time: None,
    duration,
    reason: reason.into()
  };

  let mut fields = vec![
    ("User", format!("{}\n{}\n`{}`", target.name.as_str(), target.mention(), target.id), true),
    (
      "Moderator",
      format!("{}\n{}\n`{}`", moderator.user.name.as_str(), moderator.mention(), moderator.user.id),
      true
    ),
    ("\u{200B}", "\u{200B}".to_string(), true),
    ("Reason", reason.to_string(), true),
  ];

  if duration.is_some() {
    let d = parse(&duration.unwrap().to_string()).unwrap();
    fields.push(("Duration", format_duration(d.as_secs()), false));
  }

  let embed = CreateEmbed::default()
    .color(BINARY_PROPERTIES.embed_colors.primary)
    .title(format!("{action} | Case #{case_id}"))
    .timestamp(Timestamp::from_unix_timestamp(sanctions.timestamp).unwrap())
    .fields(fields);

  match GenericChannelId::new(channel.id())
    .send_message(ctx.http(), CreateMessage::new().embed(embed))
    .await
  {
    Ok(_) => {
      sanctions.create(&db).await?;
      Ok(true)
    },
    Err(e) => {
      eprintln!("Moderation[Error] err sending message: {e}");
      Ok(false)
    }
  }
}

/// Ban a member from the server
#[poise::command(slash_command, default_member_permissions = "BAN_MEMBERS")]
pub async fn ban(
  ctx: super::PoiseContext<'_>,
  #[description = "The member to ban"] member: Member,
  #[description = "The reason for the ban"] reason: String,
  #[description = "Should the ban be soft? (ban and unban immediately)"] soft: Option<bool>
) -> Result<(), BotError> {
  let is_soft = soft.unwrap_or(false);
  let guild_id = ctx.guild_id().unwrap();
  let user_id = member.user.id;
  let case_id = generate_id(&ctx.data().postgres).await?;

  let (action_type, action_verb) = if is_soft {
    (ActionType::Softban, "softban")
  } else {
    (ActionType::Ban, "ban")
  };

  let notify_user = send_notification(&ctx, &Target::Member(member.clone()), &action_type, &reason, case_id, None).await?;

  match guild_id.ban(ctx.http(), user_id, 86400, Some(&format!("{reason} | #{case_id}"))).await {
    Ok(_) => {
      if is_soft {
        if let Err(e) = guild_id.unban(ctx.http(), user_id, Some(&format!("{reason} | #{case_id}"))).await {
          eprintln!("Error unbanning user after softban: {e}");
          ctx.reply(format!("Softbanned but failed to unban:\n`{e}`")).await?;
          return Ok(());
        }
      }

      ctx
        .send(
          CreateReply::new()
            .content(format!(
              "{} now {action_verb}ned for `{reason}` ({})",
              member.user.name,
              formate_dm_status(notify_user)
            ))
            .ephemeral(is_bkl(ctx))
        )
        .await?;

      if !log_entry(
        ctx,
        case_id,
        ctx.author_member().await.unwrap_or_default().into_owned(),
        Target::Member(member.clone()),
        action_type,
        &reason,
        None,
        LogChannel::BansAndKicks
      )
      .await?
      {
        ctx
          .send(
            CreateReply::new()
              .content("Sorry, be faster next time as this case entry already exists!")
              .ephemeral(true)
          )
          .await?;
        return Ok(());
      }
    },
    Err(e) => {
      eprintln!("Error {action_verb}ning user: {e}");
      ctx.reply(format!("Could not {action_verb} the user:\n`{e}`")).await?;
      return Ok(());
    }
  }

  Ok(())
}

/// Kick a member from the server
#[poise::command(slash_command, default_member_permissions = "KICK_MEMBERS")]
pub async fn kick(
  ctx: super::PoiseContext<'_>,
  #[description = "The member to kick"] member: Member,
  #[description = "The reason for the kick"] reason: String
) -> Result<(), BotError> {
  let case_id = generate_id(&ctx.data().postgres).await?;

  let notify_user = send_notification(&ctx, &Target::Member(member.clone()), &ActionType::Kick, &reason, case_id, None).await?;

  match member.kick(ctx.http(), Some(&format!("{reason} | #{case_id}"))).await {
    Ok(_) => {
      ctx
        .send(
          CreateReply::new()
            .content(format!(
              "{} now kicked for `{reason}` ({})",
              member.user.name,
              formate_dm_status(notify_user)
            ))
            .ephemeral(is_bkl(ctx))
        )
        .await?;

      if !log_entry(
        ctx,
        case_id,
        ctx.author_member().await.unwrap_or_default().into_owned(),
        Target::Member(member.clone()),
        ActionType::Kick,
        &reason,
        None,
        LogChannel::BansAndKicks
      )
      .await?
      {
        ctx
          .send(
            CreateReply::new()
              .content("Sorry, be faster next time as this case entry already exists!")
              .ephemeral(true)
          )
          .await?;
        return Ok(());
      }
    },
    Err(e) => {
      eprintln!("Error kicking user: {e}");
      ctx.reply(format!("Could not kick the user:\n`{e}`")).await?;
    }
  }

  Ok(())
}

/// Revoke a ban from a member
#[poise::command(slash_command, default_member_permissions = "BAN_MEMBERS")]
pub async fn unban(
  ctx: super::PoiseContext<'_>,
  #[description = "The member to revoke a ban on"] user: User,
  #[description = "The reason for the unban"] reason: String
) -> Result<(), BotError> {
  let case_id = generate_id(&ctx.data().postgres).await?;
  match ctx
    .guild_id()
    .unwrap()
    .unban(ctx.http(), user.id, Some(&format!("{reason} | #{case_id}")))
    .await
  {
    Ok(_) => {
      ctx.reply(format!("{} now unbanned for `{reason}`", user.name)).await?;

      if !log_entry(
        ctx,
        case_id,
        ctx.author_member().await.unwrap_or_default().into_owned(),
        Target::User(user.clone()),
        ActionType::Unban,
        &reason,
        None,
        LogChannel::BotLog
      )
      .await?
      {
        ctx
          .send(
            CreateReply::new()
              .content("Sorry, be faster next time as this case entry already exists!")
              .ephemeral(true)
          )
          .await?;
        return Ok(());
      }
    },
    Err(e) => {
      eprintln!("Error revoking the ban: {e}");
      ctx.reply(format!("Could not unban the user:\n`{e}`")).await?;
    }
  }

  Ok(())
}

/// Warn a member
#[poise::command(slash_command, default_member_permissions = "MODERATE_MEMBERS")]
pub async fn warn(
  ctx: super::PoiseContext<'_>,
  #[description = "The member to warn"] member: Member,
  #[description = "The reason for the warning"] reason: String
) -> Result<(), BotError> {
  let case_id = generate_id(&ctx.data().postgres).await?;
  let notify_user = send_notification(&ctx, &Target::Member(member.clone()), &ActionType::Warn, &reason, case_id, None).await?;

  match log_entry(
    ctx,
    case_id,
    ctx.author_member().await.unwrap_or_default().into_owned(),
    Target::Member(member.clone()),
    ActionType::Warn,
    &reason,
    None,
    LogChannel::BotLog
  )
  .await
  {
    Ok(_) => {
      ctx
        .reply(format!(
          "{} now warned for `{reason}` ({})",
          member.user.name,
          formate_dm_status(notify_user)
        ))
        .await?;
    },
    Err(e) => {
      eprintln!("Error warning user: {e}");
      ctx
        .send(
          CreateReply::new()
            .content("Sorry, be faster next time as this case entry already exists!")
            .ephemeral(true)
        )
        .await?;
      return Ok(());
    }
  };

  Ok(())
}

/// Send the member to the timeout corner
#[poise::command(slash_command, default_member_permissions = "MODERATE_MEMBERS")]
pub async fn mute(
  ctx: super::PoiseContext<'_>,
  #[description = "The member to timeout"] mut member: Member,
  #[description = "Timeout duration"] duration: String,
  #[description = "The reason for the timeout"] reason: String
) -> Result<(), BotError> {
  let mut d = match parse(&duration) {
    Ok(d) => d,
    Err(e) => {
      eprintln!("Moderation[Timeout:Error] {e}");
      ctx.reply("Could not parse the duration, try again").await?;
      return Ok(());
    }
  };

  const MAX_TIMEOUT_SECONDS: u64 = 2419200; // 28 days in seconds
  if d.as_secs() > MAX_TIMEOUT_SECONDS {
    d = Duration::from_secs(MAX_TIMEOUT_SECONDS);
    ctx
      .send(CreateReply::new().content("Duration has been adjusted due to your input exceeding the maximum duration of 28 days!"))
      .await?;
  }

  let dur = match Timestamp::from_unix_timestamp(d.as_secs() as i64 + SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64) {
    Ok(d) => d,
    Err(e) => {
      eprintln!("Moderation[Timeout:Error] {e}");
      ctx
        .reply(format!("Timestamp didn't parse correctly and Discord sent an error back.\n`{e}`"))
        .await?;
      return Ok(());
    }
  };

  let case_id = generate_id(&ctx.data().postgres).await?;

  let notify_user = send_notification(
    &ctx,
    &Target::Member(member.clone()),
    &ActionType::Mute,
    &reason,
    case_id,
    Some(d.as_secs())
  )
  .await?;

  match member.disable_communication_until(ctx.http(), dur).await {
    Ok(_) => {
      ctx
        .reply(format!(
          "{} now muted for `{reason}` ({})",
          member.user.name,
          formate_dm_status(notify_user)
        ))
        .await?;
      if !log_entry(
        ctx,
        case_id,
        ctx.author_member().await.unwrap_or_default().into_owned(),
        Target::Member(member.clone()),
        ActionType::Mute,
        &reason,
        Some(d.as_secs() as i64),
        LogChannel::BotLog
      )
      .await?
      {
        ctx
          .send(
            CreateReply::new()
              .content("Sorry, be faster next time as this case entry already exists!")
              .ephemeral(true)
          )
          .await?;
        return Ok(());
      }
    },
    Err(e) => {
      eprintln!("Error timing out user: {e}");
      ctx.reply(format!("Could not timeout the user:\n`{e}`")).await?;
      return Ok(());
    }
  }

  Ok(())
}

/// Revoke the mute from a member
#[poise::command(slash_command, default_member_permissions = "MODERATE_MEMBERS")]
pub async fn unmute(
  ctx: super::PoiseContext<'_>,
  #[description = "The member to remove timeout from"] mut member: Member,
  #[description = "The reason for the timeout removal"] reason: String
) -> Result<(), BotError> {
  match member.enable_communication(ctx.http()).await {
    Ok(_) => {
      ctx.reply(format!("Revoked {}'s timeout for `{reason}`", member.user.name)).await?;

      if !log_entry(
        ctx,
        generate_id(&ctx.data().postgres).await?,
        ctx.author_member().await.unwrap_or_default().into_owned(),
        Target::Member(member.clone()),
        ActionType::Unmute,
        &reason,
        None,
        LogChannel::BotLog
      )
      .await?
      {
        ctx
          .send(
            CreateReply::new()
              .content("Sorry, be faster next time as this case entry already exists!")
              .ephemeral(true)
          )
          .await?;
        return Ok(());
      }
    },
    Err(e) => {
      eprintln!("Error removing the timeout from user: {e}");
      ctx.reply(format!("Could not unmute the user:\n`{e}`")).await?;
      return Ok(());
    }
  };

  Ok(())
}

/// Manage the cases in the database
#[poise::command(slash_command, subcommands("view", "update"), default_member_permissions = "MANAGE_MESSAGES")]
pub async fn case(_: super::PoiseContext<'_>) -> Result<(), BotError> { Ok(()) }

async fn ac_cases<'a>(
  ctx: super::PoiseContext<'a>,
  partial: &'a str
) -> CreateAutocompleteResponse<'a> {
  let cases = Sanctions::get_cases(&ctx.data().postgres).await.unwrap();
  let mut filtered: Vec<_> = cases
    .iter()
    .filter(|c| {
      let p_low = partial.trim().trim_start_matches('#').to_lowercase();
      if p_low.is_empty() {
        true
      } else {
        c.case_id.to_string().starts_with(&p_low) || c.member_id.to_lowercase().starts_with(&p_low)
      }
    })
    .collect();

  filtered.sort_by(|a, b| b.case_id.cmp(&a.case_id));

  CreateAutocompleteResponse::new().set_choices(
    filtered
      .into_iter()
      .take(25)
      .map(|c| AutocompleteChoice::new(format!("#{} - {} ({})", c.case_id, c.case_type, c.member_name), c.case_id.to_string()))
      .collect::<Vec<AutocompleteChoice>>()
  )
}

/// View the case entry in the database
#[poise::command(slash_command)]
async fn view(
  ctx: super::PoiseContext<'_>,
  #[description = "Filter the search by Member ID or Case ID"]
  #[autocomplete = "ac_cases"]
  case_id: i32
) -> Result<(), BotError> {
  let db = ctx.data().postgres.clone();
  let sanctions_data = Sanctions::load_data(&db, case_id).await?;

  fn mention_user(user_id: String) -> String {
    let user_id = user_id.parse::<u64>().unwrap();
    format!("<@{user_id}>")
  }

  match sanctions_data {
    Some(sanctions) => {
      let mut fields = vec![
        (
          "User",
          format!(
            "{}\n{}\n`{}`",
            sanctions.member_name,
            mention_user(sanctions.member_id.clone()),
            sanctions.member_id
          ),
          true
        ),
        (
          "Moderator",
          format!(
            "{}\n{}\n`{}`",
            sanctions.moderator_name,
            mention_user(sanctions.moderator_id.clone()),
            sanctions.moderator_id
          ),
          true
        ),
        ("\u{200B}", "\u{200B}".to_string(), true),
        ("Reason", sanctions.reason, true),
      ];

      if sanctions.duration.is_some() {
        let d = parse(&sanctions.duration.unwrap().to_string()).unwrap();
        fields.push(("Duration", format_duration(d.as_secs()), false));
      }

      let embed = CreateEmbed::default()
        .color(BINARY_PROPERTIES.embed_colors.primary)
        .title(format!("{} | Case #{case_id}", sanctions.case_type))
        .timestamp(Timestamp::from_unix_timestamp(sanctions.timestamp).unwrap())
        .fields(fields);

      ctx.send(CreateReply::new().embed(embed)).await?;
    },
    None => {
      ctx.reply("Case not found in database").await?;
    }
  }

  Ok(())
}

/// Update existing case entry with new reason
#[poise::command(slash_command)]
async fn update(
  ctx: super::PoiseContext<'_>,
  #[description = "Filter the search by Member ID or Case ID"]
  #[autocomplete = "ac_cases"]
  case_id: i32,
  #[description = "New reason for the case"] reason: String
) -> Result<(), BotError> {
  ctx.defer().await?;

  let db = ctx.data().postgres.clone();

  if let Some(case) = Sanctions::load_data(&db, case_id).await? {
    // remove this when automod's core is rewritten
    if case.moderator_id == ctx.cache().current_user().id.to_string() {
      ctx
        .send(
          CreateReply::new().embed(
            CreateEmbed::new()
              .color(BINARY_PROPERTIES.embed_colors.red)
              .title("Case not updated")
              .description(
                [
                  "Cannot edit Automoderator's case entry;-",
                  "*Nwero: Last time someone edited the automod's case, it broke the internal code for some reason so this is set in place until it \
                   is fixed in a rewrite!*"
                ]
                .join("\n")
              )
          )
        )
        .await?;
      return Ok(())
    }

    sqlx::query("UPDATE sanctions SET reason = $1 WHERE case_id = $2")
      .bind(reason.clone())
      .bind(case_id)
      .execute(&db)
      .await?;

    let log_channels = [LogChannel::BansAndKicks.to_discord(), LogChannel::BotLog.to_discord()];

    for channel_id in log_channels {
      if let Ok(channel) = ctx.http().get_channel(channel_id).await {
        if let Some(channel) = channel.guild() {
          let messages = channel.id.widen().messages(ctx.http(), GetMessages::default().limit(10)).await?;

          for mut message in messages {
            if let Some((Some(title), fields)) = message.embeds.first().map(|e| (e.title.clone(), e.fields.clone())) {
              if title.contains(&format!("Case #{case_id}")) {
                let original = message.embeds.first().unwrap();
                let mut new_embed = CreateEmbed::new().title(title).color(original.colour.unwrap());

                for field in fields {
                  let field_value = if field.name == "Reason" {
                    reason.clone()
                  } else {
                    field.value.to_string()
                  };
                  new_embed = new_embed.field(field.name, field_value, field.inline);
                }

                message.edit(ctx.http(), EditMessage::new().embed(new_embed)).await?;
              }
            }
          }
        }
      }
    }

    ctx
      .send(
        CreateReply::new().embed(
          CreateEmbed::new()
            .color(BINARY_PROPERTIES.embed_colors.green)
            .title("Case updated")
            .description(format!("Case #{case_id} has been successfully updated with new reason:\n**{reason}**"))
        )
      )
      .await?;
  } else {
    ctx
      .send(
        CreateReply::new().embed(
          CreateEmbed::new()
            .color(BINARY_PROPERTIES.embed_colors.red)
            .title("Case not updated")
            .description(format!("Stop using your imagination, #{case_id} doesn't even exist!"))
        )
      )
      .await?;
  }

  Ok(())
}

enum ProhibitedType {
  Word,
  Url
}

enum CmdOperation {
  List,
  Manage(String)
}

async fn mpl(
  ctx: super::PoiseContext<'_>,
  item_type: ProhibitedType,
  operation: CmdOperation
) -> Result<(), BotError> {
  let db = ctx.data().postgres.clone();

  match operation {
    CmdOperation::Manage(input) => {
      let (normalized_input, original_input) = match item_type {
        ProhibitedType::Word => (input.clone(), input.clone()),
        ProhibitedType::Url => {
          let re = regex::Regex::new(r"(?i)^(?:https?://)?(?:www\.)?([a-zA-Z0-9][a-zA-Z0-9-]*(?:\.[a-zA-Z0-9-]+)+)/?.*$").unwrap();
          let normalized = match re.captures(&input) {
            Some(caps) => caps.get(1).map(|m| m.as_str().to_lowercase()),
            None => None
          }
          .unwrap_or_default();

          if normalized.is_empty() {
            ctx.reply("Bad input! Expected `example.com` or `https://example.com`").await?;
            return Ok(());
          }

          (normalized, input)
        }
      };

      match item_type {
        ProhibitedType::Word => {
          if ProhibitedWords::get_words(&db).await?.iter().any(|w| w.word == normalized_input) {
            ProhibitedWords::remove_word(&db, &normalized_input).await?;
            ctx.reply(format!("Removed `{normalized_input}` from the list!")).await?;
          } else {
            ProhibitedWords::add_word(&db, &normalized_input).await?;
            ctx.reply(format!("Added `{normalized_input}` to the list!")).await?;
          }
        },
        ProhibitedType::Url => {
          if ProhibitedUrls::get_urls(&db).await?.iter().any(|u| u.url == normalized_input) {
            ProhibitedUrls::remove_url(&db, &normalized_input).await?;
            ctx.reply(format!("Removed `{normalized_input}` from the list!")).await?;
          } else {
            ProhibitedUrls::add_url(&db, &normalized_input).await?;
            ctx
              .reply(format!(
                "Added `{normalized_input}` to the list!{}",
                if normalized_input != original_input {
                  format!(" (normalized from `{original_input}`)")
                } else {
                  String::new()
                }
              ))
              .await?;
          }
        },
      }
    },
    CmdOperation::List => {
      ctx.defer().await?;
      let pw = "pw.txt";
      let pu = "pu.txt";

      match item_type {
        ProhibitedType::Word => {
          let words = ProhibitedWords::get_words(&db).await?;
          if words.is_empty() {
            ctx.reply("No prohibited words found").await?;
            return Ok(());
          }

          let mut sorted_words: Vec<String> = words.into_iter().map(|w| w.word).collect();
          sorted_words.sort();

          let content = format!("Prohibited words\n- Total: {}\n\n{}", sorted_words.len(), sorted_words.join("\n"));

          let mut temp_file = File::create(pw).await?;
          temp_file.write_all(content.as_bytes()).await?;

          ctx.send(CreateReply::new().attachment(CreateAttachment::path(pw).await?)).await?;

          tokio::fs::remove_file(pw).await?;
        },
        ProhibitedType::Url => {
          let urls = ProhibitedUrls::get_urls(&db).await?;
          if urls.is_empty() {
            ctx.reply("No prohibited URLs found").await?;
            return Ok(());
          }

          let mut sorted_urls: Vec<String> = urls.into_iter().map(|u| u.url).collect();
          sorted_urls.sort();

          let content = format!("Prohibited urls\n- Total: {}\n\n{}", sorted_urls.len(), sorted_urls.join("\n"));

          let mut temp_file = File::create(pu).await?;
          temp_file.write_all(content.as_bytes()).await?;

          ctx.send(CreateReply::new().attachment(CreateAttachment::path(pu).await?)).await?;

          tokio::fs::remove_file(pu).await?;
        }
      }
    }
  }

  Ok(())
}

/// Prohibited words management
#[poise::command(slash_command, subcommands("pwm", "pwl"), default_member_permissions = "ADMINISTRATOR")]
pub async fn pw(_: super::PoiseContext<'_>) -> Result<(), BotError> { Ok(()) }

/// Prohibited urls management
#[poise::command(slash_command, subcommands("pum", "pul"), default_member_permissions = "ADMINISTRATOR")]
pub async fn pu(_: super::PoiseContext<'_>) -> Result<(), BotError> { Ok(()) }

/// Add/remove a word to Automoderator's PW list
#[poise::command(slash_command, rename = "manage")]
async fn pwm(
  ctx: super::PoiseContext<'_>,
  #[description = "The word to be added or removed"] word: String
) -> Result<(), BotError> {
  mpl(ctx, ProhibitedType::Word, CmdOperation::Manage(word)).await
}

/// Add/remove a domain to Automoderator's PU list
#[poise::command(slash_command, rename = "manage")]
async fn pum(
  ctx: super::PoiseContext<'_>,
  #[description = "The domain to be added or removed"] url: String
) -> Result<(), BotError> {
  mpl(ctx, ProhibitedType::Url, CmdOperation::Manage(url)).await
}

/// Retrieve the Automoderator's PW list
#[poise::command(slash_command, rename = "list")]
async fn pwl(ctx: super::PoiseContext<'_>) -> Result<(), BotError> { mpl(ctx, ProhibitedType::Word, CmdOperation::List).await }

/// Retrieve the Automoderator's PU list
#[poise::command(slash_command, rename = "list")]
async fn pul(ctx: super::PoiseContext<'_>) -> Result<(), BotError> { mpl(ctx, ProhibitedType::Url, CmdOperation::List).await }
