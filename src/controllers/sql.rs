mod mpservers;
pub use mpservers::MpServers;

mod prohibited_words;
pub use prohibited_words::ProhibitedWords;

mod sanctions;
pub use sanctions::Sanctions;

mod settings;
pub use settings::Settings;

mod webhooks;
pub use webhooks::Webhooks;

mod prohibited_urls;
pub use prohibited_urls::ProhibitedUrls;

static DAG_SQL: &str = "DagSql";
static QUERY_FAILED: &str = "Failed to query the database";
