use {
  crate::BotResult,
  asahi::utils::{
    format_bytes,
    format_duration,
    os::{
      get_cpu_info,
      get_kernel_info,
      get_memory,
      get_os_info,
      get_uptime
    }
  },
  daggerbot::{
    BOT_VERSION,
    GIT_COMMIT_BRANCH,
    GIT_COMMIT_HASH
  }
};

/// Retrieve host and bot uptimes
#[poise::command(slash_command)]
pub async fn uptime(ctx: super::PoiseContext<'_>) -> BotResult {
  let bot_name = ctx.cache().current_user().name.clone();

  // Fetch system and process memory usage
  let memory = get_memory();
  let (pram, sram, sram_total) = (
    format_bytes(memory.process),
    format_bytes(memory.system.used),
    format_bytes(memory.system.total)
  );

  // Fetch the node hostname from envvar
  let hostname = match std::env::var("DOCKER_HOSTNAME") {
    Ok(h) => h.to_string(),
    Err(_) => "DOCKER_HOSTNAME is empty!".to_string()
  };

  let stat_msg = [
    format!("**{} {}** `{GIT_COMMIT_HASH}:{GIT_COMMIT_BRANCH}`", bot_name, BOT_VERSION.as_str()),
    format!(">>> System: `{}`", format_duration(get_uptime().system)),
    format!("Process: `{}`", format_duration(get_uptime().process)),
    format!("Node: `{hostname}`"),
    format!("CPU: `{}`", get_cpu_info()),
    format!("RAM: `{pram}` (`{sram}/{sram_total}`)"),
    format!("OS: `{}`", get_os_info()),
    format!("Kernel: `{}`", get_kernel_info())
  ];
  ctx.reply(stat_msg.join("\n")).await?;

  Ok(())
}
