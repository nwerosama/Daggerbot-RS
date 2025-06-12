use crate::BotError;

struct Farm {
  farm_id:  u64,
  password: &'static str
}

/// Returns the multifarm password in specific channel
#[poise::command(prefix_command, guild_only)]
pub async fn farmpw(ctx: super::PoiseContext<'_>) -> Result<(), BotError> {
  let channel_id = ctx.channel_id().get();

  let farms = [
    Farm {
      farm_id:  1266224299174396045,
      password: "roughlane"
    },
    Farm {
      farm_id:  1266224585007824986,
      password: "foxglove"
    }
  ];

  for farm in &farms {
    if farm.farm_id == channel_id {
      let passwd_txt = format!("Farm password is `{}`", farm.password);
      ctx.reply(passwd_txt).await?;
      break;
    }
  }

  Ok(())
}
