use super::{
  DAG_SQL,
  QUERY_FAILED
};

use sqlx::{
  FromRow,
  PgPool,
  Result
};

#[derive(Clone, FromRow, PartialEq)]
pub struct ProhibitedUrls {
  pub url: String
}

impl ProhibitedUrls {
  pub async fn get_urls(pool: &PgPool) -> Result<Vec<ProhibitedUrls>> {
    let rows = match sqlx::query_as::<_, ProhibitedUrls>("SELECT url FROM prohibited_urls")
      .fetch_all(pool)
      .await
    {
      Ok(r) => r,
      Err(e) => {
        eprintln!("{DAG_SQL}[Database:ProhibitedUrls:get_urls:Error] {QUERY_FAILED}\n{e}");
        return Err(e)
      }
    };

    Ok(rows)
  }

  pub async fn add_url(
    pool: &PgPool,
    url: &str
  ) -> Result<()> {
    match sqlx::query("INSERT INTO prohibited_urls (url) VALUES ($1) ON CONFLICT DO NOTHING")
      .bind(url)
      .execute(pool)
      .await
    {
      Ok(_) => (),
      Err(e) => {
        eprintln!("{DAG_SQL}[Database:ProhibitedUrls:add_url:Error] {QUERY_FAILED}\n{e}");
        return Err(e)
      }
    };

    Ok(())
  }

  pub async fn remove_url(
    pool: &PgPool,
    url: &str
  ) -> Result<()> {
    match sqlx::query("DELETE FROM prohibited_urls WHERE url = $1").bind(url).execute(pool).await {
      Ok(_) => (),
      Err(e) => {
        eprintln!("{DAG_SQL}[Database:ProhibitedUrls:remove_url:Error] {QUERY_FAILED}\n{e}");
        return Err(e)
      }
    };

    Ok(())
  }
}
