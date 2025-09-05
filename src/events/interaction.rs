use {
  crate::errors::BotError,
  asahi::{
    error,
    warn
  },
  poise::serenity_prelude::{
    Context,
    CreateInteractionResponseFollowup,
    CreateMessage,
    Interaction,
    UserId
  }
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

pub async fn on_interaction_create(
  ctx: &Context,
  interaction: &Interaction
) -> Result<(), BotError> {
  if let Interaction::Component(component) = interaction
    && component.data.custom_id.starts_with("dm-")
    && let Err(e) = async {
      let uid = component
        .data
        .custom_id
        .strip_prefix("dm-")
        .and_then(|id| id.parse::<u64>().ok())
        .ok_or_else(|| BotError::from("Component stored an invalid UserID!"))?;

      let data = poise::execute_modal_on_component_interaction::<DmModal>(ctx, component.clone(), None, None).await?;

      let mod_reply = match data {
        Some(d) => d.mod_reply,
        None => {
          warn!("Modal passed empty value, not sending response back to user!");
          return Ok(())
        }
      };

      UserId::new(uid)
        .dm(
          &ctx.http,
          CreateMessage::new().content(format!("You have a new message!\n>>> {mod_reply}"))
        )
        .await?;

      component
        .create_followup(
          &ctx.http,
          CreateInteractionResponseFollowup::new().content(format!("Sent your response back!\n>>> {mod_reply}"))
        )
        .await?;

      Ok::<(), BotError>(())
    }
    .await
  {
    error!("Error handling DM reply interaction: {e}");
  }

  Ok(())
}
