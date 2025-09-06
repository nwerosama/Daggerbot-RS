use {
  super::QUERY_FAILED,
  sqlx::{
    PgPool,
    Result
  }
};

pub struct Webhooks {
  pub name:       String,
  pub thread_id:  String,
  pub message_id: String,
  pub id:         String,
  pub token:      String
}

impl Webhooks {
  pub async fn get_hooks(pool: &PgPool) -> Result<Vec<Self>> {
    match sqlx::query_as!(Webhooks, "SELECT * FROM webhooks").fetch_all(pool).await {
      Ok(h) => Ok(h),
      Err(e) => {
        asahi::error!("{QUERY_FAILED}\n{e}");
        Err(e)
      }
    }
  }
}
