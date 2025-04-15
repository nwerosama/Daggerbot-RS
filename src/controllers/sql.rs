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

use {
  regex::Regex,
  std::{
    fs,
    process
  }
};

static DAG_SQL: &str = "DagSql";
static QUERY_FAILED: &str = "Failed to query the database";

const SCHEMA_DIR: &str = "schemas";

/// Recursively execute all SQL statements in the `schemas` directory<br>
/// ### Errors
/// The function will return an error if:
/// - Directory does not exist in root-level
/// - Improper SQL syntaxes
///   - Guides you to which file it failed on and the error returned from database
pub async fn execute_schemas(pool: &sqlx::PgPool) -> Result<String, crate::BotError> {
  let paths = match fs::read_dir(SCHEMA_DIR) {
    Ok(p) => p,
    Err(e) => {
      eprintln!("{DAG_SQL}[Database:Schemas:Error] {e}");
      process::exit(1);
    }
  };

  let mut executed_schemas = Vec::new();

  for path in paths {
    let path = path?;
    let path = path.path();
    let fmt_path = path.display().to_string().replace(&format!("{SCHEMA_DIR}/"), "");

    if path.is_file() && path.extension().is_some_and(|ext| ext == "sql") {
      let mut in_dollar_block = false;
      let mut query_buff = String::new();
      let sql = {
        let file = fs::read_to_string(&path)?;
        let (s, m) = remove_sql_comments();
        let cleaned_sql = s.replace_all(&file, "").to_string();
        m.replace_all(&cleaned_sql, "").to_string()
      };

      for line in sql.lines() {
        // Detect dollar-quoted strings
        if line.contains("$$") {
          in_dollar_block = !in_dollar_block;
        }

        query_buff.push_str(line);
        query_buff.push(' '); // Keep spaces between lines

        // Only split queries on semicolons if not inside a dollar-quoted block
        if !in_dollar_block && line.trim().ends_with(";") {
          let query = query_buff.trim();
          if !query.is_empty() {
            if let Err(e) = sqlx::query(query).execute(pool).await {
              eprintln!("{DAG_SQL}[Database:Schemas:Error] Failed to execute {fmt_path}\n{e}");
              process::exit(1);
            }
          }
          query_buff.clear(); // Clean the buffer for the next query to be read
        }
      }

      if !query_buff.trim().is_empty() {
        if let Err(e) = sqlx::query(query_buff.trim()).execute(pool).await {
          eprintln!("{DAG_SQL}[Database:Schemas:Error] Failed to execute {fmt_path}\n{e}");
          process::exit(1);
        }
      }

      executed_schemas.push(fmt_path.clone());
    }
  }

  let mut execution_success = String::new();
  if !executed_schemas.is_empty() {
    println!("{DAG_SQL}[Database:Schemas:Info] Successfully executed: {}", executed_schemas.join(", "));

    let linted = executed_schemas.iter().map(|s| format!("`{s}`")).collect::<Vec<String>>().join(", ");
    execution_success.push_str(&format!("Successfully executed: {linted}"));
  }

  Ok(execution_success)
}

fn remove_sql_comments() -> (Regex, Regex) {
  let single_line = Regex::new(r"--.*").unwrap();
  let multi_line = Regex::new(r"/\*[\s\S]*?\*/").unwrap();

  (single_line, multi_line)
}
