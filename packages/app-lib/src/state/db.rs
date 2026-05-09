use crate::ErrorKind;
use crate::state::DirectoryInfo;
use sqlx::sqlite::{
    SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions,
};
use sqlx::{Pool, Sqlite};
use std::str::FromStr;
use std::time::Duration;
use tokio::time::Instant;

pub(crate) async fn connect() -> crate::Result<Pool<Sqlite>> {
    let pool = connect_without_migrate().await?;

    // [AR] Fix: Patch corrupted migration checksums before applying migrations.
    // sqlx::migrate!() embeds checksums at compile time. If the .sql files on disk
    // have different line endings than at compile time (LF vs CRLF), the embedded
    // checksums won't match the database. We patch the DB to match the compiled checksums.
    let _ = apply_migration_fix(&pool).await;

    let migrator = sqlx::migrate!();

    if let Err(err) = migrator.run(&pool).await {
        tracing::warn!(
            "Migration failed (likely due to line ending mismatch): {err}. Resetting migration state and retrying..."
        );

        // Clear migration tracking to force re-run
        sqlx::query("DELETE FROM _sqlx_migrations")
            .execute(&pool)
            .await?;

        // Try again with fresh tracking
        if let Err(err2) = migrator.run(&pool).await {
            tracing::error!("Migration failed after reset: {err2}");
            return Err(crate::ErrorKind::OtherError(format!(
                "Failed to apply database migrations: {err2}"
            )).into());
        }
    }

    if let Err(err) = stale_data_cleanup(&pool).await {
        tracing::warn!(
            "Failed to clean up stale data from state database: {err}"
        );
    }

    Ok(pool)
}

// [AR] Feature. Implement SQLite3 connection without SQLx migrations.
async fn connect_without_migrate() -> crate::Result<Pool<Sqlite>> {
    let settings_dir = DirectoryInfo::get_initial_settings_dir().ok_or(
        ErrorKind::FSError("Could not find valid config dir".to_string()),
    )?;

    if !settings_dir.exists() {
        crate::util::io::create_dir_all(&settings_dir).await?;
    }

    let db_path = settings_dir.join("app.db");

    let uri = format!("sqlite:{}", db_path.display());
    let conn_options = SqliteConnectOptions::from_str(&uri)?
        .busy_timeout(Duration::from_secs(30))
        .journal_mode(SqliteJournalMode::Wal)
        .optimize_on_close(true, None)
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(100)
        .connect_with(conn_options)
        .await?;

    Ok(pool)
}

/// Cleans up data from the database that is no longer referenced, but must be
/// kept around for a little while to allow users to recover from accidental
/// deletions.
async fn stale_data_cleanup(pool: &Pool<Sqlite>) -> crate::Result<()> {
    let mut tx = pool.begin().await?;

    sqlx::query!(
        "DELETE FROM default_minecraft_capes WHERE minecraft_user_uuid NOT IN (SELECT uuid FROM minecraft_users)"
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "DELETE FROM custom_minecraft_skins WHERE minecraft_user_uuid NOT IN (SELECT uuid FROM minecraft_users)"
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}
/// Patches _sqlx_migrations table with checksums from the compiled migrator.
/// This fixes line-ending mismatch issues where sqlx::migrate!() checksums
/// (embedded at compile time) differ from what's in the database.
pub(crate) async fn apply_migration_fix(pool: &Pool<Sqlite>) -> crate::Result<bool> {
    let started = Instant::now();

    tracing::info!(
        "⚙️  Patching migration checksums from compiled migrator..."
    );

    let migrator = sqlx::migrate!();
    let mut changed = false;

    for migration in migrator.iter() {
        let version = migration.version;
        let checksum = migration.checksum.iter().map(|b| format!("{:02x}", b)).collect::<String>();

        let result = sqlx::query(
            r#"
            UPDATE _sqlx_migrations
            SET checksum = ?2
            WHERE version = ?1;
            "#
        )
        .bind(version as i64)
        .bind(migration.checksum.to_vec())
        .execute(pool)
        .await?;

        if result.rows_affected() > 0 {
            tracing::info!(
                "  ✓ Patched migration {version} -> {checksum}"
            );
            changed = true;
        }
    }

    tracing::info!(
        "✅ Checksum patching completed in {:.2?} (changes: {})",
        started.elapsed(),
        changed
    );

    Ok(changed)
}
