use std::{path::Path, str::FromStr};

use chrono::prelude::{DateTime, FixedOffset};
use sqlx::{
    SqliteConnection, SqlitePool,
    migrate::{MigrateError, Migrator},
    prelude::Row,
    sqlite::{SqliteConnectOptions, SqliteRow},
};

use crate::dbtypes::{RssMsg, TargetChannel};

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
pub async fn upsert_msgs(msgs: Vec<RssMsg>, conn: &mut SqliteConnection) -> sqlx::Result<()> {
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
        .bind(msg.serialize_date())
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

pub async fn get_last_run_date(conn: &mut SqliteConnection) -> sqlx::Result<DateTime<FixedOffset>> {
    let res: Option<SqliteRow> = sqlx::query("SELECT time FROM rss_scan LIMIT 1")
        .fetch_optional(&mut *conn)
        .await?;
    Ok(res
        .and_then(|row| DateTime::parse_from_rfc3339(&row.get::<String, _>("time")).ok())
        .unwrap_or_default())
}

/// Update the last run date to the current time.
pub async fn update_run_date(conn: &mut SqliteConnection) -> sqlx::Result<()> {
    sqlx::query("UPDATE rss_scan SET time=$1;")
        .bind(chrono::offset::Utc::now().to_rfc3339())
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// Get an unsent message
pub async fn get_unsent_message(conn: &mut SqliteConnection) -> sqlx::Result<Option<RssMsg>> {
    sqlx::query_as(
        "
    SELECT title, description, link, guid, pub_date, image_url
    FROM rss_msg
    WHERE sent=0
    ORDER BY pub_date ASC
    LIMIT 1
",
    )
    .bind(chrono::offset::Utc::now().to_rfc3339())
    .fetch_optional(&mut *conn)
    .await
}

pub async fn set_message_sent(guid: &str, conn: &mut SqliteConnection) -> sqlx::Result<()> {
    sqlx::query(
        "
    UPDATE rss_msg
    SET sent=1
    WHERE guid=$1
        ",
    )
    .bind(guid)
    .execute(conn)
    .await?;
    Ok(())
}

/// Get all guilds and channels
pub async fn get_discord_channels(conn: &mut SqliteConnection) -> sqlx::Result<Vec<TargetChannel>> {
    sqlx::query_as(
        "
    SELECT guild_id, channel_id
    FROM discord_channels;
",
    )
    .fetch_all(&mut *conn)
    .await
}

/// Add a new guild and channel to send to.
pub async fn add_discord_channel(
    target: &TargetChannel,
    conn: &mut SqliteConnection,
) -> sqlx::Result<()> {
    sqlx::query("\
    INSERT INTO discord_channels (guild_id, channel_id)
    VALUES (?, ?);
"
    )
    .bind(&target.guild_id.to_string())
    .bind(&target.channel_id.to_string())
    .execute(&mut *conn)
    .await?;
    Ok(())
}