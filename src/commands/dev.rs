use {
  crate::BotResult,
  poise::{
    CreateReply,
    serenity_prelude::{
      CreateAllowedMentions,
      GenericChannelId,
      builder::CreateMessage
    }
  }
};

/// Developer commands
#[poise::command(slash_command, owners_only, subcommands("echo", "deploy"), default_member_permissions = "MANAGE_GUILD")]
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
