pub mod monica;

pub use monica::monica;

use {
  lazy_static::lazy_static,
  poise::serenity_prelude::Context,
  std::{
    collections::HashSet,
    future::Future,
    sync::{
      Arc,
      Mutex
    },
    time::{
      Duration,
      Instant
    }
  },
  tokio::{
    task::spawn,
    time::sleep
  }
};

lazy_static! {
  static ref RUNNING_TASKS: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

fn task_info(
  name: &str,
  message: &str
) {
  println!("TaskScheduler[{name}] {message}")
}

fn task_err(
  name: &str,
  message: &str
) {
  eprintln!("TaskScheduler[{name}:Error] {message}")
}

fn get_backoff_duration(attempts: u32) -> Duration {
  // Back off up to 5 minutes
  Duration::from_secs(2u64.pow(attempts).min(300))
}

pub async fn run_task<F, T>(
  ctx: Arc<Context>,
  task: F,
  id: &str
) where
  F: Fn(Arc<Context>) -> T + Send + 'static,
  T: Future<Output = Result<(), crate::BotError>> + Send + 'static
{
  let task_id = id.to_string();

  {
    let mut running_tasks = RUNNING_TASKS.lock().unwrap();
    if running_tasks.contains(&task_id) {
      task_info(&task_id, "Task is already running, avoiding duplication...");
      return;
    }
    running_tasks.insert(task_id.clone());
  }

  spawn(async move {
    let mut attempts = 0;
    let mut last_error = Instant::now();

    loop {
      match task(Arc::clone(&ctx)).await {
        Ok(()) => {
          task_info(&task_id, "Task no longer running, removing from the running tasks list...");
          break;
        },
        Err(e) => {
          if last_error.elapsed() >= Duration::from_secs(60) && attempts > 0 {
            task_info(
              &task_id,
              &format!(
                "Resetting attempts counter after {:.1}m of stability",
                last_error.elapsed().as_secs_f32() / 60.0
              )
            );
            attempts = 0;
          }

          attempts += 1;
          last_error = Instant::now();
          task_err(&task_id, &format!("Failed to execute the task, error reason: {e}"));

          if let Some(src) = e.source() {
            task_err(&task_id, &format!("Caused by: {src:#?}"));
          }
          if attempts >= 10 {
            task_err(&task_id, "Recovery unsuccessful, stopping...");
            break;
          } else {
            let backoff = get_backoff_duration(attempts);
            task_err(
              &task_id,
              &format!("Task recovery attempt ({attempts}), retrying in {} seconds...", backoff.as_secs())
            );
            sleep(backoff).await;
          }
        }
      }
    }

    let mut running_tasks = RUNNING_TASKS.lock().unwrap();
    running_tasks.remove(&task_id);
  });
}
