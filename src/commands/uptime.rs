use crate::{
  BotError,
  GIT_COMMIT_BRANCH,
  GIT_COMMIT_HASH,
  internals::utils::BOT_VERSION
};

use {
  asahi::utils::{
    format_bytes,
    format_duration,
    os::{
      get_kernel_info,
      get_memory,
      get_os_info,
      get_uptime
    }
  },
  std::env::var,
  sysinfo::System
};

/// Retrieve host and bot uptimes
#[poise::command(slash_command)]
pub async fn uptime(ctx: super::PoiseContext<'_>) -> Result<(), BotError> {
  let _bot = ctx.http().get_current_user().await.unwrap();
  let mut sys = System::new_all();
  sys.refresh_all();

  // Fetch system's processor
  let cpu = sys.cpus();

  // Fetch system and process memory usage
  let memory = get_memory();
  let (pram, sram, sram_total) = (
    format_bytes(memory.process),
    format_bytes(memory.system.used),
    format_bytes(memory.system.total)
  );

  // Fetch the node hostname from envvar
  let docker_node = match var("DOCKER_HOSTNAME") {
    Ok(h) => h.to_string(),
    Err(_) => "DOCKER_HOSTNAME is empty!".to_string()
  };

  let stat_msg = [
    format!("**{} {}** `{GIT_COMMIT_HASH}:{GIT_COMMIT_BRANCH}`", _bot.name, BOT_VERSION.as_str()),
    format!(">>> System: `{}`", format_duration(get_uptime().system)),
    format!("Process: `{}`", format_duration(get_uptime().process)),
    format!("Node: `{docker_node}`"),
    format!("CPU: `{}`", cpu[0].brand()),
    format!("RAM: `{pram}` (`{sram}/{sram_total}`)"),
    format!("OS: `{}`", get_os_info()),
    format!("Kernel: `{}`", get_kernel_info())
  ];
  ctx.reply(stat_msg.join("\n")).await?;

  Ok(())
}
