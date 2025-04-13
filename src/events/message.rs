use crate::{
  BotData,
  Error,
  internals::{
    ansi::Color,
    config::BINARY_PROPERTIES,
    utils::format_timestamp
  }
};

use {
  lazy_static::lazy_static,
  poise::serenity_prelude::{
    Attachment,
    Context,
    CreateActionRow,
    CreateButton,
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
) -> Result<CachedMessage, Error> {
  let redis = &ctx.data::<BotData>().redis;
  let rkey = REDIS_MSG_KEY.replace("{{ message_id }}", msg_id.to_string().as_str());

  match redis.set(&rkey, &serde_json::to_string(&cached)?).await {
    Ok(_) => {
      #[cfg(not(feature = "production"))]
      println!("Message[Cache] Message cached successfully!");
      redis.expire(&rkey, 43200).await?; // 12 hours, extended from 4 hours due to Automod purposes
      Ok(cached)
    },
    Err(e) => {
      eprintln!("Message[Cache:Error] {e}");
      Err(Error::from(e))
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

async fn reusable_log(
  ctx: &Context,
  color: u32,
  author: &User,
  title: &str,
  fields: Vec<(&str, String, bool)>,
  evt_msg: Option<&Message>
) -> Result<(), Error> {
  for (_, v, _) in &fields {
    if v.len() > 1024 {
      println!("MessageLog[reusable_log] Embed field's value exceeds 1024 characters, not sending it");
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

  if title.contains("edited") {
    if let Some(msg) = evt_msg {
      message = message.components(vec![CreateActionRow::Buttons(Cow::Owned(vec![
        CreateButton::new_link(msg.link()).label("Jump!"),
      ]))]);
    }
  }

  match GenericChannelId::new(BINARY_PROPERTIES.bot_log).send_message(&ctx.http, message).await {
    Ok(_) => Ok(()),
    Err(e) => {
      eprintln!("MessageLog[Error] {e}");
      Err(Error::from(e))
    }
  }
}

async fn ignored_channels(
  ctx: &Context,
  channel_id: &u64
) -> sqlx::Result<bool> {
  let q = sqlx::query("SELECT * FROM settings WHERE $1 = ANY(logs_ignored_channels)")
    .bind(*channel_id as i64)
    .execute(&ctx.data_ref::<BotData>().postgres)
    .await;

  match q {
    Ok(r) => Ok(r.rows_affected() > 0),
    Err(e) => {
      eprintln!("IgnoredChannels[Error] {e}");
      Err(e)
    }
  }
}

pub async fn on_message_delete(
  ctx: &Context,
  channel_id: &GenericChannelId,
  deleted_message_id: &MessageId
) -> Result<(), Error> {
  if ignored_channels(ctx, &channel_id.get()).await? {
    return Ok(());
  }

  let redis = &ctx.data::<BotData>().redis;
  let rkey = REDIS_MSG_KEY.replace("{{ message_id }}", &deleted_message_id.to_string());

  let mut get_cached_msg: CachedMessage = match redis.get(&rkey).await {
    Ok(m) => {
      let msg = match m {
        Some(msg) => msg,
        None => {
          #[cfg(not(feature = "production"))]
          eprintln!("MessageDelete[Error] Message not found in cache");
          return Ok(());
        }
      };
      match serde_json::from_str(&msg) {
        Ok(c) => c,
        Err(e) => {
          eprintln!("MessageDelete[Deserialization:Error] {e}");
          return Ok(());
        }
      }
    },
    Err(e) => {
      eprintln!("MessageDelete[Error] {e}");
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
) -> Result<(), Error> {
  if event.message.author.bot() || ignored_channels(ctx, &event.message.channel_id.get()).await? {
    return Ok(());
  }

  let redis = &ctx.data::<BotData>().redis;
  let rkey = REDIS_MSG_KEY.replace("{{ message_id }}", &event.message.id.to_string());

  let mut get_cached_msg: CachedMessage = match redis.get(&rkey).await {
    Ok(m) => {
      let msg = match m {
        Some(msg) => msg,
        None => {
          #[cfg(not(feature = "production"))]
          eprintln!("MessageUpdate[Error] Message not found in cache");
          return Ok(());
        }
      };
      match serde_json::from_str(&msg) {
        Ok(c) => c,
        Err(e) => {
          eprintln!("MessageUpdate[Deserialization:Error] {e}");
          return Ok(());
        }
      }
    },
    Err(e) => {
      eprintln!("MessageUpdate[Error] {e}");
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

  let event_content = event.message.content.clone().to_string();
  let diffs = TextDiff::from_chars(get_cached_msg.content.as_str(), event_content.as_str());

  let mut content_old = String::new();
  let mut content_new = String::new();

  for diff in diffs.iter_all_changes() {
    match diff.tag() {
      ChangeTag::Equal => {
        content_old.push_str(diff.value());
        content_new.push_str(diff.value());
      },
      ChangeTag::Insert => {
        for ch in diff.value().chars() {
          content_new.push_str(&Color::Green.normal().paint(&ch.to_string()));
        }
      },
      ChangeTag::Delete => {
        for ch in diff.value().chars() {
          content_old.push_str(&Color::Red.normal().paint(&ch.to_string()));
        }
      },
    }
  }

  content_old = truncate_content(FixedString::from_str_trunc(&content_old)).to_string();
  content_new = truncate_content(FixedString::from_str_trunc(&content_new)).to_string();

  reusable_log(
    ctx,
    BINARY_PROPERTIES.embed_colors.primary,
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
) -> Result<(), Error> {
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
  {
    use crate::controllers::automod::Automoderator;
    let automod = Automoderator::new(&ctx.data::<BotData>().postgres, ctx.data::<BotData>().redis.clone()).await?;
    automod.process_message(ctx, new_message).await?;
  }

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

pub async fn on_message_lua(
  ctx: &Context,
  new_message: &Message
) -> Result<(), Error> {
  let bridge = ctx.data_ref::<BotData>().serenity_bridge.clone();

  for plugin in &["Message", "MsgResponse"] {
    bridge.register_plugin(plugin)?;
  }

  if new_message.author.bot() || new_message.guild_id != Some(GuildId::new(BINARY_PROPERTIES.guild_id)) {
    return Ok(());
  }

  if (!new_message.attachments.is_empty() || !new_message.sticker_items.is_empty()) && new_message.content.is_empty() {
    return Ok(());
  }

  let message_table = bridge.build_message_table(new_message)?;

  let multifarm_pw: mlua::Function = bridge.lua.globals().get("MFPassword")?;
  multifarm_pw.call::<()>(message_table.clone())?;

  // Lua version of a famous ResponseModule from v3 (TypeScript)
  if new_message.channel_id == GenericChannelId::new(BINARY_PROPERTIES.general_chat) {
    let response: mlua::Table = bridge.lua.globals().get("Response")?;
    let outgoing_arrays: mlua::Function = response.get("outgoingArrays")?;
    outgoing_arrays.call::<()>(message_table.clone())?;

    let tod = ["morning", "afternoon", "evening", "night"];
    for keyword in tod.iter() {
      let respond: mlua::Function = response.get("respond")?;
      respond.call::<()>((message_table.clone(), *keyword))?;
    }
  }

  Ok(())
}

async fn on_message_dm(
  ctx: &Context,
  new_message: &Message
) -> Result<(), Error> {
  let (name, dname) = { (new_message.author.name.clone(), new_message.author.display_name()) };
  let content = new_message.content.clone();

  GenericChannelId::new(BINARY_PROPERTIES.bot_log)
    .send_message(
      &ctx.http,
      CreateMessage::new().content(format!("Relayed the DM from **{name}** (**{dname}**)```\n{content}\n```"))
    )
    .await?;

  Ok(())
}
