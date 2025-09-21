mod dev;
mod faq;
mod moderation;
mod mp;
mod ping;
mod prefix;
mod settings;
mod uptime;

pub use {
  dev::dev,
  faq::faq,
  moderation::*,
  mp::mp,
  ping::ping,
  prefix::{
    farmpw,
    set_farmpw
  },
  settings::settings,
  uptime::uptime
};

pub type PoiseContext<'a> = poise::Context<'a, crate::BotData, crate::BotError>;

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
      // fsmp stuff
      commands::mp(),
      commands::farmpw(),
      commands::set_farmpw(),
      // unsorted mess
      commands::faq(),
      commands::ping(),
      commands::settings(),
      commands::uptime(),
    ]
  };
}
pub(crate) use collect;
