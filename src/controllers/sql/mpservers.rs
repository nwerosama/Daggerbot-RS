use super::{
  DAG_SQL,
  QUERY_FAILED
};

use {
  asahi::error,
  serde::{
    Deserialize,
    Serialize
  },
  sqlx::{
    FromRow,
    PgPool,
    Result
  }
};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
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
    match sqlx::query_as::<_, Self>(
      "SELECT name, is_active, ip,
        code, game_password, peak_players
      FROM mpservers ORDER BY name"
    )
    .fetch_all(pool)
    .await
    {
      Ok(servers) => Ok(servers),
      Err(e) => {
        error!("{DAG_SQL}[Database:MpServers:get_servers:Error] {QUERY_FAILED}\n{e}");
        Err(e)
      }
    }
  }

  pub async fn get_server(
    pool: &PgPool,
    name: String
  ) -> Result<Option<Self>> {
    match sqlx::query_as::<_, Self>(
      "SELECT name, is_active, ip,
        code, game_password, peak_players
      FROM mpservers WHERE name = $1"
    )
    .bind(name)
    .fetch_optional(pool)
    .await
    {
      Ok(server) => Ok(server),
      Err(e) => {
        error!("{DAG_SQL}[Database:MpServers:get_server:Error] {QUERY_FAILED}\n{e}");
        Err(e)
      }
    }
  }

  pub async fn get_peak_players(
    pool: &PgPool,
    name: String
  ) -> Result<i32> {
    match sqlx::query_scalar::<_, i32>("SELECT peak_players FROM mpservers WHERE name = $1")
      .bind(name)
      .fetch_one(pool)
      .await
    {
      Ok(peak) => Ok(peak),
      Err(e) => {
        error!("{DAG_SQL}[Database:MpServers:get_peak_players:Error] {QUERY_FAILED}\n{e}");
        Err(e)
      }
    }
  }

  pub async fn get_player_data(
    pool: &PgPool,
    name: String
  ) -> Result<Vec<i32>> {
    match sqlx::query_scalar::<_, Vec<i32>>("SELECT player_data FROM mpservers WHERE name = $1")
      .bind(name)
      .fetch_one(pool)
      .await
    {
      Ok(data) => Ok(data),
      Err(e) => {
        error!("{DAG_SQL}[Database:MpServers:get_player_data:Error] {QUERY_FAILED}\n{e}");
        Err(e)
      }
    }
  }

  pub async fn reset_peak_players(
    pool: &PgPool,
    name: String
  ) -> Result<bool> {
    match sqlx::query_scalar::<_, i32>(
      "UPDATE mpservers
      SET peak_players = 0, last_peak_update = CURRENT_TIMESTAMP
      WHERE name = $1 AND (
        last_peak_update IS NULL OR last_peak_update < CURRENT_TIMESTAMP - INTERVAL '3 days'
      )
      RETURNING 1"
    )
    .bind(name)
    .fetch_optional(pool)
    .await
    {
      Ok(Some(_)) => Ok(true),
      Ok(None) => Ok(false),
      Err(e) => {
        error!("{DAG_SQL}[Database:MpServers:reset_peak_players:Error] {QUERY_FAILED}\n{e}");
        Err(e)
      }
    }
  }

  pub async fn update_peak_players(
    pool: &PgPool,
    name: String,
    current_players: i32
  ) -> Result<bool> {
    match sqlx::query_scalar::<_, i32>(
      "UPDATE mpservers
      SET peak_players = $1, last_peak_update = CURRENT_TIMESTAMP
      WHERE name = $2 AND peak_players < $1
      RETURNING 1"
    )
    .bind(current_players)
    .bind(name)
    .fetch_optional(pool)
    .await
    {
      Ok(Some(_)) => Ok(true),
      Ok(None) => Ok(false),
      Err(e) => {
        error!("{DAG_SQL}[Database:MpServers:update_peak_players:Error] {QUERY_FAILED}\n{e}");
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
    match sqlx::query(
      "UPDATE mpservers
      SET player_data = CASE
        WHEN array_length(player_data, 1) > 70 THEN ARRAY[$1]::int[]
        ELSE array_append(player_data, $1)
      END
      WHERE name = $2"
    )
    .bind(current_players)
    .bind(name)
    .execute(pool)
    .await
    {
      Ok(_) => Ok(()),
      Err(e) => {
        error!("{DAG_SQL}[Database:MpServers:update_player_data:Error] {QUERY_FAILED}\n{e}");
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
    let q = sqlx::query(
      "INSERT INTO mpservers (name, is_active, ip, code, game_password, peak_players, player_data)
      VALUES ($1, $2, $3, $4, $5, 0, '{0,0,0}')"
    )
    .bind(name)
    .bind(active)
    .bind(ip)
    .bind(code)
    .bind(password)
    .execute(pool)
    .await;

    match q {
      Ok(r) => Ok(r.rows_affected() > 0),
      Err(e) => {
        error!("{DAG_SQL}[Database:MpServers:create_server:Error] {QUERY_FAILED}\n{e}");
        Err(e)
      }
    }
  }

  pub async fn delete_server(
    pool: &PgPool,
    name: String
  ) -> Result<bool> {
    let q = sqlx::query("DELETE FROM mpservers WHERE name = $1").bind(name).execute(pool).await;

    match q {
      Ok(r) => Ok(r.rows_affected() > 0),
      Err(e) => {
        error!("{DAG_SQL}[Database:MpServers:delete_server:Error] {QUERY_FAILED}\n{e}");
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
    let q = sqlx::query(
      "UPDATE mpservers SET is_active = $1, ip = $2, code = $3, game_password = $4
      WHERE name = $5"
    )
    .bind(is_active)
    .bind(ip)
    .bind(code)
    .bind(game_password)
    .bind(name)
    .execute(pool)
    .await;

    match q {
      Ok(r) => Ok(r.rows_affected() > 0),
      Err(e) => {
        error!("{DAG_SQL}[Database:MpServers:update_server:Error] {QUERY_FAILED}\n{e}");
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
