use {
  crate::{
    BotResult,
    controllers::sql::Settings
  },
  poise::serenity_prelude::{
    Channel,
    Mentionable
  }
};

/// Manage settings for specific namespaces in the bot
#[poise::command(slash_command, subcommands("logs"), default_member_permissions = "ADMINISTRATOR")]
pub async fn settings(_: super::PoiseContext<'_>) -> BotResult { Ok(()) }

/// Manage settings within logs namespace
#[poise::command(slash_command, subcommands("list_ignored_channels", "ignored_channels"))]
async fn logs(_: super::PoiseContext<'_>) -> BotResult { Ok(()) }

/// View the list of ignored channels for message edits and deletes
#[poise::command(slash_command)]
async fn list_ignored_channels(ctx: super::PoiseContext<'_>) -> BotResult {
  let postgres = ctx.data().postgres.clone();
  let settings = Settings::get_logs_ignored_channels(&postgres).await?;

  let mut response = String::from("List of channels that are ignored by the message logs:\n");

  for channel in settings {
    response.push_str(&format!("<#{channel}>\n"));
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
  channel: Channel
) -> BotResult {
  let postgres = ctx.data().postgres.clone();
  let settings = Settings::get_logs_ignored_channels(&postgres).await?;

  if settings.contains(&(channel.id().get() as i64)) {
    let mut new_settings = settings.clone();
    new_settings.retain(|&x| x != channel.id().get() as i64);

    let settings = Settings {
      logs_ignored_channels: new_settings
    };

    settings.update_logs_ignored_channels(&postgres).await?;
    ctx.say(format!("{} is no longer ignored", channel.mention())).await?;
  } else {
    let mut new_settings = settings.clone();
    new_settings.push(channel.id().get() as i64);

    let settings = Settings {
      logs_ignored_channels: new_settings
    };

    settings.update_logs_ignored_channels(&postgres).await?;
    ctx.say(format!("{} is now ignored", channel.mention())).await?;
  }

  Ok(())
}
