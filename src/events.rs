mod audit_log;
pub mod invite;
mod member;
mod message;
pub mod ready;

use poise::serenity_prelude::{
  Context,
  EventHandler,
  FullEvent,
  async_trait
};

pub struct DiscordEvents;

#[async_trait]
impl EventHandler for DiscordEvents {
  async fn dispatch(
    &self,
    ctx: &Context,
    event: &FullEvent
  ) {
    match event {
      FullEvent::Ready { data_about_bot, .. } => ready::on_ready(ctx, data_about_bot).await.unwrap(),
      FullEvent::InviteCreate { data, .. } => invite::on_invite_create(ctx, data).await.unwrap(),
      FullEvent::InviteDelete { data, .. } => invite::on_invite_delete(ctx, data).await.unwrap(),
      FullEvent::Message { new_message, .. } => {
        message::on_message(ctx, new_message).await.unwrap();
        message::on_message_lua(ctx, new_message).await.unwrap();
      },
      FullEvent::MessageUpdate { event, .. } => message::on_message_update(ctx, event).await.unwrap(),
      FullEvent::MessageDelete {
        channel_id,
        deleted_message_id,
        ..
      } => message::on_message_delete(ctx, channel_id, deleted_message_id).await.unwrap(),
      FullEvent::GuildMemberAddition { new_member, .. } => member::on_guild_member_addition(ctx, new_member).await.unwrap(),
      FullEvent::GuildMemberRemoval {
        member_data_if_available,
        user,
        ..
      } => member::on_guild_member_removal(ctx, member_data_if_available, user).await.unwrap(),
      FullEvent::GuildAuditLogEntryCreate { entry, guild_id, .. } => audit_log::on_audit_log_entry_create(ctx, entry, guild_id).await.unwrap(),
      _ => ()
    }
  }
}
