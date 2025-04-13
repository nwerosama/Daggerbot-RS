mod dev;
mod moderation;
mod mp;
mod ping;
mod settings;
mod uptime;

pub use {
  dev::dev,
  moderation::*,
  mp::mp,
  ping::ping,
  settings::settings,
  uptime::uptime
};

pub type PoiseContext<'a> = poise::Context<'a, crate::BotData, crate::Error>;

macro_rules! collect {
  () => {
    vec![
      // dev
      commands::dev(),
      // moderation
      commands::ban(),
      commands::kick(),
      commands::unban(),
      commands::warn(),
      commands::mute(),
      commands::unmute(),
      commands::case(),
      commands::pw(),
      commands::pu(),
      // unsorted mess
      commands::mp(),
      commands::ping(),
      commands::settings(),
      commands::uptime(),
    ]
  };
}
pub(crate) use collect;
