use std::sync::LazyLock;

pub struct ConfigMeta {
  pub guild_id:        u64,
  pub embed_colors:    EmbedColorPalette,
  pub ready_notify:    u64,
  pub mp_channels:     MpChannels,
  pub mp_mod_role:     u64,
  pub mp_manager_role: u64,
  pub mp_players_role: u64,
  pub welcome:         u64,
  pub general_chat:    u64,
  pub bot_log:         u64,
  pub bans_kicks_log:  u64,
  pub members_role:    u64,
  pub members_chat:    u64,
  pub developers:      Vec<u64>
}

pub struct MpChannels {
  pub info:                u64,
  pub info_msg:            u64,
  pub suggestion_pool_msg: u64,
  pub announcements:       u64,
  pub activeplayers:       u64,
  pub mod_chat:            u64
}

pub struct EmbedColorPalette {
  pub primary: u32,
  pub red:     u32,
  pub green:   u32,
  pub yellow:  u32
}

#[cfg(feature = "production")]
pub static BINARY_PROPERTIES: LazyLock<ConfigMeta> = LazyLock::new(ConfigMeta::new);

#[cfg(not(feature = "production"))]
pub static BINARY_PROPERTIES: LazyLock<ConfigMeta> = LazyLock::new(|| {
  ConfigMeta::new()
    .guild_id(929807948748832798) // Daggerwin Dev Server
    .embed_colors(EmbedColorPalette {
      primary: 0x559999,
      red:     0xE62C3B,
      green:   0x57F287,
      yellow:  0xFFEA00
    })
    .ready_notify(1091300529696673792) // #i-talk-to-myself-alot
    .mp_info(1091300529696673792) // #i-talk-to-myself-alot
    .mp_info_msg(1259386817355190302)
    .mp_mod_chat(1072397807031418920) // #webhook-testing
    .mp_mod_role(930563630150324345) // Dev
    .mp_manager_role(930563630150324345) // Dev
    .mp_players_role(1040018152827928616) // Test accounts
    .mp_suggestion_pool_msg(1276017784786255952)
    .mp_announcements(929807948748832801) // #spam-chat
    .welcome(1091300529696673792) // #i-talk-to-myself-alot
    .general_chat(1091300529696673792) // #i-talk-to-myself-alot
    .bot_log(929807948748832801) // #spam-chat
    .bans_kicks_log(1091300529696673792) // #i-talk-to-myself-alot
    .members_role(1201551119411847248) // star icon
    .members_chat(1094550226674647040) // #scrapyard-spam
});

impl ConfigMeta {
  fn new() -> Self {
    Self {
      guild_id:        468835415093411861, // Daggerwin
      embed_colors:    EmbedColorPalette {
        primary: 0x0052CF,
        // primary: 0xFFFFFF, // Christmas!
        // primary: 0xFF69B4, // Breast Cancer Awareness month
        red:     0xE62C3B,
        green:   0x57F287,
        yellow:  0xFFEA00
      },
      ready_notify:    548032776830582794,  // #bot-log
      mp_mod_role:     572151330710487041,  // MP Moderator
      mp_manager_role: 1028735939813585029, // MP Manager
      mp_players_role: 798285830669598762,  // MP Players
      mp_channels:     MpChannels {
        info:                543494084363288637, // #mp-server-info
        info_msg:            1149141188079779900,
        suggestion_pool_msg: 1141293129673232435, // in #mp-moderators channel
        announcements:       1084864116776251463, // #mp-announcements
        activeplayers:       739084625862852715,  // #mp-active-players
        mod_chat:            516344221452599306   // #mp-moderators
      },
      welcome:         621134751897616406,  // #welcome
      general_chat:    468835415093411863,  // #general-chat
      bot_log:         548032776830582794,  // #bot-log
      bans_kicks_log:  1048341961901363352, // #bans-and-kicks
      members_role:    473243905132068874,  // YouTube Sponsor
      members_chat:    511657659364147200,  // #sponsor-general
      developers:      vec![
        190407856527376384, // nwero.sama
      ]
    }
  }

  // Scalable functions below;
  #[cfg(not(feature = "production"))]
  fn guild_id(
    mut self,
    guild_id: u64
  ) -> Self {
    self.guild_id = guild_id;
    self
  }

  #[cfg(not(feature = "production"))]
  fn embed_colors(
    mut self,
    colors: EmbedColorPalette
  ) -> Self {
    self.embed_colors = colors;
    self
  }

  #[cfg(not(feature = "production"))]
  fn ready_notify(
    mut self,
    channel_id: u64
  ) -> Self {
    self.ready_notify = channel_id;
    self
  }

  #[cfg(not(feature = "production"))]
  fn mp_info(
    mut self,
    channel_id: u64
  ) -> Self {
    self.mp_channels.info = channel_id;
    self
  }

  #[cfg(not(feature = "production"))]
  fn mp_info_msg(
    mut self,
    message_id: u64
  ) -> Self {
    self.mp_channels.info_msg = message_id;
    self
  }

  #[cfg(not(feature = "production"))]
  fn mp_mod_chat(
    mut self,
    channel_id: u64
  ) -> Self {
    self.mp_channels.mod_chat = channel_id;
    self
  }

  #[cfg(not(feature = "production"))]
  fn mp_mod_role(
    mut self,
    role_id: u64
  ) -> Self {
    self.mp_mod_role = role_id;
    self
  }

  #[cfg(not(feature = "production"))]
  fn mp_manager_role(
    mut self,
    role_id: u64
  ) -> Self {
    self.mp_manager_role = role_id;
    self
  }

  #[cfg(not(feature = "production"))]
  fn mp_players_role(
    mut self,
    role_id: u64
  ) -> Self {
    self.mp_players_role = role_id;
    self
  }

  #[cfg(not(feature = "production"))]
  fn mp_suggestion_pool_msg(
    mut self,
    message_id: u64
  ) -> Self {
    self.mp_channels.suggestion_pool_msg = message_id;
    self
  }

  #[cfg(not(feature = "production"))]
  fn mp_announcements(
    mut self,
    channel_id: u64
  ) -> Self {
    self.mp_channels.announcements = channel_id;
    self
  }

  #[cfg(not(feature = "production"))]
  fn welcome(
    mut self,
    channel_id: u64
  ) -> Self {
    self.welcome = channel_id;
    self
  }

  #[cfg(not(feature = "production"))]
  fn general_chat(
    mut self,
    channel_id: u64
  ) -> Self {
    self.general_chat = channel_id;
    self
  }

  #[cfg(not(feature = "production"))]
  fn bot_log(
    mut self,
    channel_id: u64
  ) -> Self {
    self.bot_log = channel_id;
    self
  }

  #[cfg(not(feature = "production"))]
  fn bans_kicks_log(
    mut self,
    channel_id: u64
  ) -> Self {
    self.bans_kicks_log = channel_id;
    self
  }

  #[cfg(not(feature = "production"))]
  fn members_role(
    mut self,
    role_id: u64
  ) -> Self {
    self.members_role = role_id;
    self
  }

  #[cfg(not(feature = "production"))]
  fn members_chat(
    mut self,
    channel_id: u64
  ) -> Self {
    self.members_chat = channel_id;
    self
  }
}
