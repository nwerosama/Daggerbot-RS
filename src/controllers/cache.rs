use {
  crate::internals::utils::token_path,
  bb8_redis::{
    RedisConnectionManager,
    bb8::Pool,
    redis::{
      AsyncCommands,
      RedisError,
      RedisResult,
      cmd
    }
  },
  std::io::Error,
  tokio::time::{
    Duration,
    sleep
  }
};

/// Boilerplate code for Redis commands
macro_rules! with_conn {
  ($self:expr, $cmd:ident($($arg:expr),*)) => {
    if let Ok(mut conn) = $self.pool.get().await {
      conn.$cmd($($arg),*).await
    } else {
      Err(RedisError::from(Error::other("Failed to get a connection!")))
    }
  }
}

#[derive(Clone)]
pub struct RedisController {
  pool: Pool<RedisConnectionManager>
}

impl RedisController {
  pub async fn new() -> Result<Self, RedisError> {
    let manager = RedisConnectionManager::new(token_path().await.redis_uri.as_str())?;
    let pool = Self::create_pool(manager).await;
    Ok(Self { pool })
  }

  async fn create_pool(manager: RedisConnectionManager) -> Pool<RedisConnectionManager> {
    let mut backoff = 1;

    loop {
      match Pool::builder().max_size(26).retry_connection(true).build(manager.clone()).await {
        Ok(pool) => match pool.get().await {
          Ok(mut conn) => {
            let ping: RedisResult<String> = cmd("PING").query_async(&mut *conn).await;
            match ping {
              Ok(_) => {
                println!("Redis[Info] Successfully connected");
                return pool.clone();
              },
              Err(e) => Self::backoff_from_error("", &e, &mut backoff).await
            }
          },
          Err(e) => {
            eprintln!("Redis[ConnError] {e}, retrying in {backoff} seconds");
            Self::apply_backoff(&mut backoff).await;
          }
        },
        Err(e) => Self::backoff_from_error("Pool", &e, &mut backoff).await
      }
    }
  }

  async fn backoff_from_error(
    s: &str,
    error: &RedisError,
    backoff: &mut u64
  ) {
    eprintln!("Redis[{s}Error] {error}, retrying in {backoff} seconds");
    Self::apply_backoff(backoff).await;
  }

  async fn apply_backoff(backoff: &mut u64) {
    sleep(Duration::from_secs(*backoff)).await;
    if *backoff < 64 {
      *backoff *= 2;
    }
  }

  /// Get a key from the cache
  pub async fn get(
    &self,
    key: &str
  ) -> RedisResult<Option<String>> {
    with_conn!(self, get(key))
  }

  /// Set a key with the value
  pub async fn set(
    &self,
    key: &str,
    value: &str
  ) -> RedisResult<()> {
    with_conn!(self, set(key, value))
  }

  /// Set a key with an expiration time in seconds
  pub async fn expire(
    &self,
    key: &str,
    seconds: i64
  ) -> RedisResult<()> {
    with_conn!(self, expire(key, seconds))
  }

  /// Delete a key from the cache if it exists
  pub async fn del(
    &self,
    key: &str
  ) -> RedisResult<()> {
    with_conn!(self, del(key))
  }
}
