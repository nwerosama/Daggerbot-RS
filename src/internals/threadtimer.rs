use {
  crate::{
    BotData,
    internals::config::BINARY_PROPERTIES
  },
  asahi::{
    AsahiCoordinator,
    AsahiResult,
    async_trait,
    error,
    info
  },
  poise::serenity_prelude::{
    ChannelId,
    Context,
    GuildId
  },
  std::{
    sync::Arc,
    time::{
      SystemTime,
      UNIX_EPOCH
    }
  }
};

pub struct ThreadTimer {
  pub ctx: Arc<Context>
}

#[async_trait]
impl AsahiCoordinator<BotData> for ThreadTimer {
  fn name(&self) -> &'static str { "Thread Timer" }

  fn interval(&self) -> u64 { 900 }

  async fn main_loop(
    &self,
    _: Arc<BotData>
  ) -> AsahiResult<()> {
    let help_forum_id = ChannelId::new(BINARY_PROPERTIES.help_forum);
    let mut threads = Vec::new();

    loop {
      match self.ctx.http.get_guild_active_threads(GuildId::new(BINARY_PROPERTIES.guild_id)).await {
        Ok(t) => {
          threads.extend(t.threads.into_iter().filter(|th| th.parent_id == help_forum_id));
          if !t.has_more {
            break;
          }
        },
        Err(e) => {
          error!("Error fetching the active threads: {e}");
          break;
        }
      }

      match self.ctx.http.get_channel_archived_public_threads(help_forum_id, None, None).await {
        Ok(t) => {
          threads.extend(t.threads);
          if !t.has_more {
            break;
          }
        },
        Err(e) => {
          error!("Error fetching the archived threads: {e}");
          break;
        }
      }
    }

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let d14 = now - 1209600;

    for thread in threads {
      if let Some(last_msg) = thread.base.last_message_id {
        match self.ctx.http.get_message(thread.id.widen(), last_msg).await {
          Ok(m) => {
            let msg_ts = m.timestamp.timestamp() as u64;

            if msg_ts <= d14 {
              match self.ctx.http.delete_channel(thread.id.widen(), None).await {
                Ok(_) => info!("Deleted \"#{}\" as it has been inactive for 14 days", thread.base.name),
                Err(e) => error!("Couldn't delete \"#{}\" due to an error: {e}", thread.base.name)
              }
            }
          },
          Err(e) => {
            error!("Couldn't delete the thread so continuing: {e}");
            continue
          }
        }
      }
    }

    Ok(())
  }
}
