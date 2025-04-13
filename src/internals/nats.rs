use {
  async_nats::{
    Client,
    ConnectOptions,
    Error,
    Message
  },
  serde::{
    Deserialize,
    Serialize
  },
  serde_json::Value,
  std::env::var
};

/// Payload received from client
#[derive(Debug, Serialize, Deserialize)]
pub struct MonicaNatsPayload {
  pub identifier: String, // identifier is a service loop name, e.g autorefresh, mp_cmd, etc.
  pub data:       Value
}

/// Monica responded to client's request
#[derive(Debug, Serialize, Deserialize)]
pub struct MonicaNatsResponse {
  pub identifier: String,
  pub data:       Value
}

/// Client setup to communicate with NATS server
#[derive(Debug, Clone)]
pub struct MonicaNatsClient {
  pub client: Client
}

impl MonicaNatsClient {
  /// Initialize the NATS client<br>
  /// **This requires access to `MONICA_NATS_URI` envvar to establish a connection!**
  pub async fn new() -> Result<Self, Error> {
    let uri = var("MONICA_NATS_URI").expect("NATS Connection required! (e.g. MONICA_NATS_URI=nats://127.0.0.1:4222)");
    let client = async_nats::connect_with_options(
      uri.clone(),
      ConnectOptions::new()
        .connection_timeout(tokio::time::Duration::from_secs(15))
        .name("daggerbot")
    )
    .await?;

    println!("NATS({uri})[Info] Connection successfully established");

    Ok(Self { client })
  }

  /// Publish a response to client
  pub async fn publish(
    &self,
    payload: MonicaNatsPayload
  ) -> Result<MonicaNatsResponse, Error> {
    let serialized = serde_json::to_string(&payload)?;
    let message: Message = self.client.request("monica.pubsub", serialized.into()).await?;

    let str_data = String::from_utf8_lossy(&message.payload);
    let inner_json: String = serde_json::from_str(&str_data)?;
    let response: MonicaNatsResponse = serde_json::from_str(&inner_json)?;

    Ok(response)
  }
}
