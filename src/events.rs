mod audit_log;
pub mod invite;
mod member;
mod message;
pub mod ready;

use {
  crate::errors::BotError,
  dag_kube::HealthProbe,
  poise::serenity_prelude::{
    ConnectionStage,
    Context,
    CreateInteractionResponseFollowup,
    CreateMessage,
    EventHandler,
    FullEvent,
    Interaction,
    UserId,
    async_trait
  },
  std::sync::Arc
};

#[derive(Debug, poise::Modal)]
#[name = "Reply Panel"]
struct DmModal {
  #[name = "Your message"]
  #[placeholder = "Markdown is supported, but attachments do not"]
  #[max_length = 1024]
  #[paragraph]
  mod_reply: String
}

pub struct DiscordEvents {
  pub probe: Arc<HealthProbe>
}

#[async_trait]
impl EventHandler for DiscordEvents {
  async fn dispatch(
    &self,
    ctx: &Context,
    event: &FullEvent
  ) {
    match event {
      FullEvent::Ready { data_about_bot, .. } => ready::on_ready(ctx, data_about_bot).await.unwrap(),
      FullEvent::ShardStageUpdate { event, .. } => match event.new {
        ConnectionStage::Connected => self.probe.update_ws_status(true).await,
        ConnectionStage::Resuming => self.probe.update_ws_status(true).await,
        ConnectionStage::Disconnected => self.probe.update_ws_status(false).await,
        _ => ()
      },
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
      FullEvent::InteractionCreate { interaction, .. } => {
        if let Interaction::Component(component) = interaction
          && component.data.custom_id.starts_with("dm-")
          && let Err(e) = async {
            let uid = component
              .data
              .custom_id
              .strip_prefix("dm-")
              .and_then(|id| id.parse::<u64>().ok())
              .ok_or_else(|| BotError::from("Component collected an invalid UserID!"))?;

            let data = poise::execute_modal_on_component_interaction::<DmModal>(ctx, component.clone(), None, None).await?;

            UserId::new(uid)
              .dm(
                &ctx.http,
                CreateMessage::new().content(format!("You have a new message!\n> {}", data.unwrap().mod_reply))
              )
              .await?;

            component
              .create_followup(&ctx.http, CreateInteractionResponseFollowup::new().content("Sent your response back!"))
              .await?;

            Ok::<(), BotError>(())
          }
          .await
        {
          eprintln!("Error handling DM reply interaction: {e}");
        }
      },
      _ => ()
    }
  }
}
