use {
  cargo_toml::Manifest,
  std::sync::LazyLock
};

#[cfg(feature = "production")]
pub static GIT_COMMIT_HASH: &str = env!("GIT_COMMIT_HASH");
pub static GIT_COMMIT_BRANCH: &str = env!("GIT_COMMIT_BRANCH");

#[cfg(not(feature = "production"))]
pub static GIT_COMMIT_HASH: &str = "devel";

pub static BOT_VERSION: LazyLock<String> = LazyLock::new(|| {
  let cargo_version = Manifest::from_str(include_str!("../Cargo.toml"))
    .unwrap()
    .package
    .unwrap()
    .version
    .unwrap();
  format!("v{cargo_version}")
});
