use crate::{
  BotData,
  BotError,
  internals::{
    config::BINARY_PROPERTIES,
    invite_data::InviteData
  }
};

use poise::serenity_prelude::{
  Context,
  InviteCreateEvent,
  InviteDeleteEvent
};

pub async fn on_invite_create(
  ctx: &Context,
  data: &InviteCreateEvent
) -> Result<(), BotError> {
  if data.guild_id.is_none() || data.guild_id.unwrap().get() != BINARY_PROPERTIES.guild_id {
    return Ok(());
  }

  let guild_id = match data.guild_id {
    Some(id) => id,
    None => return Ok(())
  };

  guild_id.invites(&ctx.http).await.unwrap().iter().for_each(|invite| {
    let creator = match &invite.inviter {
      Some(u) => u,
      None => return
    };

    ctx.data::<BotData>().invite_data.insert(
      invite.code.clone(),
      InviteData {
        code:    invite.code.clone(),
        uses:    invite.uses,
        creator: creator.clone(),
        channel: invite.channel.name.clone()
      }
    )
  });

  Ok(())
}

pub async fn on_invite_delete(
  ctx: &Context,
  data: &InviteDeleteEvent
) -> Result<(), BotError> {
  if data.guild_id.is_none() {
    return Ok(());
  }

  ctx.data::<BotData>().invite_data.remove(&data.code);

  Ok(())
}
