use std::str::FromStr;

use async_trait::async_trait;
use atuin_common::db;
use atuin_common::db::SqliteDbUrl;
use secrecy::SecretString;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

use super::{Database, DbError, DbResult, OrdinalBindingDialect};

#[derive(Clone)]
pub struct Sqlite {
    pool: sqlx::Pool<sqlx::sqlite::Sqlite>,
}

#[async_trait]
impl Database for Sqlite {
    type Db = sqlx::sqlite::Sqlite;
    type Url = SqliteDbUrl;
    type Dialect = OrdinalBindingDialect;

    fn pool(&self) -> &sqlx::Pool<Self::Db> {
        &self.pool
    }

    async fn connect(url: SqliteDbUrl, password: Option<SecretString>) -> DbResult<Self> {
        if password.is_some() {
            return Err(DbError::Other(eyre::eyre!("SQLite databases take no password")));
        }

        let opts = SqliteConnectOptions::from_str(url.as_str())?
            .journal_mode(SqliteJournalMode::Wal)
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new().connect_with(opts).await?;

        db::migrate!(&pool, "src/db/sqlite/migrations")
            .await
            .map_err(|error| DbError::Other(error.into()))?;

        Ok(Self { pool })
    }
}

#[cfg(test)]
mod tests {
    use atuin_common::db::SqliteDbUrl;
    use rstest::rstest;

    use super::{Database, Sqlite};

    #[rstest]
    #[tokio::test]
    async fn password_is_rejected_not_ignored() {
        let url = SqliteDbUrl("sqlite::memory:".to_owned());
        assert!(Sqlite::connect(url, Some("hunter2".into())).await.is_err());
    }
}
