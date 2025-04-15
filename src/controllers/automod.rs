use crate::{
  BotData,
  BotError,
  commands::{
    ActionType,
    LogChannel,
    Target,
    generate_id
  },
  controllers::{
    cache::RedisController,
    sql::{
      ProhibitedUrls,
      ProhibitedWords,
      Sanctions
    }
  },
  internals::{
    config::BINARY_PROPERTIES,
    utils::{
      format_duration,
      token_path
    }
  }
};

use {
  dashmap::DashMap,
  lazy_static::lazy_static,
  poise::serenity_prelude::{
    CacheHttp,
    Context,
    CreateEmbed,
    CreateMessage,
    Member,
    Mentionable,
    Message,
    Timestamp,
    UserId
  },
  regex::Regex,
  reqwest::{
    Client,
    Url
  },
  serde::{
    Deserialize,
    Serialize
  },
  smallvec::SmallVec,
  sqlx::PgPool,
  std::{
    borrow::Cow,
    sync::{
      Arc,
      atomic::{
        AtomicU32,
        Ordering::SeqCst
      }
    },
    time::{
      Duration,
      SystemTime,
      UNIX_EPOCH
    }
  },
  tokio::{
    sync::RwLock,
    time::sleep
  }
};

const MD_KEY_MAIN: &str = "MaliciousDomains";
const MD_KEY_LU: &str = "MaliciousDomains:LastUpdate";
const MD_BLOCKLIST: [&str; 4] = [
  "https://raw.githubusercontent.com/Discord-AntiScam/scam-links/main/list.txt",
  "https://raw.githubusercontent.com/mitchellkrogza/Phishing.Database/master/phishing-links-NEW-today.txt",
  "https://raw.githubusercontent.com/RedPanda4552/PandaPhishLists/main/seen-on-discord.txt",
  "https://raw.githubusercontent.com/nwerosama/FishDB/main/domains.txt"
];

lazy_static! {
  static ref URL_REGEX: Regex = Regex::new(r"(?i)(?:https?://)?(?:www\.)?([a-zA-Z0-9][a-zA-Z0-9-]*(?:\.[a-zA-Z0-9-]+)+)").unwrap();
  static ref INVITE_REGEX: Regex = Regex::new(r"(?i)discord(?:\.gg|(?:app)?\.com/invite)/[\w-]+").unwrap();
  static ref MASKED_URL_REGEX: Regex = Regex::new(r"\[.*?\]\(<?(https?://[^>]+)>?\)").unwrap();
  static ref REQWEST_CLIENT: Client = Client::new();
}

// Rule configuration
#[derive(Debug, Clone)]
pub struct AutomodPolicy {
  pub enabled:        bool,
  pub policy_type:    AutomodPolicyType,
  pub action:         ActionType,
  pub reason:         String,
  /// Number of warns before policy's action is triggered
  pub warn_threshold: u32,
  /// Duration in seconds
  pub mute_duration:  Option<i64>
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum AutomodPolicyType {
  AntiSpam,
  InviteLinks,
  ProhibitedWords,
  MaliciousLinks, // For phishing links
  ProhibitedUrls
}

// Spam tracking
#[derive(Debug, Default, Serialize, Deserialize)]
struct UserMessageStats {
  messages:        SmallVec<[i64; 5]>,
  policy_warnings: DashMap<AutomodPolicyType, (AtomicU32, i64)>
}

impl UserMessageStats {
  fn increment_warnings(
    &self,
    policy_type: &AutomodPolicyType,
    timestamp: i64
  ) -> u32 {
    let mut entry = self
      .policy_warnings
      .entry(policy_type.clone())
      .or_insert_with(|| (AtomicU32::new(0), timestamp));
    entry.1 = timestamp;
    entry.0.fetch_add(1, SeqCst) + 1
  }

  fn reset_warnings(
    &self,
    policy_type: &AutomodPolicyType
  ) {
    if let Some(entry) = self.policy_warnings.get(policy_type) {
      entry.0.store(0, SeqCst);
    }
  }

  fn check_and_reset_warns(
    &self,
    policy_type: &AutomodPolicyType,
    current_ts: i64,
    reset_interval: i64
  ) {
    if let Some(entry) = self.policy_warnings.get(policy_type) {
      if current_ts - entry.1 >= reset_interval {
        self.reset_warnings(policy_type);
      }
    }
  }
}

pub struct Automoderator {
  policies: Arc<RwLock<Vec<AutomodPolicy>>>,
  pw_list:  Vec<Regex>,
  pu_list:  Vec<String>,
  redis:    Arc<RedisController>
}

impl AutomodPolicy {
  pub fn anti_spam() -> Self {
    Self {
      enabled:        true,
      policy_type:    AutomodPolicyType::AntiSpam,
      action:         ActionType::Mute,
      reason:         "Spam detection".to_string(),
      warn_threshold: 3,
      mute_duration:  Some(3600) // 1 hour
    }
  }

  pub fn prohibited_words() -> Self {
    Self {
      enabled:        true,
      policy_type:    AutomodPolicyType::ProhibitedWords,
      action:         ActionType::Mute,
      reason:         "Use of prohibited words".to_string(),
      warn_threshold: 2,
      mute_duration:  Some(1800) // 30 minutes
    }
  }

  pub fn invite_links() -> Self {
    Self {
      enabled:        true,
      policy_type:    AutomodPolicyType::InviteLinks,
      action:         ActionType::Ban,
      reason:         "Posting invite link".to_string(),
      warn_threshold: 2,
      mute_duration:  None
    }
  }

  pub fn malicious_links() -> Self {
    Self {
      enabled:        true,
      policy_type:    AutomodPolicyType::MaliciousLinks,
      action:         ActionType::Ban,
      reason:         "Posting a malicious link".to_string(),
      warn_threshold: 2,
      mute_duration:  None
    }
  }

  pub fn prohibited_urls() -> Self {
    Self {
      enabled:        true,
      policy_type:    AutomodPolicyType::ProhibitedUrls,
      action:         ActionType::Mute,
      reason:         "Posting a banned link".to_string(),
      warn_threshold: 2,
      mute_duration:  Some(1800) // 30 minutes
    }
  }
}

impl Automoderator {
  pub async fn new(
    db: &PgPool,
    redis: Arc<RedisController>
  ) -> Result<Self, BotError> {
    Ok(Self {
      policies: Arc::new(RwLock::new(vec![
        AutomodPolicy::anti_spam(),
        AutomodPolicy::prohibited_words(),
        AutomodPolicy::invite_links(),
        AutomodPolicy::malicious_links(),
        AutomodPolicy::prohibited_urls(),
      ])),
      pw_list: Self::load_prohibited_words(db).await?,
      pu_list: Self::load_prohibited_urls(db).await?,
      redis
    })
  }

  fn staff_check(
    &self,
    member: &Member
  ) -> bool {
    let staff_roles = [468842789053136897, 468841295150972929];
    member.roles.iter().any(|r| staff_roles.contains(&r.get()))
  }

  pub async fn process_message(
    &self,
    ctx: &Context,
    msg: &Message
  ) -> Result<(), BotError> {
    if let Some(violation) = self.check_violations(msg).await {
      match msg.member(ctx).await {
        Ok(m) => {
          if self.staff_check(&m) {
            println!("[automod::process_message] {} has a staff role, ignoring", msg.author.name);
            return Ok(());
          }
        },
        Err(_) => {
          eprintln!("[automod::process_message] Got hit by an error, couldn't check anyway!");
          return Ok(());
        }
      }

      if violation.policy_type == AutomodPolicyType::MaliciousLinks {
        println!("[automod::process_message] ({}) Malicious URL: {}", msg.author.name, msg.content);
      }

      self.handle_violation(ctx, msg, violation).await?;
    }

    Ok(())
  }

  async fn check_violations(
    &self,
    msg: &Message
  ) -> Option<AutomodPolicy> {
    let policies = self.policies.read().await;
    let current_ts = msg.timestamp.unix_timestamp();

    let checks = [
      (AutomodPolicyType::InviteLinks, self.contains_invite_links(&msg.content)),
      (
        AutomodPolicyType::AntiSpam,
        self.is_spam(msg.author.id.get(), msg.timestamp.unix_timestamp()).await
      ),
      (AutomodPolicyType::ProhibitedWords, self.contains_prohibited_words(&msg.content)),
      (AutomodPolicyType::MaliciousLinks, self.contains_malicious_links(&msg.content).await),
      (AutomodPolicyType::ProhibitedUrls, self.contains_prohibited_urls(&msg.content))
    ];

    for (policy_type, violated) in checks {
      if violated {
        if let Some(policy) = policies.iter().find(|p| p.enabled && p.policy_type == policy_type) {
          let user_stats_key = format!("Discord:UserStats:{}", msg.author.id.get());
          if let Ok(Some(d)) = self.redis.get(&user_stats_key).await {
            let stats: UserMessageStats = serde_json::from_str(&d).unwrap_or_default();
            stats.check_and_reset_warns(&policy_type, current_ts, 300); // 5m
            let data = serde_json::to_string(&stats).unwrap();
            self.redis.set(&user_stats_key, &data).await.unwrap();
          }
          return Some(policy.clone());
        }
      }
    }

    None
  }

  async fn is_spam(
    &self,
    user_id: u64,
    timestamp: i64
  ) -> bool {
    let user_stats_key = format!("Discord:UserStats:{user_id}");

    let mut stats: UserMessageStats = match self.redis.get(&user_stats_key).await {
      Ok(Some(d)) => serde_json::from_str(&d).unwrap_or_default(),
      Ok(None) => UserMessageStats::default(),
      Err(_) => UserMessageStats::default()
    };

    stats.messages.retain(|t| timestamp - *t <= 5);
    stats.messages.push(timestamp);

    let data = serde_json::to_string(&stats).unwrap();
    self.redis.set(&user_stats_key, &data).await.unwrap();

    stats.messages.len() >= 4
  }

  fn contains_prohibited_words(
    &self,
    content: &str
  ) -> bool {
    self.pw_list.iter().any(|r| r.is_match(content))
  }

  fn contains_prohibited_urls(
    &self,
    content: &str
  ) -> bool {
    let mut domains = Vec::new();
    for cap in URL_REGEX.captures_iter(content) {
      if let Some(domain) = cap.get(1) {
        domains.push(domain.as_str().to_lowercase());
      }
    }

    for domain in domains {
      for prohibited in &self.pu_list {
        if domain == *prohibited || domain.ends_with(&format!(".{prohibited}")) {
          return true;
        }
      }
    }

    false
  }

  fn contains_invite_links(
    &self,
    content: &str
  ) -> bool {
    INVITE_REGEX.is_match(content)
  }

  async fn contains_malicious_links(
    &self,
    content: &str
  ) -> bool {
    if let Err(e) = self.update_malicious_domains().await {
      eprintln!("failed to update malicious domains: {e}");
    }

    let domains: Vec<String> = match self.redis.get(MD_KEY_MAIN).await.unwrap_or(None) {
      Some(json) => serde_json::from_str(&json).unwrap_or_default(),
      None => Vec::new()
    };

    // Check regular URLs
    for cap in URL_REGEX.captures_iter(content) {
      if let Some(domain) = cap.get(1) {
        if domains.iter().any(|d| &domain.as_str().to_lowercase() == d) {
          return true;
        }
      }
    }

    // Check masked URLs
    for cap in MASKED_URL_REGEX.captures_iter(content) {
      if let Some(url) = cap.get(1) {
        if let Ok(parsed) = Url::parse(url.as_str()) {
          if let Some(domain) = parsed.host_str() {
            if domains.iter().any(|d| &domain.to_lowercase() == d) {
              return true;
            }
          }
        }
      }
    }

    false
  }

  async fn update_malicious_domains(&self) -> Result<(), BotError> {
    let last_update = match self.redis.get(MD_KEY_LU).await? {
      Some(ts) => ts.parse::<i64>().unwrap_or(0),
      None => 0
    };

    let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;

    // run an update if hour+ old
    if current_time - last_update <= 3600 {
      return Ok(());
    }

    let initial_cap = match self.redis.get(MD_KEY_MAIN).await? {
      Some(j) => {
        let domains: Vec<String> = serde_json::from_str(&j).unwrap_or_default();
        (domains.len() + (domains.len() / 10)).max(50000)
      },
      None => 80000 // fallback to 80k if Redis key doesn't exist!
    };

    let mut domains = Vec::with_capacity(initial_cap);
    let mut success = 0;
    let mut total = 0;

    for url in MD_BLOCKLIST.iter() {
      match REQWEST_CLIENT
        .get(*url)
        .header("User-Agent", "Daggerbot - MaliciousDomains Scanner")
        .header("Authorization", format!("Token {}", token_path().await.octokit))
        .timeout(Duration::from_secs(10))
        .send()
        .await
      {
        Ok(r) => {
          if !r.status().is_success() {
            eprintln!("MaliciousDomains[Debug] {url} returned status {}", r.status());
            continue;
          }

          match r.text().await {
            Ok(txt) => {
              let entries = Self::process_response_text(txt).await;
              let count = entries.len();
              domains.extend(entries);
              total += count;
              success += 1;
            },
            Err(e) => eprintln!("MaliciousDomains[Err] {url} reported an error: {e}")
          }
        },
        Err(e) => eprintln!("MaliciousDomains[Err] {url} didn't want to respond: {e}")
      }
    }

    println!(
      "MaliciousDomains[Info] Refreshed from {success} of {} sources, {total} domains total",
      MD_BLOCKLIST.len()
    );

    if !domains.is_empty() {
      let domains_json = serde_json::to_string(&domains)?;
      self.redis.set(MD_KEY_MAIN, &domains_json).await?;
      self.redis.set(MD_KEY_LU, &current_time.to_string()).await?;

      println!("MaliciousDomains[Info] Cache refreshed | {} domains total", domains.len());
    }

    Ok(())
  }

  async fn process_response_text(text: String) -> Vec<String> {
    text
      .lines()
      .map(|line| line.trim().to_lowercase())
      .filter(|line| !line.is_empty())
      .collect()
  }

  async fn log_violation(
    &self,
    ctx: &Context,
    msg: &Message,
    policy: &AutomodPolicy,
    case_id: i32
  ) -> Result<(), BotError> {
    let log_channel = match policy.action {
      ActionType::Ban | ActionType::Kick => LogChannel::BansAndKicks,
      _ => LogChannel::BotLog
    };

    let bot_user = ctx.cache.current_user().id;
    let reason = format!("(Automod) {}", policy.reason);

    let mut fields = vec![
      (
        "User",
        format!("{}\n{}\n`{}`", msg.author.name, msg.author.mention(), msg.author.id),
        true
      ),
      (
        "Moderator",
        format!("{}\n{}\n`{bot_user}`", bot_user.to_user(ctx).await?.name, bot_user.mention()),
        true
      ),
      ("\u{200B}", "\u{200B}".to_string(), true),
      ("Reason", reason.clone(), true),
    ];

    if let Some(duration) = policy.mute_duration {
      fields.push(("Duration", format_duration(duration as u64), false));
    }

    let embed = CreateEmbed::default()
      .color(BINARY_PROPERTIES.embed_colors.primary)
      .title(format!("{} | Case #{case_id}", policy.action))
      .timestamp(msg.timestamp)
      .fields(fields);

    let channel = ctx
      .http()
      .get_channel(log_channel.to_discord())
      .await?
      .guild()
      .expect("Log channel not found");

    channel.send_message(&ctx.http, CreateMessage::new().embed(embed)).await?;

    send_notification(
      ctx,
      msg,
      &Target::User(msg.author.clone()),
      &policy.action,
      &reason,
      case_id,
      policy.mute_duration.map(|d| d as u64)
    )
    .await?;

    Ok(())
  }

  async fn handle_violation(
    &self,
    ctx: &Context,
    msg: &Message,
    policy: AutomodPolicy
  ) -> Result<(), BotError> {
    let user_id = msg.author.id.get();
    let user_stats_key = format!("Discord:UserStats:{user_id}");
    let user_stats: UserMessageStats = match self.redis.get(&user_stats_key).await? {
      Some(d) => serde_json::from_str(&d)?,
      None => UserMessageStats::default()
    };

    let postgres = &ctx.data::<BotData>().postgres.clone();
    let case_id = generate_id(postgres).await?;

    if (Sanctions::load_data(postgres, case_id).await?).is_some() {
      eprintln!(
        "[automod::handle_violation] attempted to create case entry but database already has it: #{case_id} - {}",
        msg.author.name
      );
      return Ok(())
    }

    let current_ts = msg.timestamp.unix_timestamp();
    let new_warnings = user_stats.increment_warnings(&policy.policy_type, current_ts);

    let user_stats_data = serde_json::to_string(&user_stats)?;
    self.redis.set(&user_stats_key, &user_stats_data).await?;

    {
      let reply_to_msg = match policy.policy_type {
        AutomodPolicyType::AntiSpam => "Stop spamming!",
        AutomodPolicyType::InviteLinks => "Discord invite links aren't allowed in this server!",
        AutomodPolicyType::ProhibitedWords => "Watch your language!",
        AutomodPolicyType::MaliciousLinks => "Phishing links aren't allowed in this server!",
        AutomodPolicyType::ProhibitedUrls => "That link is currently banned in this server!"
      };

      if let Ok(reply) = msg.reply(&ctx.http, reply_to_msg).await {
        let http = ctx.http.clone();
        let reply_id = reply.id;
        let channel_id = reply.channel_id;
        tokio::spawn(async move {
          sleep(Duration::from_secs(10)).await;
          if let Err(e) = channel_id.delete_message(&http, reply_id, None).await {
            eprintln!("[automod::delete_reply_message] Failed to delete the bot's reply message: {e}");
          }
        });
      } else {
        eprintln!("[automod::reply_message] Failed to reply to user's message");
      }

      if let Err(e) = msg.delete(&ctx.http, Some("Message violated the automod's policy!")).await {
        eprintln!("[automod::delete_message] Failed to delete the message: {e}");
      }
    }

    let should_action = new_warnings >= policy.warn_threshold;
    if should_action {
      user_stats.reset_warnings(&policy.policy_type);

      // Save state to external cache
      let user_stats_data = serde_json::to_string(&user_stats)?;
      self.redis.set(&user_stats_key, &user_stats_data).await?;

      self.log_violation(ctx, msg, &policy, case_id).await?;

      match policy.action {
        ActionType::Warn => self.create_sanction(ctx, msg.author.id, "Warn", &policy.reason, None, case_id).await?,
        ActionType::Mute => {
          if let Some(duration) = policy.mute_duration {
            let guild_id = msg.guild_id.expect("Expected message to be in guild");
            if let Ok(mut member) = guild_id.member(&ctx.http, msg.author.id).await {
              let until = Timestamp::from_unix_timestamp(msg.timestamp.unix_timestamp() + duration).expect("Invalid timestamp");
              member.disable_communication_until(&ctx.http, until).await?;
              self
                .create_sanction(ctx, msg.author.id, "Mute", &policy.reason, Some(duration), case_id)
                .await?;
            }
          }
        },
        ActionType::Kick => {
          if let Ok(member) = msg.member(&ctx.http).await {
            member.kick(&ctx.http, Some(&policy.reason)).await?;
            self.create_sanction(ctx, msg.author.id, "Kick", &policy.reason, None, case_id).await?;
          }
        },
        ActionType::Ban => {
          let guild_id = msg.guild_id.expect("Expected message to be in guild");
          guild_id.ban(&ctx.http, msg.author.id, 86400, Some(&policy.reason)).await?;
          self.create_sanction(ctx, msg.author.id, "Ban", &policy.reason, None, case_id).await?;
        },
        ActionType::Softban => {
          let guild_id = msg.guild_id.expect("Expected message to be in guild");
          guild_id.ban(&ctx.http, msg.author.id, 86400, Some(&policy.reason)).await?;
          guild_id.unban(&ctx.http, msg.author.id, None).await?;
          self.create_sanction(ctx, msg.author.id, "Softban", &policy.reason, None, case_id).await?;
        },
        _ => println!("[automod::should_action] Unknown ActionType ended up here!")
      }
    }

    Ok(())
  }

  async fn create_sanction(
    &self,
    ctx: &Context,
    user_id: UserId,
    action_type: &str,
    reason: &str,
    duration: Option<i64>,
    case_id: i32
  ) -> Result<(), BotError> {
    let timestamp = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("System time is lagging behind or is in the future")
      .as_secs() as i64;

    let member_name = user_id.to_user(ctx.http()).await?.name;
    let (moderator_name, moderator_id) = {
      let cu = ctx.cache.current_user();
      (cu.name.clone(), cu.id.to_string())
    };

    let sanction = Sanctions {
      case_id,
      case_type: Cow::Borrowed(action_type).into_owned(),
      member_name: Cow::Borrowed(&member_name).into_owned().to_string(),
      member_id: user_id.to_string(),
      moderator_name: Cow::Borrowed(&moderator_name).into_owned().to_string(),
      moderator_id,
      timestamp,
      end_time: duration.map(|d| timestamp + d),
      duration,
      reason: Cow::Borrowed(reason).into_owned()
    };

    sanction.create(&ctx.data::<BotData>().postgres).await?;
    Ok(())
  }

  async fn load_prohibited_words(db: &PgPool) -> Result<Vec<Regex>, BotError> {
    let words = ProhibitedWords::get_words(db).await?;

    let regexes = words
      .into_iter()
      .filter_map(|w| {
        Regex::new(&format!(r"(?i)\b{}(?:ing|ed|s|[0-9]*)?\b", regex::escape(&w.word)))
          .map_err(|e| eprintln!("Invalid word pattern ({}): {e}", w.word))
          .ok()
      })
      .collect();

    Ok(regexes)
  }

  async fn load_prohibited_urls(db: &PgPool) -> Result<Vec<String>, BotError> {
    let urls = ProhibitedUrls::get_urls(db).await?;
    let domains = urls.into_iter().map(|u| u.url.to_lowercase()).collect();
    Ok(domains)
  }
}

/// Send a notification to a user about a moderation action
async fn send_notification(
  ctx: &Context,
  msg: &Message,
  target: &Target,
  action: &ActionType,
  reason: &str,
  case_id: i32,
  duration: Option<u64>
) -> Result<bool, BotError> {
  let user = match target {
    Target::User(user) => user,
    Target::Member(mem) => &mem.user
  };

  let description = format!(
    "You've been **{}** in **{}** for:```\n{reason}\n```",
    match action {
      ActionType::Ban => "banned",
      ActionType::Kick => "kicked",
      ActionType::Mute => "timed out",
      ActionType::Softban => "softbanned",
      ActionType::Warn => "warned",
      _ => ""
    },
    msg.guild_id.unwrap().to_partial_guild(&ctx.http).await?.name
  );

  let mut fields = vec![("Case ID", case_id.to_string(), true)];

  if let Some(duration) = duration {
    let d = parse_duration::parse(&duration.to_string()).unwrap();
    fields.insert(1, ("Duration", format_duration(d.as_secs()), true));
  }

  let embed = CreateEmbed::new()
    .color(BINARY_PROPERTIES.embed_colors.primary)
    .title("Notice from automoderator")
    .fields(fields)
    .description(description);

  match user.id.direct_message(&ctx.http, CreateMessage::new().embed(embed)).await {
    Ok(_) => {
      println!("[automod::send_notification] (#{case_id}:{}) Sent DM with reason \"{reason}\"", user.name);
      Ok(true)
    },
    Err(e) => {
      eprintln!("[automod::send_notification] (#{case_id}:{}) Send DM failed with error: {e}", user.name);
      Ok(false)
    }
  }
}
