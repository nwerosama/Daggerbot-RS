struct Farm {
  farm_id:  u64,
  password: &'static str
}

/// Returns the multifarm password in specific channel
#[poise::command(prefix_command, guild_only)]
pub async fn farmpw(ctx: super::PoiseContext<'_>) -> Result<(), crate::BotError> {
  let farms = [
    Farm {
      farm_id:  1266224299174396045,
      password: "mossypine"
    },
    Farm {
      farm_id:  1266224585007824986,
      password: "ravenwood"
    }
  ];

  for farm in &farms {
    if farm.farm_id == ctx.channel_id().get() {
      ctx.reply(format!("Farm password is `{}`", farm.password)).await?;
      break;
    }
  }

  Ok(())
}
