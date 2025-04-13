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
pub struct ProhibitedWords {
  pub word: String
}

impl ProhibitedWords {
  pub async fn get_words(pool: &PgPool) -> Result<Vec<ProhibitedWords>> {
    let rows = match sqlx::query_as::<_, ProhibitedWords>("SELECT word FROM prohibited_words")
      .fetch_all(pool)
      .await
    {
      Ok(r) => r,
      Err(e) => {
        eprintln!("{DAG_SQL}[Database:ProhibitedWords:get_words:Error] {QUERY_FAILED}\n{e}");
        return Err(e)
      }
    };

    Ok(rows)
  }

  pub async fn add_word(
    pool: &PgPool,
    word: &str
  ) -> Result<()> {
    match sqlx::query("INSERT INTO prohibited_words (word) VALUES ($1) ON CONFLICT DO NOTHING")
      .bind(word)
      .execute(pool)
      .await
    {
      Ok(_) => (),
      Err(e) => {
        eprintln!("{DAG_SQL}[Database:ProhibitedWords:add_word:Error] {QUERY_FAILED}\n{e}");
        return Err(e)
      }
    };

    Ok(())
  }

  pub async fn remove_word(
    pool: &PgPool,
    word: &str
  ) -> Result<()> {
    match sqlx::query("DELETE FROM prohibited_words WHERE word = $1").bind(word).execute(pool).await {
      Ok(_) => (),
      Err(e) => {
        eprintln!("{DAG_SQL}[Database:ProhibitedWords:remove_word:Error] {QUERY_FAILED}\n{e}");
        return Err(e)
      }
    };

    Ok(())
  }
}
