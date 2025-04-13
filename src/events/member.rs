use crate::{
  BotData,
  Error,
  internals::{
    config::BINARY_PROPERTIES,
    invite_data::InviteData,
    utils::format_timestamp
  }
};

use {
  poise::serenity_prelude::{
    Context,
    CreateEmbed,
    CreateEmbedFooter,
    CreateMessage,
    GenericChannelId,
    GuildId,
    Member,
    RoleId,
    Timestamp,
    User,
    small_fixed_array::FixedString
  },
  serde::{
    Deserialize,
    Serialize
  }
};

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct CachedMember {
  pub nick:  FixedString<u8>,
  pub roles: Vec<RoleId>,
  pub user:  User
}

pub async fn on_guild_member_addition(
  ctx: &Context,
  new_member: &Member
) -> Result<(), Error> {
  if new_member.guild_id == GuildId::new(929807948748832798) {
    let dagstaff_role = RoleId::new(1009754300915916860);
    let test_accs_role = RoleId::new(1040018152827928616);
    let nwero_alts: Vec<u64> = vec![107456650071269376, 923275627136700457];

    let mut is_alt = false;
    for alt in nwero_alts.iter() {
      if new_member.user.id == *alt {
        new_member
          .add_role(&ctx.http, test_accs_role, Some("Account owned by nwero.sama, guinea pig role given"))
          .await?;
        is_alt = true;
        break;
      }
    }

    if !is_alt || new_member.roles.contains(&dagstaff_role) {
      new_member.add_role(&ctx.http, dagstaff_role, Some("Staff member in main guild")).await?;
    }
  } else if new_member.guild_id == GuildId::new(BINARY_PROPERTIES.guild_id) {
    println!("GuildMemberAddition[Debug] WS event received, preparing to fire welcome message");
    println!("GuildMemberAddition[Debug] Gateway sent member data for {}", new_member.user.tag());

    let cached_guild = match new_member.guild_id.to_guild_cached(&ctx.cache) {
      Some(g) => g.clone(),
      None => return Ok(())
    };

    let ordinal_suffix = match cached_guild.member_count % 100 {
      11..=13 => "th",
      _ => match cached_guild.member_count % 10 {
        1 => "st",
        2 => "nd",
        3 => "rd",
        _ => "th"
      }
    };

    let mut is_bot = "Bot";
    if !new_member.user.bot() {
      is_bot = "Member";
    }

    let welcome_channel = GenericChannelId::new(BINARY_PROPERTIES.welcome);
    let log_channel = GenericChannelId::new(BINARY_PROPERTIES.bot_log);

    const NO_INVITE_DATA: &str = "Invite data not populated!";
    let invite_data = ctx.data::<BotData>().invite_data.clone();
    let new_invites = new_member.guild_id.invites(&ctx.http).await?;
    let used_invite = new_invites.iter().find(|i| match invite_data.get(&i.code) {
      Some(inv) => inv.uses < i.uses,
      None => false
    });

    // Proceed even if the invite data is not populated yet
    let invite_data_string = match used_invite {
      Some(i) => match invite_data.get(&i.code) {
        Some(inv) => [
          format!("Invite: `{}`", i.code),
          format!("Created by: **{}**", inv.creator.name),
          format!("Channel: **#{}**", inv.channel)
        ]
        .join("\n"),
        None => NO_INVITE_DATA.to_string()
      },
      None => NO_INVITE_DATA.to_string()
    };

    // Populate the invite cache with new invite entries if available
    for i in new_invites.iter() {
      let creator = match i.inviter.as_ref() {
        Some(u) => u.clone(),
        None => continue
      };
      invite_data.insert(
        i.code.clone(),
        InviteData {
          uses: i.uses,
          code: i.code.clone(),
          creator,
          channel: i.channel.name.clone()
        }
      )
    }

    match welcome_channel
      .send_message(
        &ctx.http,
        CreateMessage::new().embed(
          CreateEmbed::new()
            .color(BINARY_PROPERTIES.embed_colors.primary)
            .thumbnail(new_member.user.face())
            .title(format!("Welcome to {}, {}!", cached_guild.name, new_member.user.tag()))
            .footer(CreateEmbedFooter::new(format!("{}{ordinal_suffix} member", cached_guild.member_count)))
        )
      )
      .await
    {
      Ok(_) => {
        log_channel
          .send_message(
            &ctx.http,
            CreateMessage::new().embed(
              CreateEmbed::new()
                .color(BINARY_PROPERTIES.embed_colors.green)
                .thumbnail(new_member.user.face())
                .title(format!("{is_bot} Joined: {}", new_member.user.tag()))
                .fields(vec![
                  (
                    "Account Creation Date:",
                    format_timestamp(new_member.user.id.created_at().timestamp()),
                    false
                  ),
                  ("Invite Data:", invite_data_string, false),
                ])
                .footer(CreateEmbedFooter::new(format!(
                  "Total members: {}{ordinal_suffix} | ID: {}",
                  cached_guild.member_count, new_member.user.id
                )))
                .timestamp(Timestamp::now())
            )
          )
          .await?;
      },
      Err(e) => eprintln!("Error sending welcome message: {e:?}")
    }
  }

  Ok(())
}

pub async fn on_guild_member_removal(
  ctx: &Context,
  member_data_if_available: &Option<Member>,
  user: &User
) -> Result<(), Error> {
  if member_data_if_available
    .as_ref()
    .is_none_or(|data| data.guild_id != GuildId::new(BINARY_PROPERTIES.guild_id))
  {
    #[cfg(not(feature = "production"))]
    println!("GuildMemberRemoval[Debug] Member data unavailable, not emitting log");
    return Ok(());
  }

  let log_channel = GenericChannelId::new(BINARY_PROPERTIES.bot_log);

  let mut is_bot = "Bot";
  if !user.bot() {
    is_bot = "Member";
  }

  println!("GuildMemberRemoval[Debug] WS event received, preparing to fire leave message");
  let member_data = match member_data_if_available {
    Some(m) => m.clone(),
    None => return Ok(())
  };
  println!("GuildMemberRemoval[Debug] Gateway sent member data for {}", member_data.user.tag());

  let roles = member_data.roles.iter().map(|r| format!("<@&{r}>")).collect::<Vec<String>>().join(" ");
  let roles_count = member_data.roles.len();

  log_channel
    .send_message(
      &ctx.http,
      CreateMessage::new().embed(
        CreateEmbed::new()
          .color(BINARY_PROPERTIES.embed_colors.red)
          .thumbnail(user.face())
          .title(format!("{is_bot} Left: {}", user.tag()))
          .fields(vec![
            ("Account Creation Date:", format_timestamp(user.id.created_at().timestamp()), false),
            ("Server Join Date:", format_timestamp(member_data.joined_at.unwrap().timestamp()), false),
            (&format!("Roles: {roles_count}"), roles, false),
          ])
          .footer(CreateEmbedFooter::new(format!("ID: {}", user.id)))
          .timestamp(Timestamp::now())
      )
    )
    .await?;

  Ok(())
}
