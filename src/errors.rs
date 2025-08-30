use {
  crate::{
    BotData,
    internals::utils::mention_dev
  },
  asahi::error,
  poise::{
    CreateReply,
    FrameworkError
  }
};

pub type BotError = Box<dyn std::error::Error + Send + Sync>;

pub async fn fw_errors(error: FrameworkError<'_, BotData, BotError>) {
  match error {
    FrameworkError::Command { error, ctx, .. } => {
      if (ctx
        .reply(format!(
          "Encountered an error during command execution, ask {} to check console for more details!",
          mention_dev(ctx).unwrap_or_default()
        ))
        .await)
        .is_err()
      {
        error!("PoiseCommandError({}): {error}", ctx.command().qualified_name);
      }

      error!("CommandErrorDebug: {error:?}");
    },
    FrameworkError::CommandPanic { payload, ctx, .. } => {
      if (ctx
        .reply(format!(
          "Encountered a panic during command execution, ask {} to check console for more details!",
          mention_dev(ctx).unwrap_or_default()
        ))
        .await)
        .is_err()
      {
        error!("PoiseCommandPanic({}): {payload:#?}", ctx.command().qualified_name);
      }
    },
    FrameworkError::CommandCheckFailed { error, ctx, .. } => {
      let error = match error {
        Some(e) => e.to_string(),
        None => format!("{} does not fulfill the check's requirements", ctx.author().display_name())
      };

      error!("PoiseCommandCheckFailed({}): {error}", ctx.command().qualified_name);
      ctx
        .send(
          CreateReply::default()
            .content("This command uses a check and you don't meet the requirements.")
            .ephemeral(true)
        )
        .await
        .expect("Error sending message");
    },
    FrameworkError::ArgumentParse { error, input, ctx, .. } => {
      let input = input.unwrap_or_else(|| "<none>".to_string());
      let msg = format!("PoiseArgumentParse({input}): {error:?}");
      if (ctx.reply(format!("Wrong command argument! Used `{input}`, error: `{error}`")))
        .await
        .is_err()
      {
        error!(msg)
      }
      error!(msg)
    },
    FrameworkError::NotAnOwner { ctx, .. } => {
      error!(
        "PoiseNotAnOwner: {} tried to execute a developer-level command ({})",
        ctx.author().name,
        ctx.command().qualified_name
      );
      ctx
        .reply("This command is only available to the bot owners, you're not one of them!")
        .await
        .expect("Error sending message");
    },
    FrameworkError::UnknownInteraction { interaction, .. } => {
      error!(
        "PoiseUnknownInteraction: {} tried to execute an unknown interaction ({})",
        interaction.user.name, interaction.data.name
      );
    },
    FrameworkError::UnknownCommand { .. } => (),
    other => error!("PoiseOtherError: {other}")
  }
}
