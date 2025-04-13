use crate::{
  Error,
  internals::config::BINARY_PROPERTIES
};

use poise::serenity_prelude::{
  AuditLogEntry,
  Change,
  Context,
  CreateMessage,
  GenericChannelId,
  GuildId,
  MemberAction,
  Mentionable,
  RoleId,
  UserId,
  model::guild::audit_log::Action
};

pub async fn on_audit_log_entry_create(
  ctx: &Context,
  entry: &AuditLogEntry,
  guild_id: &GuildId
) -> Result<(), Error> {
  if *guild_id != GuildId::new(BINARY_PROPERTIES.guild_id) {
    return Ok(());
  }

  if !matches!(entry.action, Action::Member(MemberAction::RoleUpdate)) {
    return Ok(());
  }

  let yt_role = RoleId::new(BINARY_PROPERTIES.members_role);

  let role_added = entry.changes.iter().any(|change| match change {
    Change::RolesAdded { new: Some(roles), .. } => roles.iter().any(|role| role.id == yt_role),
    _ => false
  });

  if !role_added {
    return Ok(());
  }

  let user_id = match entry.target_id {
    Some(id) => id,
    None => return Ok(())
  };

  let http = &ctx.http;
  match http.get_member(*guild_id, UserId::new(user_id.get())).await {
    Ok(member) => {
      GenericChannelId::new(BINARY_PROPERTIES.members_chat)
        .send_message(
          http,
          CreateMessage::new().content(
            [
              &format!("## Welcome {}, thanks for supporting Daggerwin!", member.mention()),
              "You unlocked new perks;",
              "- Access to **Members** server",
              "  - Server details are located [here](https://discord.com/channels/468835415093411861/511657659364147200/1333059733854224384)",
              "  - Todo list: <#949380187668242483>",
              "- Early access to new episodes before it goes live to everyone else!",
              "- Members-only community posts",
              "- Nickname & external emotes permissions"
            ]
            .join("\n")
          )
        )
        .await?;
    },
    Err(y) => eprintln!("AuditLogEntry[Err] Failed to fetch user: {y}")
  }

  Ok(())
}
