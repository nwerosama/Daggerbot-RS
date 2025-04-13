use super::pluginloader::{
  BridgeError,
  LuaPluginLoader,
  PluginError
};

use {
  mlua::{
    Function,
    Lua,
    Result,
    Table
  },
  poise::serenity_prelude::{
    GenericChannelId,
    Message,
    UserId,
    http::Http
  },
  std::sync::{
    Arc,
    Mutex
  }
};

enum LuaFunction {
  SendMessage,
  FetchUser
}

impl LuaFunction {
  fn register(
    &self,
    bridge: &LuaSerenityBridge
  ) -> Result<()> {
    match self {
      Self::SendMessage => bridge.register_send_message(),
      Self::FetchUser => bridge.register_fetch_user()
    }
  }
}

pub struct LuaSerenityBridge {
  pub lua:           Arc<Lua>,
  pub serenity_http: Arc<Http>,
  pub plugin_loader: Mutex<LuaPluginLoader>
}

impl LuaSerenityBridge {
  pub fn new(
    lua: Arc<Lua>,
    serenity_http: Arc<Http>
  ) -> Self {
    let plugin_loader = Mutex::new(LuaPluginLoader::new(Arc::clone(&lua)));
    Self {
      lua,
      serenity_http,
      plugin_loader
    }
  }

  pub fn register_plugin(
    &self,
    plugin: &str
  ) -> PluginError {
    let loader = self.plugin_loader.lock().map_err(|e| BridgeError::LockPoisoned(e.to_string()))?;
    loader.load_plugin(plugin)?;

    Ok(())
  }

  pub fn register_all(&self) -> PluginError {
    let funcs = vec![LuaFunction::SendMessage, LuaFunction::FetchUser];

    for func in funcs {
      func.register(self)?;
    }

    Ok(())
  }

  pub fn register_send_message(&self) -> Result<()> {
    let send_message_fn = self.create_send_message_fn()?;
    self.lua.globals().set("send_message", send_message_fn)?;
    Ok(())
  }

  pub fn register_fetch_user(&self) -> Result<()> {
    let fetch_user_fn = self.create_fetch_user_fn()?;
    self.lua.globals().set("fetch_user", fetch_user_fn)?;
    Ok(())
  }

  fn create_send_message_fn(&self) -> Result<Function> {
    let http = Arc::clone(&self.serenity_http);
    self.lua.create_function(move |_, (channel_id, content): (u64, String)| {
      let channel_id = GenericChannelId::new(channel_id);
      let http = Arc::clone(&http);

      tokio::spawn(async move {
        if let Err(y) = channel_id.say(&http, content).await {
          eprintln!("SerenityBridge[Error] {y:?}");
        }
      });

      Ok(())
    })
  }

  pub fn build_message_table(
    &self,
    message: &Message
  ) -> Result<Table> {
    let message_table = self.lua.create_table()?;
    let author_table = self.lua.create_table()?;

    let member_nick = message.member.as_ref().and_then(|m| m.nick.clone());
    let author_global = message.author.global_name.clone();
    let nick_or_global = member_nick
      .or_else(|| author_global.clone())
      .unwrap_or_else(|| message.author.name.clone());

    let author_bank = vec![("name", message.author.name.clone()), ("global", nick_or_global)];
    for (key, value) in author_bank {
      author_table.set(key, value.to_string())?;
    }

    message_table.set("content", message.content.clone().to_string())?;
    message_table.set("author", author_table)?;
    message_table.set("channel_id", message.channel_id.to_string())?;

    Ok(message_table)
  }

  fn create_fetch_user_fn(&self) -> Result<Function> {
    let http = Arc::clone(&self.serenity_http);
    self.lua.create_function(move |_, user_id: u64| {
      let http = Arc::clone(&http);

      tokio::spawn(async move {
        if let Err(y) = http.get_user(UserId::new(user_id)).await {
          eprintln!("SerenityBridge[Error] {y:?}");
        }
      });

      Ok(())
    })
  }
}
