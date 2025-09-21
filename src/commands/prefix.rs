use {
  super::mp::tools_perm_check,
  crate::BotResult
};

/// Returns the farm password in specific channel
#[poise::command(prefix_command, guild_only)]
pub async fn farmpw(ctx: super::PoiseContext<'_>) -> BotResult {
  let channel = ctx.channel().await.expect("expected channel data").id().get() as i64;
  let farm = sqlx::query!("SELECT password FROM farms WHERE farm_id = $1", channel)
    .fetch_one(&ctx.data().postgres)
    .await?;
  ctx.reply(format!("Farm password is `{}`", farm.password)).await?;

  Ok(())
}

/// Sets the farm password for specific channel
#[poise::command(prefix_command, guild_only, check = "tools_perm_check", aliases("sfp", "set-farmpw"))]
pub async fn set_farmpw(
  ctx: super::PoiseContext<'_>,
  farm_channel: String,
  farm_name: String,
  password: String
) -> BotResult {
  let q = sqlx::query!(
    "INSERT INTO farms
    (farm_id, farm_name, password)
    VALUES ($1, $2, $3) ON CONFLICT (farm_id)
    DO UPDATE SET
      farm_name = EXCLUDED.farm_name,
      password = EXCLUDED.password
    RETURNING farm_name",
    farm_channel.parse::<i64>()?,
    farm_name,
    password
  )
  .fetch_optional(&ctx.data().postgres)
  .await;

  match q {
    Ok(Some(f)) => {
      ctx.reply(format!("Updated the details for **{}**!", f.farm_name)).await?;
    },
    Ok(None) => {
      ctx.reply("The farm entry doesn't exist!").await?;
    },
    Err(e) => {
      ctx.reply(format!("Database error: {e}")).await?;
    }
  }

  Ok(())
}
