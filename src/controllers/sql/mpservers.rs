use super::{
  DAG_SQL,
  QUERY_FAILED
};

use {
  serde::{
    Deserialize,
    Serialize
  },
  sqlx::{
    FromRow,
    PgPool,
    Result,
    Row,
    types::chrono::{
      DateTime,
      Utc
    }
  },
  std::time::{
    Duration,
    SystemTime
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
    let q = sqlx::query("SELECT * FROM mpservers").fetch_all(pool).await;

    let mut servers = Vec::new();

    match q {
      Ok(rows) => {
        for row in rows {
          servers.push(Self {
            name:          row.get("name"),
            is_active:     row.get("is_active"),
            ip:            row.get("ip"),
            code:          row.get("code"),
            game_password: row.get("game_password"),
            peak_players:  row.get("peak_players")
          })
        }
      },
      Err(e) => {
        eprintln!("{DAG_SQL}[Database:MpServers:get_servers:Error] {QUERY_FAILED}\n{e}");
        return Err(e);
      }
    }

    Ok(servers)
  }

  pub async fn get_server(
    pool: &PgPool,
    name: String
  ) -> Result<Option<Self>> {
    let q = sqlx::query("SELECT * FROM mpservers WHERE name = $1").bind(name).fetch_one(pool).await;

    match q {
      Ok(row) => Ok(Some(Self {
        name:          row.get("name"),
        is_active:     row.get("is_active"),
        ip:            row.get("ip"),
        code:          row.get("code"),
        game_password: row.get("game_password"),
        peak_players:  row.get("peak_players")
      })),
      Err(e) => {
        eprintln!("{DAG_SQL}[Database:MpServers:get_server:Error] {QUERY_FAILED}\n{e}");
        Ok(None)
      }
    }
  }

  pub async fn get_peak_players(
    pool: &PgPool,
    name: String
  ) -> Result<i32> {
    let q = sqlx::query("SELECT peak_players FROM mpservers WHERE name = $1")
      .bind(name)
      .fetch_one(pool)
      .await;

    match q {
      Ok(r) => Ok(r.get("peak_players")),
      Err(e) => {
        eprintln!("{DAG_SQL}[Database:MpServers:get_peak_players:Error] {QUERY_FAILED}\n{e}");
        Err(e)
      }
    }
  }

  pub async fn get_player_data(
    pool: &PgPool,
    name: String
  ) -> Result<Vec<i32>> {
    let q = sqlx::query("SELECT player_data FROM mpservers WHERE name = $1")
      .bind(name)
      .fetch_one(pool)
      .await;

    match q {
      Ok(r) => Ok(r.get("player_data")),
      Err(e) => {
        eprintln!("{DAG_SQL}[Database:MpServers:get_player_data:Error] {QUERY_FAILED}\n{e}");
        Err(e)
      }
    }
  }

  pub async fn reset_peak_players(
    pool: &PgPool,
    name: String
  ) -> Result<bool> {
    let q = sqlx::query("SELECT last_peak_update, peak_players FROM mpservers WHERE name = $1")
      .bind(name.clone())
      .fetch_one(pool)
      .await;

    match q {
      Ok(r) => {
        let last_peak_update: Option<DateTime<Utc>> = r.get("last_peak_update");

        if let Some(update_time) = last_peak_update {
          let update_sys_time: SystemTime = SystemTime::from(update_time);
          let current_time = SystemTime::now();

          if let Ok(durat_since_last) = current_time.duration_since(update_sys_time) {
            if durat_since_last >= Duration::from_secs(259200) {
              sqlx::query(
                "UPDATE mpservers SET peak_players = 0, last_peak_update = NOW()
                WHERE name = $1"
              )
              .bind(name)
              .execute(pool)
              .await?;
              return Ok(true);
            }
          }
        }
        Ok(false)
      },
      Err(e) => {
        eprintln!("{DAG_SQL}[Database:MpServers:reset_peak_players:Error] {QUERY_FAILED}\n{e}");
        Err(e)
      }
    }
  }

  pub async fn update_peak_players(
    pool: &PgPool,
    name: String,
    current_players: i32
  ) -> Result<bool> {
    let q = sqlx::query("SELECT peak_players FROM mpservers WHERE name = $1")
      .bind(name.clone())
      .fetch_one(pool)
      .await;

    match q {
      Ok(r) => {
        let peak_players: i32 = r.get("peak_players");

        if current_players > peak_players {
          sqlx::query(
            "UPDATE mpservers SET peak_players = $1, last_peak_update = NOW()
            WHERE name = $2"
          )
          .bind(current_players)
          .bind(name)
          .execute(pool)
          .await?;
          return Ok(true);
        }
        Ok(false)
      },
      Err(e) => {
        eprintln!("{DAG_SQL}[Database:MpServers:update_peak_players:Error] {QUERY_FAILED}\n{e}");
        Err(e)
      }
    }
  }

  pub async fn update_player_data(
    pool: &PgPool,
    name: String,
    current_players: i32
  ) -> Result<()> {
    let q = sqlx::query("SELECT player_data FROM mpservers WHERE name = $1")
      .bind(name.clone())
      .fetch_one(pool)
      .await;

    match q {
      Ok(r) => {
        let player_data: Vec<i32> = r.get("player_data");

        let mut player_data = player_data;
        if player_data.len() > 70 {
          player_data = Vec::new(); // Selfnote: 3150/45 = 220, where 3150 is the max PD size and 45 is Monica's update interval
        }
        player_data.push(current_players);

        sqlx::query(
          "UPDATE mpservers SET player_data = $1
          WHERE name = $2"
        )
        .bind(player_data)
        .bind(name)
        .execute(pool)
        .await?;
        Ok(())
      },
      Err(e) => {
        eprintln!("{DAG_SQL}[Database:MpServers:update_player_data:Error] {QUERY_FAILED}\n{e}");
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
        eprintln!("{DAG_SQL}[Database:MpServers:create_server:Error] {QUERY_FAILED}\n{e}");
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
        eprintln!("{DAG_SQL}[Database:MpServers:delete_server:Error] {QUERY_FAILED}\n{e}");
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
        eprintln!("{DAG_SQL}[Database:MpServers:update_server:Error] {QUERY_FAILED}\n{e}");
        Err(e)
      }
    }
  }
}
