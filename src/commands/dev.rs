use {
  crate::{
    BotResult,
    bridges::PLUGIN_DIR
  },
  asahi::utils::database::prepare_tables,
  poise::{
    CreateReply,
    serenity_prelude::{
      Attachment,
      CreateAllowedMentions,
      GenericChannelId,
      builder::CreateMessage
    }
  }
};

/// Developer commands
#[poise::command(
  slash_command,
  owners_only,
  subcommands("echo", "deploy", "schemas", "upload_plugin"),
  default_member_permissions = "MANAGE_GUILD"
)]
pub async fn dev(_: super::PoiseContext<'_>) -> BotResult { Ok(()) }

/// Turn your message into a bot message
#[poise::command(slash_command)]
async fn echo(
  ctx: super::PoiseContext<'_>,
  #[description = "Message to be echoed as a bot"] message: String,
  #[description = "Channel to send this to"]
  #[channel_types("Text", "PublicThread", "PrivateThread")]
  channel: Option<GenericChannelId>
) -> BotResult {
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
async fn deploy(ctx: super::PoiseContext<'_>) -> BotResult {
  poise::builtins::register_application_commands(ctx, false).await?;
  Ok(())
}

/// Load schemas into the database
#[poise::command(prefix_command, slash_command)]
async fn schemas(ctx: super::PoiseContext<'_>) -> BotResult {
  match prepare_tables(&ctx.data().postgres, "schemas").await {
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
) -> BotResult {
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
