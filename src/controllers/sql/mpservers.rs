use {
  super::QUERY_FAILED,
  asahi::error,
  serde::{
    Deserialize,
    Serialize
  },
  sqlx::{
    PgPool,
    Result
  }
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MpServers {
  pub name:          String,
  pub is_active:     bool,
  pub ip:            String,
  pub code:          String,
  pub game_password: String,
  pub peak_players:  i32
}

impl MpServers {
  pub async fn get_servers(pool: &PgPool) -> Result<Vec<Self>> {
    match sqlx::query_as!(
      MpServers,
      "SELECT name, is_active, ip, code, game_password, peak_players
      FROM mpservers ORDER BY name"
    )
    .fetch_all(pool)
    .await
    {
      Ok(servers) => Ok(servers),
      Err(e) => {
        error!("{QUERY_FAILED}\n{e}");
        Err(e)
      }
    }
  }

  pub async fn get_server(
    pool: &PgPool,
    name: String
  ) -> Result<Option<Self>> {
    match sqlx::query_as!(
      MpServers,
      "SELECT name, is_active, ip, code, game_password, peak_players
      FROM mpservers WHERE name = $1",
      name
    )
    .fetch_optional(pool)
    .await
    {
      Ok(server) => Ok(server),
      Err(e) => {
        error!("{QUERY_FAILED}\n{e}");
        Err(e)
      }
    }
  }

  pub async fn get_peak_players(
    pool: &PgPool,
    name: String
  ) -> Result<i32> {
    match sqlx::query_scalar!("SELECT peak_players FROM mpservers WHERE name = $1", name)
      .fetch_one(pool)
      .await
    {
      Ok(peak) => Ok(peak),
      Err(e) => {
        error!("{QUERY_FAILED}\n{e}");
        Err(e)
      }
    }
  }

  pub async fn get_player_data(
    pool: &PgPool,
    name: String
  ) -> Result<Vec<i32>> {
    match sqlx::query_scalar!("SELECT player_data FROM mpservers WHERE name = $1", name)
      .fetch_one(pool)
      .await
    {
      Ok(data) => Ok(data),
      Err(e) => {
        error!("{QUERY_FAILED}\n{e}");
        Err(e)
      }
    }
  }

  pub async fn reset_peak_players(
    pool: &PgPool,
    name: String
  ) -> Result<bool> {
    match sqlx::query_scalar!(
      "UPDATE mpservers
      SET peak_players = 0, last_peak_update = CURRENT_TIMESTAMP
      WHERE name = $1 AND (
        last_peak_update IS NULL OR last_peak_update < CURRENT_TIMESTAMP - INTERVAL '3 days'
      )
      RETURNING 1",
      name
    )
    .fetch_optional(pool)
    .await
    {
      Ok(Some(_)) => Ok(true),
      Ok(None) => Ok(false),
      Err(e) => {
        error!("{QUERY_FAILED}\n{e}");
        Err(e)
      }
    }
  }

  pub async fn update_peak_players(
    pool: &PgPool,
    name: String,
    current_players: i32
  ) -> Result<bool> {
    match sqlx::query_scalar!(
      "UPDATE mpservers
      SET peak_players = $1, last_peak_update = CURRENT_TIMESTAMP
      WHERE name = $2 AND peak_players < $1 RETURNING 1",
      current_players,
      name
    )
    .fetch_optional(pool)
    .await
    {
      Ok(Some(_)) => Ok(true),
      Ok(None) => Ok(false),
      Err(e) => {
        error!("{QUERY_FAILED}\n{e}");
        Err(e)
      }
    }
  }

  pub async fn update_player_data(
    pool: &PgPool,
    name: String,
    current_players: i32
  ) -> Result<()> {
    // Selfnote: 3150/45 = 220, where 3150 is the max PD size and 45 is Monica's update interval
    //           70 points * 45 seconds = 3150 seconds = 52.5 minutes
    match sqlx::query!(
      "UPDATE mpservers
      SET player_data = CASE
        WHEN array_length(player_data, 1) > 70 THEN ARRAY[$1]::int[]
        ELSE array_append(player_data, $1)
      END
      WHERE name = $2",
      current_players,
      name
    )
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

  pub async fn create_server(
    pool: &PgPool,
    name: String,
    ip: String,
    code: String,
    password: String,
    active: bool
  ) -> Result<bool> {
    match sqlx::query!(
      "INSERT INTO mpservers (name, ip, code, is_active, game_password, peak_players, player_data)
      VALUES ($1, $2, $3, $4, $5, 0, '{0,0}')",
      name,
      ip,
      code,
      active,
      password
    )
    .execute(pool)
    .await
    {
      Ok(r) => Ok(r.rows_affected() > 0),
      Err(e) => {
        error!("{QUERY_FAILED}\n{e}");
        Err(e)
      }
    }
  }

  pub async fn delete_server(
    pool: &PgPool,
    name: String
  ) -> Result<bool> {
    match sqlx::query!("DELETE FROM mpservers WHERE name = $1", name).execute(pool).await {
      Ok(r) => Ok(r.rows_affected() > 0),
      Err(e) => {
        error!("{QUERY_FAILED}\n{e}");
        Err(e)
      }
    }
  }

  pub async fn update_server(
    pool: &PgPool,
    name: String,
    is_active: bool,
    ip: String,
    code: String,
    game_password: String
  ) -> Result<bool> {
    match sqlx::query!(
      "UPDATE mpservers SET ip = $1, code = $2, is_active = $3, game_password = $4
      WHERE name = $5",
      ip,
      code,
      is_active,
      game_password,
      name
    )
    .execute(pool)
    .await
    {
      Ok(r) => Ok(r.rows_affected() > 0),
      Err(e) => {
        error!("{QUERY_FAILED}\n{e}");
        Err(e)
      }
    }
  }
}

impl std::fmt::Display for MpServers {
  fn fmt(
    &self,
    f: &mut std::fmt::Formatter<'_>
  ) -> std::fmt::Result {
    write!(f, "{}", self.name)
  }
}
