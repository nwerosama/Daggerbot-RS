use {
  crate::BotError,
  serde::Deserialize
};

#[derive(Deserialize)]
struct StatusPage {
  metrics: Vec<Metrics>
}

#[derive(Deserialize)]
struct Metrics {
  summary: Summary
}

#[derive(Deserialize)]
struct Summary {
  mean: f64
}

/// Check latency between bot and Discord API
#[poise::command(slash_command)]
pub async fn ping(ctx: super::PoiseContext<'_>) -> Result<(), BotError> {
  let statuspage = reqwest::get("https://discordstatus.com/metrics-display/5k2rt9f7pmny/day.json")
    .await
    .unwrap()
    .json::<StatusPage>()
    .await
    .unwrap();

  let mut latencies = String::new();
  latencies.push_str(&format!("Discord: **{:.0?}ms**\n", statuspage.metrics[0].summary.mean));
  latencies.push_str(&format!("WebSocket: **{:.0?}**", ctx.ping().await.unwrap_or_default()));

  ctx.reply(latencies).await?;

  Ok(())
}
