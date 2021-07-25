#[derive(sqlx::FromRow)]
pub struct PathInfo {
    pub id: i64,
    pub path: String,
    pub hash: String,
}
