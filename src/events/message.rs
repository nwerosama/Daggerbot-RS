mod autoresponder;

use {
  crate::{
    BotData,
    BotError,
    BotResult,
    internals::config::BINARY_PROPERTIES
  },
  asahi::{
    error,
    utils::{
      ansi,
      format_timestamp
    }
  },
  autoresponder::{
    Keywords,
    PREFIXES,
    SUFFIXES,
    match_keywords
  },
  lazy_static::lazy_static,
  poise::serenity_prelude::{
    Attachment,
    ButtonStyle,
    Context,
    CreateActionRow,
    CreateButton,
    CreateComponent,
    CreateEmbed,
    CreateEmbedAuthor,
    CreateMessage,
    GenericChannelId,
    GuildId,
    Mentionable,
    Message,
    MessageId,
    MessageReference,
    MessageUpdateEvent,
    Poll,
    Timestamp,
    User,
    small_fixed_array::FixedString
  },
  rand::seq::IndexedRandom,
  regex::Regex,
  serde::{
    Deserialize,
    Serialize
  },
  similar::{
    ChangeTag,
    TextDiff
  },
  std::borrow::Cow
};

#[derive(Clone, Serialize, Deserialize)]
struct CachedMessage {
  content:     FixedString<u16>,
  attachments: Vec<Attachment>,
  poll:        Option<Poll>,
  sent_at:     i64,
  reference:   Option<MessageReference>,
  author:      User
}

/// Redis cache key for message events<br>
const REDIS_MSG_KEY: &str = "Discord:Message:{{ message_id }}";

lazy_static! {
  static ref ATTACHMENT_TXT: FixedString<u16> = FixedString::from_str_trunc("(Attachment)");
  static ref MSG_FORWARDED_TXT: FixedString<u16> = FixedString::from_str_trunc("(Forwarded message)");
}

async fn store_msg_cache(
  ctx: &Context,
  cached: CachedMessage,
  msg_id: MessageId
) -> BotResult<CachedMessage> {
  let redis = &ctx.data::<BotData>().redis;
  let rkey = REDIS_MSG_KEY.replace("{{ message_id }}", msg_id.to_string().as_str());

  match redis.set(&rkey, &serde_json::to_string(&cached)?).await {
    Ok(_) => {
      #[cfg(not(feature = "production"))]
      asahi::debug!("Message cached successfully!");
      redis.expire(&rkey, 43200).await?; // 12 hours
      Ok(cached)
    },
    Err(e) => {
      error!("Message failed to cache: {e}");
      Err(BotError::from(e))
    }
  }
}

fn truncate_content(s: FixedString<u16>) -> FixedString<u16> {
  if s.len() >= 1020 {
    FixedString::from_str_trunc(&format!("{}...", &s[..1000]))
  } else {
    s
  }
}

#[cfg(feature = "automod")]
async fn use_automod(
  ctx: &Context,
  msg: &Message
) -> BotResult {
  use crate::controllers::automod::Automoderator;
  let automod = Automoderator::new(&ctx.data::<BotData>().postgres, ctx.data::<BotData>().redis.clone(), ctx.http.clone())
    .await
    .expect("failed to initialize automod");
  automod.process_message(ctx, msg).await.expect("automod's process_message failed");
  Ok(())
}

async fn reusable_log(
  ctx: &Context,
  color: u32,
  author: &User,
  title: &str,
  fields: Vec<(&str, String, bool)>,
  evt_msg: Option<&Message>
) -> BotResult {
  for (_, v, _) in &fields {
    if v.len() > 1024 {
      error!("Embed field's value exceeds 1024 characters, not sending it");
      return Ok(())
    }
  }

  let mut message = CreateMessage::new().embed(
    CreateEmbed::new()
      .color(color)
      .author(CreateEmbedAuthor::new(format!("Author: {} ({})", author.name, author.id)).icon_url(author.face()))
      .title(title)
      .fields(fields)
      .timestamp(Timestamp::now())
  );

  if title.contains("edited")
    && let Some(msg) = evt_msg
  {
    message = message.components(vec![CreateComponent::ActionRow(CreateActionRow::Buttons(Cow::Owned(vec![
      CreateButton::new_link(msg.link().to_string()).label("Jump!"),
    ])))]);
  }

  match GenericChannelId::new(BINARY_PROPERTIES.bot_log).send_message(&ctx.http, message).await {
    Ok(_) => Ok(()),
    Err(e) => {
      error!("Log failed to send due to error: {e}");
      Err(BotError::from(e))
    }
  }
}

async fn ignored_channels(
  ctx: &Context,
  channel_id: &u64
) -> sqlx::Result<bool> {
  let q = sqlx::query!("SELECT * FROM settings WHERE $1 = ANY(logs_ignored_channels)", *channel_id as i64)
    .fetch_optional(&ctx.data_ref::<BotData>().postgres)
    .await;

  match q {
    Ok(Some(_)) => Ok(true),
    Ok(None) => Ok(false),
    Err(e) => {
      error!("Ignored channels error: {e}");
      Err(e)
    }
  }
}

pub async fn on_message_delete(
  ctx: &Context,
  channel_id: &GenericChannelId,
  deleted_message_id: &MessageId
) -> BotResult {
  if ignored_channels(ctx, &channel_id.get()).await? {
    return Ok(());
  }

  let redis = &ctx.data::<BotData>().redis;
  let rkey = REDIS_MSG_KEY.replace("{{ message_id }}", &deleted_message_id.to_string());

  let mut get_cached_msg: CachedMessage = match redis.get(&rkey).await {
    Ok(m) => {
      let msg = match m {
        Some(msg) => msg,
        None => return Ok(())
      };
      match serde_json::from_str(&msg) {
        Ok(c) => c,
        Err(e) => {
          error!("MessageDelete deserialization error: {e}");
          return Ok(());
        }
      }
    },
    Err(e) => {
      error!("(MessageDelete) Unknown error: {e}");
      return Ok(());
    }
  };

  if get_cached_msg.author.bot() {
    return Ok(());
  }

  match (get_cached_msg.content.is_empty(), &get_cached_msg.reference) {
    (true, None) => get_cached_msg.content = ATTACHMENT_TXT.clone(),
    (true, Some(_)) => get_cached_msg.content = MSG_FORWARDED_TXT.clone(),
    _ => ()
  }

  get_cached_msg.content = truncate_content(get_cached_msg.content);

  reusable_log(
    ctx,
    BINARY_PROPERTIES.embed_colors.red,
    &get_cached_msg.author,
    "Message deleted",
    vec![
      ("Content", format!("```\n{}\n```", get_cached_msg.content), false),
      ("Channel", format!("{}", channel_id.mention()), false),
      ("Sent at", format_timestamp(get_cached_msg.sent_at), false),
    ],
    None
  )
  .await?;

  redis.del(&rkey).await?;

  Ok(())
}

pub async fn on_message_update(
  ctx: &Context,
  event: &MessageUpdateEvent
) -> BotResult {
  if event.message.author.bot() || ignored_channels(ctx, &event.message.channel_id.get()).await? {
    return Ok(());
  }

  let redis = &ctx.data::<BotData>().redis;
  let rkey = REDIS_MSG_KEY.replace("{{ message_id }}", &event.message.id.to_string());

  let mut get_cached_msg: CachedMessage = match redis.get(&rkey).await {
    Ok(m) => {
      let msg = match m {
        Some(msg) => msg,
        None => return Ok(())
      };
      match serde_json::from_str(&msg) {
        Ok(c) => c,
        Err(e) => {
          error!("MessageUpdate deserialization error: {e}");
          return Ok(());
        }
      }
    },
    Err(e) => {
      error!("(MessageUpdate) Unknown error: {e}");
      return Ok(());
    }
  };

  match get_cached_msg.content.as_str() {
    "" => get_cached_msg.content = ATTACHMENT_TXT.clone(),
    content => match event.message.content.as_str() {
      c if content == c => return Ok(()),
      _ => ()
    }
  }

  if get_cached_msg.poll.is_some() || !get_cached_msg.attachments.is_empty() {
    return Ok(());
  }

  #[cfg(feature = "automod")]
  use_automod(ctx, &event.message).await?;

  let event_content = event.message.content.clone().to_string();
  let diffs = TextDiff::from_chars(get_cached_msg.content.as_str(), event_content.as_str());

  const ANSI_THRESHOLD: u16 = 1024;
  let total_length = get_cached_msg.content.len() + event_content.len() as u16;
  let use_ansi = total_length <= ANSI_THRESHOLD;

  let mut content_old = String::new();
  let mut content_new = String::new();

  for diff in diffs.iter_all_changes() {
    match diff.tag() {
      ChangeTag::Equal => {
        content_old.push_str(diff.value());
        content_new.push_str(diff.value());
      },
      ChangeTag::Insert => {
        if use_ansi {
          for ch in diff.value().chars() {
            content_new.push_str(&ansi::Green::NORMAL.paint(&ch.to_string()));
          }
        } else {
          content_new.push_str(diff.value());
        }
      },
      ChangeTag::Delete => {
        if use_ansi {
          for ch in diff.value().chars() {
            content_old.push_str(&ansi::Red::NORMAL.paint(&ch.to_string()));
          }
        } else {
          content_old.push_str(diff.value());
        }
      },
    }
  }

  content_old = truncate_content(FixedString::from_str_trunc(&content_old)).to_string();
  content_new = truncate_content(FixedString::from_str_trunc(&content_new)).to_string();

  reusable_log(
    ctx,
    BINARY_PROPERTIES.embed_colors.primary(),
    &get_cached_msg.author,
    "Message edited",
    vec![
      ("Old", format!("```ansi\n{content_old}\n```"), false),
      ("New", format!("```ansi\n{content_new}\n```"), false),
      ("Channel", format!("{}", event.message.channel_id.mention()), false),
      ("Sent at", format_timestamp(get_cached_msg.sent_at), false),
    ],
    Some(&event.message)
  )
  .await?;

  get_cached_msg.content = FixedString::from_str_trunc(&event.message.content);
  store_msg_cache(ctx, get_cached_msg, event.message.id).await?;

  Ok(())
}

pub async fn on_message(
  ctx: &Context,
  new_message: &Message
) -> BotResult {
  // We maintain our own cache for message events
  // since Serenity's cache gets sweeped once the
  // message is deleted/updated before we get a
  // chance to process the said event.

  if new_message.author.bot() {
    return Ok(());
  }

  if new_message.guild_id.is_none() {
    on_message_dm(ctx, new_message).await?;
  }

  if ignored_channels(ctx, &new_message.channel_id.get()).await? || new_message.guild_id != Some(GuildId::new(BINARY_PROPERTIES.guild_id)) {
    return Ok(());
  }

  #[cfg(feature = "automod")]
  use_automod(ctx, new_message).await?;

  let cached_message = CachedMessage {
    content:     new_message.content.clone(),
    attachments: new_message.attachments.clone().into_vec(),
    poll:        new_message.poll.clone().map(|p| *p),
    sent_at:     new_message.timestamp.timestamp(),
    reference:   new_message.message_reference.clone(),
    author:      new_message.author.clone()
  };

  store_msg_cache(ctx, cached_message, new_message.id).await?;

  Ok(())
}

pub async fn on_message_autores(
  ctx: &Context,
  new_message: &Message
) -> BotResult {
  if new_message.author.bot()
    || new_message.channel_id != GenericChannelId::new(BINARY_PROPERTIES.general_chat)
    || (!new_message.attachments.is_empty() || !new_message.sticker_items.is_empty()) && new_message.content.is_empty()
  {
    return Ok(());
  }

  let mut reply: Option<String> = None;

  let nick_or_global = new_message
    .member
    .as_ref()
    .and_then(|m| m.nick.clone())
    .or_else(|| new_message.author.global_name.clone())
    .unwrap_or_else(|| new_message.author.name.clone());

  'a: for k in [Keywords::Morning, Keywords::Afternoon, Keywords::Evening, Keywords::Night] {
    let prefix = PREFIXES.join("|");
    let suffix = SUFFIXES.iter().map(|s| regex::escape(s)).collect::<Vec<_>>().join("|");

    {
      let pattern = format!(r"^({prefix})?\s?{k}\s+({suffix})\b");
      let re = Regex::new(&pattern).unwrap();

      if re.is_match(&new_message.content.to_lowercase()) {
        let responses = match_keywords(k, nick_or_global.to_string());
        if let Some(response) = responses.choose(&mut rand::rng()) {
          reply = Some(response.to_owned());
        }

        break 'a;
      }
    }
  }

  if let Some(r) = reply {
    new_message.reply(&ctx.http, r).await.unwrap();
  }

  Ok(())
}

async fn on_message_dm(
  ctx: &Context,
  new_message: &Message
) -> BotResult {
  let (name, dname, uid) = {
    (
      new_message.author.name.clone(),
      new_message.author.display_name(),
      new_message.author.id.get()
    )
  };
  let content = new_message.content.clone();

  GenericChannelId::new(BINARY_PROPERTIES.bot_log)
    .send_message(
      &ctx.http,
      CreateMessage::new()
        .content(format!("Relayed the DM from **{dname}** (**{name}**)```\n{content}\n```"))
        .button(
          CreateButton::new(format!("dm-{uid}"))
            .label("Reply back")
            .emoji('📡')
            .style(ButtonStyle::Secondary)
        )
    )
    .await?;

  Ok(())
}
