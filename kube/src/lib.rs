use {
  serde::{
    Deserialize,
    Serialize
  },
  std::sync::Arc,
  tokio::sync::RwLock,
  warp::{
    Filter,
    http::StatusCode,
    reply::with_status,
    reply::json,
    Rejection
  }
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health {
  pub status:         String,
  pub ws_connected:   bool
}

#[derive(Debug, Clone)]
pub struct HealthProbe {
  pub status: Arc<RwLock<Health>>
}

impl Default for HealthProbe {
  fn default() -> Self { Self::new() }
}

impl HealthProbe {
  pub fn new() -> Self {
    Self {
      status: Arc::new(RwLock::new(Health {
        status:         "starting".to_string(),
        ws_connected:   false
      }))
    }
  }

  pub async fn update_ws_status(
    &self,
    connected: bool
  ) {
    asahi::info!(
      "health endpoint updated; ws_connected: {connected}"
    );
    let mut status = self.status.write().await;
    status.ws_connected = connected;
    status.status = if connected { "healthy".to_string() } else { "unhealthy".to_string() }
  }

  pub async fn init(
    &self,
    port: u16
  ) {
    let health_prober = self.clone();
    let readiness_prober = self.clone();

    let health = warp::path("health").and(warp::get()).and_then(move || {
      let prober = health_prober.clone();
      async move {
        let status = prober.status.read().await;
        let status_code = if status.ws_connected {
          StatusCode::OK
        } else {
          StatusCode::SERVICE_UNAVAILABLE
        };

        Ok::<_, Rejection>(with_status(json(&*status), status_code))
      }
    });

    let readiness = warp::path("ready").and(warp::get()).and_then(move || {
      let prober = readiness_prober.clone();
      async move {
        let status = prober.status.read().await;
        if status.ws_connected {
          Ok::<_, Rejection>(with_status("Ready", StatusCode::OK))
        } else {
          Ok::<_, Rejection>(with_status("Not Ready", StatusCode::SERVICE_UNAVAILABLE))
        }
      }
    });

    warp::serve(health.or(readiness)).run(([0, 0, 0, 0], port)).await;
  }
}
