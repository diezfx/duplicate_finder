use chrono::{DateTime, Utc};

#[derive(sqlx::FromRow)]
pub struct PathInfo {
    pub path: String,
    pub hash: String,
    pub last_modified: DateTime<Utc>,
}
