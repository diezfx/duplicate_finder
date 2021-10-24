use crate::models::PathInfo;
use sqlx::sqlite::SqlitePool;

pub struct PathRepository {
    connection: sqlx::SqlitePool,
}

impl PathRepository {
    pub async fn new(db_path: &str) -> anyhow::Result<Self> {
        let connection_string = format!("{}", db_path);
        let conn = SqlitePool::connect(&connection_string).await;
        let res = conn.unwrap();

        Ok(PathRepository { connection: res })
    }

    pub async fn store(&self, new_path: PathInfo) -> anyhow::Result<i64> {
        let insert_stmt = "INSERT INTO path_description (path,hash,last_modified)VALUES( ?,?,?);";

        let result = sqlx::query(insert_stmt)
            .bind(new_path.path)
            .bind(new_path.hash)
            .bind(new_path.last_modified)
            .execute(&self.connection)
            .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn update(&self, new_path: PathInfo) -> anyhow::Result<i64> {
        let update_stmt = "UPDATE path_description SET path=?,hash=?,last_modified=? WHERE path=?";
        let path_copy = new_path.path.clone();
        let result = sqlx::query(update_stmt)
            .bind(new_path.path)
            .bind(new_path.hash)
            .bind(new_path.last_modified)
            .bind(path_copy)
            .execute(&self.connection)
            .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn find_by_path(&self, path: &str) -> anyhow::Result<Option<PathInfo>> {
        let result = sqlx::query_as::<_, PathInfo>("SELECT * FROM path_description WHERE path = ?")
            .bind(path)
            .fetch_optional(&self.connection)
            .await?;

        Ok(result)
    }
}
