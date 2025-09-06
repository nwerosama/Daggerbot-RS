use {
  super::QUERY_FAILED,
  asahi::error,
  sqlx::{
    PgPool,
    Result
  }
};

#[derive(Clone, PartialEq)]
pub struct ProhibitedWords {
  pub word: String
}

impl ProhibitedWords {
  pub async fn get_words(pool: &PgPool) -> Result<Vec<ProhibitedWords>> {
    match sqlx::query_as!(ProhibitedWords, "SELECT word FROM prohibited_words")
      .fetch_all(pool)
      .await
    {
      Ok(r) => Ok(r),
      Err(e) => {
        error!("{QUERY_FAILED}\n{e}");
        Err(e)
      }
    }
  }

  pub async fn add_word(
    pool: &PgPool,
    word: &str
  ) -> Result<()> {
    match sqlx::query!("INSERT INTO prohibited_words (word) VALUES ($1) ON CONFLICT DO NOTHING", word)
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

  pub async fn remove_word(
    pool: &PgPool,
    word: &str
  ) -> Result<()> {
    match sqlx::query!("DELETE FROM prohibited_words WHERE word = $1", word).execute(pool).await {
      Ok(_) => Ok(()),
      Err(e) => {
        error!("{QUERY_FAILED}\n{e}");
        Err(e)
      }
    }
  }
}
