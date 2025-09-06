use {
  super::QUERY_FAILED,
  asahi::error,
  sqlx::{
    PgPool,
    Result
  }
};

#[derive(Clone, PartialEq)]
pub struct ProhibitedUrls {
  pub url: String
}

impl ProhibitedUrls {
  pub async fn get_urls(pool: &PgPool) -> Result<Vec<ProhibitedUrls>> {
    match sqlx::query_as!(ProhibitedUrls, "SELECT url FROM prohibited_urls").fetch_all(pool).await {
      Ok(r) => Ok(r),
      Err(e) => {
        error!("{QUERY_FAILED}\n{e}");
        Err(e)
      }
    }
  }

  pub async fn add_url(
    pool: &PgPool,
    url: &str
  ) -> Result<()> {
    match sqlx::query!("INSERT INTO prohibited_urls (url) VALUES ($1) ON CONFLICT DO NOTHING", url)
      .execute(pool)
      .await
    {
      Ok(_) => Ok(()),
      Err(e) => {
        error!("{QUERY_FAILED}\n{e}");
        Err(e)
      }
    }
  }

  pub async fn remove_url(
    pool: &PgPool,
    url: &str
  ) -> Result<()> {
    match sqlx::query!("DELETE FROM prohibited_urls WHERE url = $1", url).execute(pool).await {
      Ok(_) => Ok(()),
      Err(e) => {
        error!("{QUERY_FAILED}\n{e}");
        Err(e)
      }
    }
  }
}
