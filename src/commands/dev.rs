use crate::{
  BotError,
  bridges::PLUGIN_DIR,
  controllers::sql::execute_schemas,
  events::ready::{
    Activity,
    TOML_FILE,
    TomlConfig
  }
};

use poise::{
  CreateReply,
  serenity_prelude::{
    ActivityData,
    Attachment,
    CreateAllowedMentions,
    GenericChannelId,
    builder::CreateMessage
  }
};

/// Developer commands
#[poise::command(
  slash_command,
  owners_only,
  subcommands("presence", "echo", "deploy", "schemas", "upload_plugin", "invite_data", "sql"),
  default_member_permissions = "MANAGE_GUILD"
)]
pub async fn dev(_: super::PoiseContext<'_>) -> Result<(), BotError> { Ok(()) }

/// Update bot's presence
#[poise::command(slash_command)]
async fn presence(
  ctx: super::PoiseContext<'_>,
  #[description = "Activity message to set"] name: String,
  #[description = "YouTube video to set"] video: String
) -> Result<(), BotError> {
  let mut presence_data = vec![];

  presence_data.push(format!("Name: **{name}**"));
  presence_data.push(format!("URL: `{video}`"));

  let toml_content = match std::fs::read_to_string(TOML_FILE) {
    Ok(c) => c,
    Err(y) => {
      ctx.reply(format!("{y}")).await?;
      return Ok(());
    }
  };

  let mut conf: TomlConfig = match toml::from_str(&toml_content) {
    Ok(c) => c,
    Err(y) => {
      ctx.reply(format!("{y}")).await?;
      return Ok(());
    }
  };

  conf.presence.activities = vec![Activity {
    name: name.clone(),
    url:  video.clone()
  }];

  let updated_toml = toml::to_string(&conf).expect("[TomlConfig] Failed to serialize TOML data");
  std::fs::write(TOML_FILE, updated_toml).expect("[TomlConfig] Failed to write to TOML file");

  ctx.reply(format!("Presence updated:\n{}", presence_data.join("\n"))).await?;
  ctx.serenity_context().set_activity(Some(ActivityData::streaming(name, video).unwrap()));

  Ok(())
}

/// Turn your message into a bot message
#[poise::command(slash_command)]
async fn echo(
  ctx: super::PoiseContext<'_>,
  #[description = "Message to be echoed as a bot"] message: String,
  #[description = "Channel to send this to"]
  #[channel_types("Text", "PublicThread", "PrivateThread")]
  channel: Option<GenericChannelId>
) -> Result<(), BotError> {
  let channel = match channel {
    Some(c) => c,
    None => ctx.channel_id()
  };

  match GenericChannelId::new(channel.get())
    .send_message(
      ctx.http(),
      CreateMessage::new()
        .content(message)
        .allowed_mentions(CreateAllowedMentions::new().empty_roles().empty_users())
    )
    .await
  {
    Ok(_) => {
      ctx.send(CreateReply::new().content("Sent!").ephemeral(true)).await?;
    },
    Err(y) => {
      ctx.send(CreateReply::new().content(format!("Failed... `{y}`")).ephemeral(true)).await?;
      return Ok(());
    }
  }

  Ok(())
}

/// Deploy commands to current guild
#[poise::command(prefix_command)]
async fn deploy(ctx: super::PoiseContext<'_>) -> Result<(), BotError> {
  poise::builtins::register_application_commands(ctx, false).await?;
  Ok(())
}

/// Load schemas into the database
#[poise::command(prefix_command, slash_command)]
async fn schemas(ctx: super::PoiseContext<'_>) -> Result<(), BotError> {
  match execute_schemas(&ctx.data().postgres).await {
    Ok(s) => {
      ctx.reply(s).await?;
    },
    Err(e) => {
      ctx.reply(e.to_string()).await?;
      return Ok(())
    }
  }

  Ok(())
}

/// Upload a Lua plugin to the container
#[poise::command(slash_command)]
async fn upload_plugin(
  ctx: super::PoiseContext<'_>,
  #[description = "Lua plugin file"] file: Attachment
) -> Result<(), BotError> {
  ctx.defer().await?;

  match file.download().await {
    Ok(f) => {
      if let Err(y) = std::fs::write(format!("{PLUGIN_DIR}/{}", file.filename), f) {
        ctx.reply(format!("Failed to write the plugin: `{y}`")).await?;
        return Ok(());
      };
      ctx.reply(format!("Successfully uploaded `{}` plugin!", file.filename)).await?;
    },
    Err(y) => {
      ctx.reply(format!("Failed to upload the plugin: `{y}`")).await?;
      return Ok(());
    }
  };

  Ok(())
}

/// Display the invite cache data (sent as paginated embed)
#[poise::command(slash_command)]
async fn invite_data(ctx: super::PoiseContext<'_>) -> Result<(), BotError> {
  let invite_data = ctx.framework().user_data().invite_data.clone();

  if invite_data.get_all().is_empty() {
    ctx
      .reply(
        [
          "InviteData{} is currently empty!",
          "Check for incoming data from `InviteCreate`, `InviteDelete` and `GuildMemberAddition` events!"
        ]
        .join("\n")
      )
      .await?;
    return Ok(())
  }

  let pages: Vec<String> = invite_data
    .get_all()
    .iter()
    .map(|data| {
      format!(
        "Uses: **{}**\nCode: `{}`\nCreator: **{}**\nChannel: **#{}**",
        data.uses, data.code, data.creator.name, data.channel
      )
    })
    .collect();

  let page_refs: Vec<&str> = pages.iter().map(|s| s.as_str()).collect();

  poise::builtins::paginate(ctx, &page_refs).await?;

  Ok(())
}

/// Perform a SQL query against the database
#[poise::command(slash_command)]
async fn sql(
  ctx: super::PoiseContext<'_>,
  #[description = "PostgreSQL-compatible SQL query"] query: String
) -> Result<(), BotError> {
  let postgres = ctx.data().postgres.clone();
  let mut buf = format!("**Query:**```sql\n{query}\n```");

  match sqlx::query(&query).execute(&postgres).await {
    Ok(r) => {
      let affected = r.rows_affected();
      buf.push_str(&format!("**Result:**```\n{affected}\n```"))
    },
    Err(e) => buf.push_str(&format!("**Error:**```\n{e}\n```"))
  }

  ctx.reply(buf).await?;

  Ok(())
}
