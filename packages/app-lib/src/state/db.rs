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
    let _ = repair_completed_initial_migration(&pool).await;

    run_migrations_with_repair(&pool).await?;

    if let Err(err) = stale_data_cleanup(&pool).await {
        tracing::warn!(
            "Failed to clean up stale data from state database: {err}"
        );
    }

    Ok(pool)
}

const INITIAL_MIGRATION_VERSION: i64 = 20240711194701;
const INITIAL_SCHEMA_OBJECTS: &[(&str, &str)] = &[
    ("table", "settings"),
    ("table", "java_versions"),
    ("table", "minecraft_users"),
    ("index", "minecraft_users_active"),
    ("table", "minecraft_device_tokens"),
    ("table", "modrinth_users"),
    ("index", "modrinth_users_active"),
    ("table", "cache"),
    ("table", "profiles"),
    ("table", "processes"),
    ("index", "processes_profile_path"),
];

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
pub(crate) async fn apply_migration_fix(
    pool: &Pool<Sqlite>,
) -> crate::Result<bool> {
    let started = Instant::now();

    tracing::info!(
        "⚙️  Patching migration checksums from compiled migrator..."
    );

    let migrator = sqlx::migrate!();
    let mut changed = false;

    for migration in migrator.iter() {
        let version = migration.version;
        let checksum = migration
            .checksum
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

        let result = sqlx::query(
            r#"
            UPDATE _sqlx_migrations
            SET checksum = ?2
            WHERE version = ?1;
            "#,
        )
        .bind(version as i64)
        .bind(migration.checksum.to_vec())
        .execute(pool)
        .await?;

        if result.rows_affected() > 0 {
            tracing::info!("  ✓ Patched migration {version} -> {checksum}");
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

/// Repairs a database whose schema exists but whose SQLx migration tracking was
/// lost or has stale checksums.
pub async fn repair_migration_state_from_disk() -> crate::Result<bool> {
    let pool = connect().await?;
    Ok(!pool.is_closed())
}

async fn repair_completed_initial_migration(
    pool: &Pool<Sqlite>,
) -> crate::Result<bool> {
    if !initial_schema_exists(pool).await? {
        return Ok(false);
    }

    ensure_sqlx_migrations_table(pool).await?;

    let exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM _sqlx_migrations WHERE version = ?1",
    )
    .bind(INITIAL_MIGRATION_VERSION)
    .fetch_one(pool)
    .await?
        > 0;

    if exists {
        return Ok(false);
    }

    let migrator = sqlx::migrate!();
    let Some(migration) = migrator
        .iter()
        .find(|migration| migration.version == INITIAL_MIGRATION_VERSION)
    else {
        return Ok(false);
    };

    insert_migration_state(pool, migration).await?;
    tracing::warn!(
        "Recovered missing SQLx state for completed initial migration {} ({})",
        migration.version,
        migration.description
    );

    Ok(true)
}

async fn run_migrations_with_repair(pool: &Pool<Sqlite>) -> crate::Result<()> {
    let migrator = sqlx::migrate!();
    let max_attempts = migrator.iter().count() + 1;

    for attempt in 1..=max_attempts {
        match migrator.run(pool).await {
            Ok(()) => return Ok(()),
            Err(err) => {
                let message = err.to_string();
                tracing::warn!(
                    "Migration attempt {attempt}/{max_attempts} failed: {message}. Attempting migration state repair..."
                );

                if !repair_migration_state(pool, &message).await? {
                    tracing::error!(
                        "Migration failed and repair was not applicable: {message}"
                    );
                    return Err(crate::ErrorKind::OtherError(format!(
                        "Failed to apply database migrations: {message}"
                    ))
                    .into());
                }
            }
        }
    }

    Err(crate::ErrorKind::OtherError(
        "Failed to apply database migrations after repair attempts".to_string(),
    )
    .into())
}

async fn repair_migration_state(
    pool: &Pool<Sqlite>,
    migration_error: &str,
) -> crate::Result<bool> {
    let schema_exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'settings'",
    )
    .fetch_one(pool)
    .await?
        > 0;

    if !schema_exists {
        tracing::warn!(
            "Migration repair skipped: settings table does not exist yet"
        );
        return Ok(false);
    }

    let tracking_exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = '_sqlx_migrations'",
    )
    .fetch_one(pool)
    .await?
        > 0;

    if !tracking_exists {
        tracing::warn!(
            "Migration repair skipped: _sqlx_migrations table does not exist yet"
        );
        return Ok(false);
    }

    let migrator = sqlx::migrate!();
    let mut changed = false;

    for migration in migrator.iter() {
        if !should_mark_migration_applied(migration.version, migration_error) {
            continue;
        }
        if migration.version == INITIAL_MIGRATION_VERSION
            && !initial_schema_exists(pool).await?
        {
            tracing::warn!(
                "Migration repair skipped for initial migration: base schema is incomplete"
            );
            continue;
        }

        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM _sqlx_migrations WHERE version = ?1",
        )
        .bind(migration.version)
        .fetch_one(pool)
        .await?
            > 0;

        if exists {
            continue;
        }

        insert_migration_state(pool, migration).await?;

        tracing::warn!(
            "Marked migration {} ({}) as applied during repair",
            migration.version,
            migration.description
        );
        changed = true;
    }

    Ok(changed)
}

fn should_mark_migration_applied(version: i64, migration_error: &str) -> bool {
    if migration_error.is_empty() {
        return true;
    }

    migration_error.contains(&format!("migration {version}"))
        && (migration_error.contains("already exists")
            || migration_error.contains("duplicate column name")
            || migration_error.contains("UNIQUE constraint failed"))
}

async fn initial_schema_exists(pool: &Pool<Sqlite>) -> crate::Result<bool> {
    for (type_, name) in INITIAL_SCHEMA_OBJECTS {
        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = ?1 AND name = ?2",
        )
        .bind(type_)
        .bind(name)
        .fetch_one(pool)
        .await?
            > 0;

        if !exists {
            return Ok(false);
        }
    }

    Ok(true)
}

async fn ensure_sqlx_migrations_table(
    pool: &Pool<Sqlite>,
) -> crate::Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS _sqlx_migrations (
            version BIGINT PRIMARY KEY,
            description TEXT NOT NULL,
            installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            success BOOLEAN NOT NULL,
            checksum BLOB NOT NULL,
            execution_time BIGINT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn insert_migration_state(
    pool: &Pool<Sqlite>,
    migration: &sqlx::migrate::Migration,
) -> crate::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO _sqlx_migrations
            (version, description, success, checksum, execution_time)
        VALUES (?1, ?2, TRUE, ?3, 0)
        "#,
    )
    .bind(migration.version)
    .bind(migration.description.to_string())
    .bind(migration.checksum.to_vec())
    .execute(pool)
    .await?;

    Ok(())
}
