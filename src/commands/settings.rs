use crate::{
  BotError,
  controllers::sql::Settings
};

use poise::serenity_prelude::{
  ChannelId,
  Mentionable
};

/// Manage settings for specific namespaces in the bot
#[poise::command(slash_command, subcommands("logs"), default_member_permissions = "ADMINISTRATOR")]
pub async fn settings(_: super::PoiseContext<'_>) -> Result<(), BotError> { Ok(()) }

/// Manage settings within logs namespace
#[poise::command(slash_command, subcommands("list_ignored_channels", "ignored_channels"))]
async fn logs(_: super::PoiseContext<'_>) -> Result<(), BotError> { Ok(()) }

/// View the list of ignored channels for message edits and deletes
#[poise::command(slash_command)]
async fn list_ignored_channels(ctx: super::PoiseContext<'_>) -> Result<(), BotError> {
  let postgres = ctx.data().postgres.clone();
  let settings = Settings::get_logs_ignored_channels(&postgres).await?;

  let mut response = String::from("List of channels that are ignored by the message logs:\n");

  for channel in settings {
    response.push_str(&format!("<#{}>\n", channel));
  }

  ctx.say(response).await?;

  Ok(())
}

/// Manage ignored channels for message edits and deletes
#[poise::command(slash_command)]
async fn ignored_channels(
  ctx: super::PoiseContext<'_>,
  #[description = "Channel to (un)ignore"]
  #[channel_types("Text")]
  channel: ChannelId
) -> Result<(), BotError> {
  let postgres = ctx.data().postgres.clone();
  let settings = Settings::get_logs_ignored_channels(&postgres).await?;

  if settings.contains(&(channel.get() as i64)) {
    let mut new_settings = settings.clone();
    new_settings.retain(|&x| x != channel.get() as i64);

    let settings = Settings {
      logs_ignored_channels: new_settings
    };

    settings.update_logs_ignored_channels(&postgres).await?;
    ctx.say(format!("{} is no longer ignored", channel.mention())).await?;
  } else {
    let mut new_settings = settings.clone();
    new_settings.push(channel.get() as i64);

    let settings = Settings {
      logs_ignored_channels: new_settings
    };

    settings.update_logs_ignored_channels(&postgres).await?;
    ctx.say(format!("{} is now ignored", channel.mention())).await?;
  }

  Ok(())
}
