use {
  mlua::{
    Function,
    Lua,
    Result,
    Value
  },
  regex::Regex,
  std::{
    fmt,
    fs,
    path::Path,
    sync::Arc
  }
};

pub static PLUGIN_DIR: &str = if cfg!(feature = "production") { "plugins" } else { "src/plugins" };
pub type PluginError = std::result::Result<(), BridgeError>;

#[derive(Debug)]
pub enum BridgeError {
  LockPoisoned(String),
  LuaError(mlua::Error)
}

impl fmt::Display for BridgeError {
  fn fmt(
    &self,
    f: &mut fmt::Formatter
  ) -> fmt::Result {
    match self {
      Self::LockPoisoned(msg) => write!(f, "PluginLoader[Err Lock poisoned: {msg}"),
      Self::LuaError(e) => write!(f, "PluginLoader[Err] LuaVM encountered an error: {e}")
    }
  }
}

impl std::error::Error for BridgeError {}

impl From<mlua::Error> for BridgeError {
  fn from(err: mlua::Error) -> Self { Self::LuaError(err) }
}

pub enum ValOrFunc {
  Val(Value),
  Func(Function)
}

impl From<Value> for ValOrFunc {
  fn from(val: Value) -> Self { ValOrFunc::Val(val) }
}

impl From<Function> for ValOrFunc {
  fn from(func: Function) -> Self { ValOrFunc::Func(func) }
}

// todo; setup a proper framework that looks for entrypoint
//       to execute from without setting up the code in rust
//       to support lua code and its execution.. -Nwero, 11/1/25
pub struct LuaPluginLoader {
  pub lua: Arc<Lua>
}

impl LuaPluginLoader {
  pub fn new(lua: Arc<Lua>) -> Self { Self { lua } }

  fn load_file_by_name(
    &self,
    name: &str
  ) -> Result<()> {
    let plugins = Path::new(PLUGIN_DIR);

    if plugins.exists() {
      let plugin = fs::read_to_string(plugins.join(name)).map_err(|e| mlua::Error::external(format!("Failed to read plugin file: {e}")))?;
      self
        .lua
        .load(&plugin)
        .exec()
        .map_err(|e| mlua::Error::external(format!("Failed to execute plugin: {e}")))?;
    } else {
      panic!("{:?}", mlua::Error::external(format!("[LuaVM] Plugin '{name}' does not exist!")))
    }
    Ok(())
  }

  pub fn load_plugin(
    &self,
    name: &str
  ) -> Result<()> {
    self.load_file_by_name(&format!("{name}.lua")).unwrap();
    self.load_globals()?;

    Ok(())
  }

  fn load_globals(&self) -> Result<()> {
    let lua = &self.lua;

    // Lua's regex system is weak ass, I decided to use Rust's powerful regex system instead.
    let rs_regex = lua.create_function(move |_, (pattern, string): (String, String)| {
      let r = Regex::new(&pattern).map_err(mlua::Error::external)?;
      Ok(r.is_match(&string))
    })?;
    lua.globals().set("rs_regex", rs_regex)?;

    Ok(())
  }

  #[allow(dead_code)]
  pub fn set_global<T: Into<ValOrFunc>>(
    &self,
    name: &str,
    value: T
  ) -> Result<()> {
    match value.into() {
      ValOrFunc::Val(v) => self.lua.globals().set(name, v),
      ValOrFunc::Func(f) => self.lua.globals().set(name, f)
    }
  }
}
