use std::{path::Path, str::FromStr};

use chrono::prelude::{DateTime, FixedOffset};
use sqlx::{
    SqlitePool, SqliteTransaction,
    migrate::{MigrateError, Migrator},
    prelude::Row,
    sqlite::{SqliteConnectOptions, SqliteRow},
};

use crate::{dbtypes::RssMsg, sqliteacquire::SqliteAcquire};

pub async fn get_migrator() -> Result<Migrator, MigrateError> {
    Migrator::new(Path::new("./migrations")).await
}

/// Open the appropriate sqlite db and ensure it is ready to use by applying migrations.
pub async fn open_db(author: &str, world: &str, migrator: &Migrator) -> anyhow::Result<SqlitePool> {
    let url = format!("sqlite://dbs/{author}-{world}.sqlite");
    let options = SqliteConnectOptions::from_str(&url)?.create_if_missing(true);
    let pool = SqlitePool::connect_with(options).await?;
    migrator.run(&pool).await?;
    Ok(pool)
}

/// Insert the new messages or update existing ones, based on the guid
pub async fn upsert_msgs<'a, A: SqliteAcquire<'a>>(msgs: Vec<RssMsg>, conn: A) -> sqlx::Result<()> {
    let mut conn = conn.acquire().await?;
    for msg in msgs {
        sqlx::query(
            "INSERT INTO rss_msg(
            title, description, link, guid, pub_date, sent
        ) VALUES (
         $1, $2, $3, $4, $5, 0
        ) ON CONFLICT DO UPDATE SET
         title=$1,
         description=$2,
         link=$3,
         pub_date=$5
         ",
        )
        .bind(&msg.title)
        .bind(&msg.description)
        .bind(&msg.link)
        .bind(&msg.guid)
        .bind(&msg.serialize_date())
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

pub async fn get_last_run_date<'a, A: SqliteAcquire<'a>>(
    conn: A,
) -> sqlx::Result<DateTime<FixedOffset>> {
    let mut conn = conn.acquire().await?;
    let res: Option<SqliteRow> = sqlx::query("SELECT time FROM rss_scan LIMIT 1")
        .fetch_optional(&mut *conn)
        .await?;
    Ok(res
        .map(|row| DateTime::parse_from_rfc3339(&row.get::<String, _>("time")).ok())
        .flatten()
        .unwrap_or_default())
}

/// Update the last run date to the current time.
pub async fn update_run_date<'a, A: SqliteAcquire<'a>>(conn: A) -> sqlx::Result<()> {
    let mut conn = conn.acquire().await?;
    sqlx::query("UPDATE rss_scan SET time=$1;")
        .bind(&chrono::offset::Utc::now().to_rfc3339())
        .execute(&mut *conn)
        .await?;
    Ok(())
}
