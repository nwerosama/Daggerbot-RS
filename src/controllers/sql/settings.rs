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

#[derive(Debug, Clone, FromRow)]
pub struct Settings {
  pub logs_ignored_channels: Vec<i64>
}

impl Settings {
  pub async fn get_logs_ignored_channels(pool: &PgPool) -> Result<Vec<i64>> {
    let row_exists = sqlx::query("SELECT EXISTS(SELECT 1 FROM settings WHERE id = 1)")
      .fetch_one(pool)
      .await?
      .get::<bool, _>("exists");

    if !row_exists {
      sqlx::query("INSERT INTO settings (id, logs_ignored_channels) VALUES (1, '{}')")
        .execute(pool)
        .await?;
    }

    let q = sqlx::query_as::<_, Self>("SELECT logs_ignored_channels FROM settings WHERE id = 1")
      .fetch_one(pool)
      .await;

    if let Err(e) = q {
      eprintln!("{DAG_SQL}[Database:Settings:get_logs_ignored_channels:Error] {QUERY_FAILED}\n{e}");
      return Err(e);
    };

    Ok(q.unwrap().logs_ignored_channels)
  }

  pub async fn update_logs_ignored_channels(
    &self,
    pool: &PgPool
  ) -> Result<()> {
    let q = sqlx::query("UPDATE settings SET logs_ignored_channels = $1 WHERE id = 1")
      .bind(&self.logs_ignored_channels)
      .execute(pool)
      .await;

    if let Err(e) = q {
      eprintln!("{DAG_SQL}[Database:Settings:update_logs_ignored_channels:Error] {QUERY_FAILED}\n{e}");
      return Err(e);
    };

    Ok(())
  }
}
