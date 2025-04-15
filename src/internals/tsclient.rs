use tokenservice_client::{
  TokenService,
  TokenServiceApi
};

pub struct TSClient(TokenService);

impl TSClient {
  pub fn new() -> Self {
    let args: Vec<String> = std::env::args().collect();
    let service = if args.len() > 1 { args[1].as_str() } else { "daggerbot" };
    Self(TokenService::new(service))
  }

  pub async fn get(&self) -> Result<TokenServiceApi, crate::BotError> {
    match self.0.connect().await {
      Ok(api) => Ok(api),
      Err(e) => Err(e)
    }
  }
}
