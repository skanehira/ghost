use crate::app::error::Result;
use rusqlite::Connection;

/// Initialize the database and create tables if they don't exist
pub fn init_database() -> Result<Connection> {
    init_database_with_config(None)
}

/// Initialize the database with a specific config
pub fn init_database_with_config(config: Option<crate::app::config::Config>) -> Result<Connection> {
    let config = config.unwrap_or_default();
    config.ensure_directories()?;

    let db_path = config.get_db_path();
    let conn = Connection::open(db_path)?;

    // Enable WAL mode for better concurrency
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;

    // Create tasks table
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            pid INTEGER NOT NULL,
            pgid INTEGER,
            command TEXT NOT NULL,
            env TEXT,
            cwd TEXT,
            status TEXT NOT NULL DEFAULT 'running',
            exit_code INTEGER,
            started_at INTEGER NOT NULL,
            finished_at INTEGER,
            log_path TEXT NOT NULL
        )
        "#,
        [],
    )?;

    // Create indexes for performance
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status)",
        [],
    )?;

    conn.execute("CREATE INDEX IF NOT EXISTS idx_tasks_pid ON tasks(pid)", [])?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tasks_started_at ON tasks(started_at)",
        [],
    )?;

    Ok(conn)
}
