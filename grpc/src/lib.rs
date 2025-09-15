use {
  crate::monica_service_client::MonicaServiceClient,
  std::{
    env::var,
    ops::{
      Deref,
      DerefMut
    }
  },
  tonic::transport::Channel
};

tonic::include_proto!("monica");

#[derive(Debug, Clone)]
pub struct MonicaClient {
  inner: MonicaServiceClient<Channel>
}

impl Deref for MonicaClient {
  type Target = MonicaServiceClient<Channel>;

  fn deref(&self) -> &Self::Target { &self.inner }
}

impl DerefMut for MonicaClient {
  fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}

impl MonicaClient {
  pub async fn new() -> Self {
    let uri = var("MONICA_GRPC_URI").unwrap_or_else(|_| "127.0.0.1:37090".to_owned());
    let channel = Channel::builder(format!("http://{uri}").parse().unwrap()).connect_lazy();

    Self {
      inner: MonicaServiceClient::new(channel)
    }
  }
}
