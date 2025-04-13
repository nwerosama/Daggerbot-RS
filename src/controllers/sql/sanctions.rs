use super::{
  DAG_SQL,
  QUERY_FAILED
};

use sqlx::{
  FromRow,
  PgPool,
  Result,
  Row
};

#[derive(Clone, FromRow)]
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
    let q = sqlx::query("SELECT * FROM sanctions WHERE case_id = $1")
      .bind(case_id)
      .fetch_optional(pool)
      .await?;

    if let Some(r) = q {
      Ok(Some(Self {
        case_id:        r.get("case_id"),
        case_type:      r.get("case_type"),
        member_name:    r.get("member_name"),
        member_id:      r.get("member_id"),
        moderator_name: r.get("moderator_name"),
        moderator_id:   r.get("moderator_id"),
        timestamp:      r.get("timestamp"),
        end_time:       r.try_get("end_time").ok(),
        duration:       r.try_get("duration").ok(),
        reason:         r.get("reason")
      }))
    } else {
      Ok(None)
    }
  }

  pub async fn create(
    &self,
    pool: &PgPool
  ) -> Result<Self> {
    let q = sqlx::query(
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
      ) RETURNING case_id"
    )
    .bind(self.case_id)
    .bind(self.case_type.clone())
    .bind(self.member_name.clone())
    .bind(self.member_id.clone())
    .bind(self.moderator_name.clone())
    .bind(self.moderator_id.clone())
    .bind(self.timestamp)
    .bind(self.end_time)
    .bind(self.duration)
    .bind(self.reason.clone())
    .fetch_one(pool)
    .await;

    match q {
      Ok(r) => Ok(Self {
        case_id: r.get("case_id"),
        ..self.clone()
      }),
      Err(e) => {
        eprintln!("{DAG_SQL}[Database:Sanctions:create:Error] {QUERY_FAILED}\n{e}");
        Err(e)
      }
    }
  }

  pub async fn get_cases(pool: &PgPool) -> Result<Vec<ReturnedCase>> {
    let q = sqlx::query("SELECT case_id, case_type, member_id, member_name FROM sanctions")
      .fetch_all(pool)
      .await;

    match q {
      Ok(r) => {
        let cases = r
          .into_iter()
          .map(|row| ReturnedCase {
            case_id:     row.get("case_id"),
            case_type:   row.get("case_type"),
            member_id:   row.get("member_id"),
            member_name: row.get("member_name")
          })
          .collect();

        Ok(cases)
      },
      Err(e) => {
        eprintln!("{DAG_SQL}[Database:Sanctions:get_cases:Error] {QUERY_FAILED}\n{e}");
        Err(e)
      }
    }
  }
}
