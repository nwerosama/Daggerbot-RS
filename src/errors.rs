use {
  crate::{
    BotData,
    BotError,
    internals::utils::mention_dev
  },
  poise::FrameworkError
};

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
        eprintln!("PoiseCommandError({}): {error}", ctx.command().qualified_name);
      }

      eprintln!("CommandErrorDebug: {error:?}");
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
        eprintln!("PoiseCommandPanic({}): {payload:#?}", ctx.command().qualified_name);
      }
    },
    FrameworkError::MissingBotPermissions {
      missing_permissions, ctx, ..
    } => {
      println!("PoiseMissingBotPermissions({}): {missing_permissions:#?}", ctx.command().qualified_name);
    },
    FrameworkError::MissingUserPermissions {
      missing_permissions, ctx, ..
    } => {
      println!(
        "PoiseMissingUserPermissions({}): {:#?}",
        ctx.command().qualified_name,
        missing_permissions
      );
    },
    FrameworkError::ArgumentParse { error, ctx, input, .. } => {
      let input = input.unwrap_or_default();
      println!("PoiseArgumentParse({}): {error} (input: {input})", ctx.command().qualified_name);
      ctx
        .send(
          poise::CreateReply::default()
            .content(format!("Error parsing your input: {error}"))
            .ephemeral(true)
        )
        .await
        .expect("Error sending message");
    },
    FrameworkError::CommandCheckFailed { error, ctx, .. } => {
      let error = match error {
        Some(e) => e.to_string(),
        None => format!("{} does not fulfill the check's requirements", ctx.author().display_name())
      };

      println!("PoiseCommandCheckFailed({}): {error}", ctx.command().qualified_name);
      ctx
        .send(
          poise::CreateReply::default()
            .content("This command uses a check and you don't meet the requirements.")
            .ephemeral(true)
        )
        .await
        .expect("Error sending message");
    },
    FrameworkError::NotAnOwner { ctx, .. } => {
      println!(
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
      println!(
        "PoiseUnknownInteraction: {} tried to execute an unknown interaction ({})",
        interaction.user.name, interaction.data.name
      );
    },
    other => println!("PoiseOtherError: {other}")
  }
}
