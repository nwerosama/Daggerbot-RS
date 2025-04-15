use {
  crate::{
    BotData,
    BotError
  },
  lazy_static::lazy_static,
  poise::serenity_prelude::async_trait,
  std::{
    collections::HashSet,
    sync::{
      Arc,
      Mutex
    }
  },
  tokio::time::{
    Duration,
    interval
  }
};

lazy_static! {
  static ref RUNNING_TASKS: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

#[async_trait]
pub trait TaskScheduler: Send + Sync {
  fn name(&self) -> &'static str;
  fn interval_secs(&self) -> u64;
  async fn main_loop(
    &self,
    bot_data: Arc<BotData>
  ) -> Result<(), BotError>;
}

pub async fn spawn<T: TaskScheduler + 'static>(
  t: T,
  d: Arc<BotData>
) {
  let t_name = t.name().to_string();
  let t_int = t.interval_secs();

  {
    let mut running_tasks = RUNNING_TASKS.lock().unwrap();
    if running_tasks.contains(&t_name) {
      println!("TaskScheduler({t_name}) Another thread is already running, exiting...");
      return
    }
    running_tasks.insert(t_name.clone());
  }

  tokio::spawn(async move {
    println!("TaskScheduler({t_name}) Running!");
    let mut int = interval(Duration::from_secs(t_int));

    loop {
      int.tick().await;

      if let Err(y) = t.main_loop(d.clone()).await {
        eprintln!("TaskScheduler({t_name}) Error: {y}")
      }
    }
  });
}
