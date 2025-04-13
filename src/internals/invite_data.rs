use {
  dashmap::DashMap,
  poise::serenity_prelude::{
    User,
    small_fixed_array::FixedString
  },
  std::sync::Arc
};

#[derive(Clone)]
pub struct InviteData {
  pub uses:    u64,
  pub code:    FixedString,
  pub creator: User,
  pub channel: FixedString
}

pub struct InviteCache(Arc<DashMap<FixedString, InviteData>>);

impl InviteCache {
  pub fn new() -> Self { Self(Arc::new(DashMap::new())) }

  pub fn insert(
    &self,
    code: FixedString,
    data: InviteData
  ) {
    self.0.insert(code, data);
  }

  pub fn get(
    &self,
    code: &str
  ) -> Option<InviteData> {
    self.0.get(code).map(|data| data.value().clone())
  }

  pub fn get_all(&self) -> Vec<InviteData> { self.0.iter().map(|data| data.value().clone()).collect() }

  pub fn remove(
    &self,
    code: &str
  ) -> Option<InviteData> {
    match self.0.remove(code) {
      Some(data) => Some(data.1),
      None => None
    }
  }
}
