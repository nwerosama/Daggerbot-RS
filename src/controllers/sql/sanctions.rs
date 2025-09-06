use {
  super::QUERY_FAILED,
  asahi::{
    error,
    info
  },
  dashmap::DashMap,
  lazy_static::lazy_static,
  sqlx::{
    PgPool,
    Result
  },
  tokio::time::Instant
};

const LOCK_SECONDS: u64 = 3;
lazy_static! {
  static ref LOCKS: DashMap<String, Instant> = DashMap::new();
}

#[derive(Clone)]
pub struct Sanctions {
  pub case_id:        i32,
  pub case_type:      String,
  pub member_name:    String,
  pub member_id:      String,
  pub moderator_name: String,
  pub moderator_id:   String,
  pub timestamp:      i64,         // Unix epoch
  pub end_time:       Option<i64>, // for bans and mutes
  pub duration:       Option<i64>, // for mutes
  pub reason:         String       // 255 characters max
}

pub struct ReturnedCase {
  pub case_id:     i32,
  pub case_type:   String,
  pub member_id:   String,
  pub member_name: String
}

impl Sanctions {
  pub async fn load_data(
    pool: &PgPool,
    case_id: i32
  ) -> Result<Option<Self>> {
    if let Some(r) = sqlx::query_as!(Sanctions, "SELECT * FROM sanctions WHERE case_id = $1", case_id)
      .fetch_optional(pool)
      .await?
    {
      Ok(Some(r))
    } else {
      Ok(None)
    }
  }

  pub fn acquire_lock(user_id: &str) -> bool {
    LOCKS.retain(|_, t| t.elapsed().as_secs() < LOCK_SECONDS);
    if let Some(lt) = LOCKS.get(user_id)
      && lt.elapsed().as_secs() < LOCK_SECONDS
    {
      return false // It's locked if false
    }
    LOCKS.insert(user_id.to_string(), Instant::now());
    true // True if lock acquired
  }

  pub async fn create(
    &self,
    pool: &PgPool
  ) -> Result<Self> {
    if !Self::acquire_lock(&self.member_id) {
      info!("{} is already being moderated!", self.member_name);
      return Ok(self.clone())
    }

    let q = sqlx::query!(
      "INSERT INTO sanctions (
        case_id, case_type,
        member_name, member_id,
        moderator_name, moderator_id,
        timestamp, end_time,
        duration, reason
      ) VALUES (
        $1, $2, $3, $4,
        $5, $6, $7,
        $8, $9, $10
      ) RETURNING case_id",
      self.case_id,
      self.case_type.clone(),
      self.member_name.clone(),
      self.member_id.clone(),
      self.moderator_name.clone(),
      self.moderator_id.clone(),
      self.timestamp,
      self.end_time,
      self.duration,
      self.reason.clone()
    )
    .fetch_one(pool)
    .await;

    match q {
      Ok(r) => Ok(Self {
        case_id: r.case_id,
        ..self.clone()
      }),
      Err(e) => {
        error!("{QUERY_FAILED}\n{e}");
        Err(e)
      }
    }
  }

  pub async fn get_cases(pool: &PgPool) -> Result<Vec<ReturnedCase>> {
    match sqlx::query_as!(ReturnedCase, "SELECT case_id, case_type, member_id, member_name FROM sanctions")
      .fetch_all(pool)
      .await
    {
      Ok(cases) => Ok(cases),
      Err(e) => {
        error!("{QUERY_FAILED}\n{e}");
        Err(e)
      }
    }
  }
}
