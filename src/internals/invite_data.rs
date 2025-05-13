use {
  dashmap::{
    DashMap,
    mapref::one::Ref
  },
  poise::serenity_prelude::small_fixed_array::FixedString,
  std::sync::Arc
};

pub struct InviteData {
  pub uses:    u64,
  pub creator: FixedString<u8>,
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
  ) -> Option<Ref<'_, FixedString<u32>, InviteData>> {
    self.0.get(code)
  }

  pub fn remove(
    &self,
    code: &str
  ) -> Option<InviteData> {
    match self.0.remove(code) {
      Some(data) => Some(data.1),
      None => None
    }
  }

  pub fn compare_uses(
    &self,
    code: &str,
    uses: u64
  ) -> bool {
    self.get(code).is_some_and(|i| i.uses < uses)
  }
}
