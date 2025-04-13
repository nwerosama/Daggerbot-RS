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

#[derive(FromRow)]
pub struct Webhooks {
  pub name:       String,
  pub thread_id:  String,
  pub message_id: String,
  pub id:         String,
  pub token:      String
}

impl Webhooks {
  pub async fn get_hooks(pool: &PgPool) -> Result<Vec<Self>> {
    let q = sqlx::query("SELECT * FROM webhooks").fetch_all(pool).await;

    let mut hooks = Vec::new();

    match q {
      Ok(r) => {
        for row in r {
          hooks.push(Self {
            name:       row.get("name"),
            thread_id:  row.get("thread_id"),
            message_id: row.get("message_id"),
            id:         row.get("id"),
            token:      row.get("token")
          });
        }
      },
      Err(e) => {
        eprintln!("{DAG_SQL}[Database:Webhooks:get_hooks:Error] {QUERY_FAILED}\n{e}");
        return Err(e);
      }
    }

    Ok(hooks)
  }
}
