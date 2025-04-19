use {
  std::env::var,
  tonic::{
    Request,
    Response,
    Status,
    transport::Channel
  }
};

pub mod monica {
  tonic::include_proto!("monica");
}

pub use monica::{
  FetchRequest,
  FetchResponse,
  monica_service_client::MonicaServiceClient
};

#[derive(Debug, Clone)]
pub struct MonicaGRPCClient {
  inner: MonicaServiceClient<Channel>
}

impl Default for MonicaGRPCClient {
  fn default() -> Self { Self::new() }
}

impl MonicaGRPCClient {
  pub fn new() -> Self {
    let uri = var("MONICA_GRPC_URI").unwrap_or_else(|_| "127.0.0.1:37090".to_owned());
    let channel = Channel::builder(format!("http://{uri}").parse().unwrap()).connect_lazy();

    Self {
      inner: MonicaServiceClient::new(channel)
    }
  }

  pub async fn fetch_data(
    &mut self,
    request: impl Into<FetchRequest>
  ) -> Result<Response<FetchResponse>, Status> {
    let request = Request::new(request.into());
    self.inner.fetch_data(request).await
  }
}
