use async_trait::async_trait;
use hbb_common::{log, ResultType};
use sqlx::{
    sqlite::SqliteConnectOptions, ConnectOptions, Connection, Error as SqlxError, SqliteConnection,
};
use sqlx::Row;
use std::{ops::DerefMut, str::FromStr};
//use sqlx::postgres::PgPoolOptions;
//use sqlx::mysql::MySqlPoolOptions;

type Pool = deadpool::managed::Pool<DbPool>;

pub struct DbPool {
    url: String,
}

#[async_trait]
impl deadpool::managed::Manager for DbPool {
    type Type = SqliteConnection;
    type Error = SqlxError;
    async fn create(&self) -> Result<SqliteConnection, SqlxError> {
        let mut opt = SqliteConnectOptions::from_str(&self.url).unwrap();
        opt.log_statements(log::LevelFilter::Debug);
        SqliteConnection::connect_with(&opt).await
    }
    async fn recycle(
        &self,
        obj: &mut SqliteConnection,
    ) -> deadpool::managed::RecycleResult<SqlxError> {
        Ok(obj.ping().await?)
    }
}

#[derive(Clone)]
pub struct Database {
    pool: Pool,
}

#[derive(Default)]
pub struct Peer {
    pub guid: Vec<u8>,
    pub id: String,
    pub uuid: Vec<u8>,
    pub pk: Vec<u8>,
    pub user: Option<Vec<u8>>,
    pub info: String,
    pub status: Option<i64>,
}

impl Database {
    pub async fn new(url: &str) -> ResultType<Database> {
        if !std::path::Path::new(url).exists() {
            std::fs::File::create(url).ok();
        }
        let n: usize = std::env::var("MAX_DATABASE_CONNECTIONS")
            .unwrap_or_else(|_| "1".to_owned())
            .parse()
            .unwrap_or(1);
        log::debug!("MAX_DATABASE_CONNECTIONS={}", n);
        let pool = Pool::new(
            DbPool {
                url: url.to_owned(),
            },
            n,
        );
        let _ = pool.get().await?; // test
        let db = Database { pool };
        db.create_tables().await?;
        Ok(db)
    }

    async fn create_tables(&self) -> ResultType<()> {
        sqlx::query(
            "
            create table if not exists peer (
                guid blob primary key not null,
                id varchar(100) not null,
                uuid blob not null,
                pk blob not null,
                created_at datetime not null default(current_timestamp),
                user blob,
                status tinyint,
                note varchar(300),
                info text not null
            ) without rowid;
            create unique index if not exists index_peer_id on peer (id);
            create index if not exists index_peer_user on peer (user);
            create index if not exists index_peer_created_at on peer (created_at);
            create index if not exists index_peer_status on peer (status);

            create table if not exists licence_keys (
                licence_key text primary key not null,
                registered_at integer not null,
                expired_at integer not null,
                active integer not null default 1,
                note text
            );
            "
        )
        .execute(self.pool.get().await?.deref_mut())
        .await?;
        // Create indices separately to avoid prepare-time dependency on table existence
        for stmt in [
            "create index if not exists index_licence_keys_active on licence_keys (active);",
            "create index if not exists index_licence_keys_expired_at on licence_keys (expired_at);",
        ] {
            let _ = sqlx::query(stmt)
                .execute(self.pool.get().await?.deref_mut())
                .await;
        }
        Ok(())
    }

    pub async fn get_peer(&self, id: &str) -> ResultType<Option<Peer>> {
        let row = sqlx::query(
            "select guid, id, uuid, pk, user, status, info from peer where id = ?",
        )
        .bind(id)
        .fetch_optional(self.pool.get().await?.deref_mut())
        .await?;
        Ok(row.map(|r| Peer {
            guid: r.try_get("guid").unwrap_or_default(),
            id: r.try_get::<String, _>("id").unwrap_or_default(),
            uuid: r.try_get("uuid").unwrap_or_default(),
            pk: r.try_get("pk").unwrap_or_default(),
            user: r.try_get("user").ok(),
            info: r.try_get::<String, _>("info").unwrap_or_default(),
            status: r.try_get("status").ok(),
        }))
    }

    pub async fn insert_peer(
        &self,
        id: &str,
        uuid: &[u8],
        pk: &[u8],
        info: &str,
    ) -> ResultType<Vec<u8>> {
        let guid = uuid::Uuid::new_v4().as_bytes().to_vec();
        sqlx::query("insert into peer(guid, id, uuid, pk, info) values(?, ?, ?, ?, ?)")
        .bind(&guid)
        .bind(id)
        .bind(uuid)
        .bind(pk)
        .bind(info)
        .execute(self.pool.get().await?.deref_mut())
        .await?;
        Ok(guid)
    }

    pub async fn update_pk(
        &self,
        guid: &Vec<u8>,
        id: &str,
        pk: &[u8],
        info: &str,
    ) -> ResultType<()> {
        sqlx::query("update peer set id=?, pk=?, info=? where guid=?")
        .bind(id)
        .bind(pk)
        .bind(info)
        .bind(guid)
        .execute(self.pool.get().await?.deref_mut())
        .await?;
        Ok(())
    }

    // ------------------------
    // Licence key management
    // ------------------------

    pub async fn is_key_valid(&self, key: &str) -> ResultType<bool> {
        let now = chrono::Utc::now().timestamp();
        let rec = sqlx::query("select expired_at, active from licence_keys where licence_key = ?")
            .bind(key)
            .fetch_optional(self.pool.get().await?.deref_mut())
            .await?;
        if let Some(r) = rec {
            let active: i64 = r.try_get("active").unwrap_or(0);
            let expired_at: i64 = r.try_get("expired_at").unwrap_or(0);
            Ok(active != 0 && expired_at > now)
        } else {
            Ok(false)
        }
    }

    pub async fn insert_key(&self, key: &str, expired_at: i64, active: bool, note: Option<&str>) -> ResultType<()> {
        let now = chrono::Utc::now().timestamp();
        sqlx::query("insert into licence_keys(licence_key, registered_at, expired_at, active, note) values(?, ?, ?, ?, ?)")
        .bind(key)
        .bind(now)
        .bind(expired_at)
        .bind(if active { 1 } else { 0 })
        .bind(note)
        .execute(self.pool.get().await?.deref_mut())
        .await?;
        Ok(())
    }

    pub async fn extend_key_by(&self, key: &str, seconds: i64) -> ResultType<()> {
        // if not exists, no-op
        sqlx::query("update licence_keys set expired_at = expired_at + ? where licence_key = ?")
        .bind(seconds)
        .bind(key)
        .execute(self.pool.get().await?.deref_mut())
        .await?;
        Ok(())
    }

    pub async fn set_key_active(&self, key: &str, active: bool) -> ResultType<()> {
        sqlx::query("update licence_keys set active = ? where licence_key = ?")
        .bind(if active { 1 } else { 0 })
        .bind(key)
        .execute(self.pool.get().await?.deref_mut())
        .await?;
        Ok(())
    }

    pub async fn list_keys(&self, offset: i64, limit: i64) -> ResultType<Vec<LicenceKey>> {
        let rows = sqlx::query(
            "select licence_key, registered_at, expired_at, active, note from licence_keys order by registered_at desc limit ? offset ?",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.get().await?.deref_mut())
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| {
                let licence_key: String = r.try_get("licence_key").unwrap_or_default();
                let registered_at: i64 = r.try_get("registered_at").unwrap_or_default();
                let expired_at: i64 = r.try_get("expired_at").unwrap_or_default();
                let active: i64 = r.try_get("active").unwrap_or_default();
                let note: Option<String> = r.try_get("note").ok();
                LicenceKey { licence_key, registered_at, expired_at, active, note }
            })
            .collect())
    }

    pub async fn count_keys(&self) -> ResultType<i64> {
        let r = sqlx::query("select count(1) as cnt from licence_keys")
            .fetch_one(self.pool.get().await?.deref_mut())
            .await?;
        let cnt: i64 = r.try_get("cnt").unwrap_or(0);
        Ok(cnt)
    }

    pub async fn key_exists(&self, key: &str) -> ResultType<bool> {
        let r = sqlx::query("select 1 as exists_flag from licence_keys where licence_key = ? limit 1")
            .bind(key)
            .fetch_optional(self.pool.get().await?.deref_mut())
            .await?;
        Ok(r.is_some())
    }
}

use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
pub struct LicenceKey {
    pub licence_key: String,
    pub registered_at: i64,
    pub expired_at: i64,
    pub active: i64,
    pub note: Option<String>,
}

#[cfg(test)]
mod tests {
    use hbb_common::tokio;
    #[test]
    fn test_insert() {
        insert();
    }

    #[tokio::main(flavor = "multi_thread")]
    async fn insert() {
        let db = super::Database::new("test.sqlite3").await.unwrap();
        let mut jobs = vec![];
        for i in 0..10000 {
            let cloned = db.clone();
            let id = i.to_string();
            let a = tokio::spawn(async move {
                let empty_vec = Vec::new();
                cloned
                    .insert_peer(&id, &empty_vec, &empty_vec, "")
                    .await
                    .unwrap();
            });
            jobs.push(a);
        }
        for i in 0..10000 {
            let cloned = db.clone();
            let id = i.to_string();
            let a = tokio::spawn(async move {
                cloned.get_peer(&id).await.unwrap();
            });
            jobs.push(a);
        }
        hbb_common::futures::future::join_all(jobs).await;
    }
}
