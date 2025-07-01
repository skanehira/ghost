use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, Result as SqliteResult, Row};
use serde::{Deserialize, Serialize};

use crate::core::command::{CommandError, process_exists};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub pid: u32,
    pub pgid: Option<i32>,
    pub command: String,     // JSON serialized Vec<String>
    pub env: Option<String>, // JSON serialized environment variables
    pub cwd: Option<String>,
    pub status: String, // running, exited, killed, unknown
    pub exit_code: Option<i32>,
    pub started_at: i64, // Unix timestamp
    pub finished_at: Option<i64>,
    pub log_path: String,
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Command error: {0}")]
    Command(#[from] CommandError),
}

pub type Result<T> = std::result::Result<T, StorageError>;

/// Get the default database directory path
fn get_db_dir() -> PathBuf {
    let base_dir = if cfg!(windows) {
        dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."))
    } else {
        dirs::data_dir().unwrap_or_else(|| PathBuf::from("."))
    };

    base_dir.join("ghost")
}

/// Get the database file path
pub fn get_db_path() -> PathBuf {
    get_db_dir().join("tasks.db")
}

/// Initialize the database and create tables if they don't exist
pub fn init_database() -> Result<Connection> {
    let db_dir = get_db_dir();
    std::fs::create_dir_all(&db_dir)?;

    let db_path = get_db_path();
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

/// Insert a new task into the database
#[allow(clippy::too_many_arguments)]
pub fn insert_task(
    conn: &Connection,
    id: &str,
    pid: u32,
    pgid: Option<i32>,
    command: &[String],
    env: Option<&[(String, String)]>,
    cwd: Option<&Path>,
    log_path: &Path,
) -> Result<()> {
    let command_json = serde_json::to_string(command)?;
    let env_json = env.map(serde_json::to_string).transpose()?;
    let cwd_str = cwd.map(|p| p.to_string_lossy().to_string());
    let started_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    conn.execute(
        r#"
        INSERT INTO tasks (
            id, pid, pgid, command, env, cwd, status, 
            started_at, log_path
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'running', ?7, ?8)
        "#,
        (
            id,
            pid as i64,
            pgid.map(|p| p as i64),
            command_json,
            env_json,
            cwd_str,
            started_at,
            log_path.to_string_lossy(),
        ),
    )?;

    Ok(())
}

/// Get a task by ID
pub fn get_task(conn: &Connection, task_id: &str) -> Result<Task> {
    let mut stmt = conn.prepare(
        "SELECT id, pid, pgid, command, env, cwd, status, exit_code, started_at, finished_at, log_path FROM tasks WHERE id = ?1"
    )?;

    let task = stmt.query_row([task_id], |row| {
        Ok(Task {
            id: row.get(0)?,
            pid: row.get::<_, i64>(1)? as u32,
            pgid: row.get::<_, Option<i64>>(2)?.map(|p| p as i32),
            command: row.get(3)?,
            env: row.get(4)?,
            cwd: row.get(5)?,
            status: row.get(6)?,
            exit_code: row.get::<_, Option<i64>>(7)?.map(|c| c as i32),
            started_at: row.get(8)?,
            finished_at: row.get(9)?,
            log_path: row.get(10)?,
        })
    })?;

    Ok(task)
}

/// Get all tasks, optionally filtered by status
pub fn get_tasks(conn: &Connection, status_filter: Option<&str>) -> Result<Vec<Task>> {
    let sql = match status_filter {
        Some(_) => {
            "SELECT id, pid, pgid, command, env, cwd, status, exit_code, started_at, finished_at, log_path FROM tasks WHERE status = ?1 ORDER BY started_at DESC"
        }
        None => {
            "SELECT id, pid, pgid, command, env, cwd, status, exit_code, started_at, finished_at, log_path FROM tasks ORDER BY started_at DESC"
        }
    };

    let mut stmt = conn.prepare(sql)?;

    let task_iter = if let Some(status) = status_filter {
        stmt.query_map([status], row_to_task)?
    } else {
        stmt.query_map([], row_to_task)?
    };

    let mut tasks = Vec::new();
    for task in task_iter {
        tasks.push(task?);
    }

    Ok(tasks)
}

/// Get all tasks with process existence check and status update
/// This function will check if running processes still exist and update their status to 'exited' if not
pub fn get_tasks_with_process_check(
    conn: &Connection,
    status_filter: Option<&str>,
) -> Result<Vec<Task>> {
    let tasks = get_tasks(conn, status_filter)?;

    // Check and update running tasks using existing function
    let mut updated_tasks = Vec::new();
    for task in tasks {
        if task.status == "running" {
            // Use existing update_task_status_by_process_check for consistency
            let updated_task = update_task_status_by_process_check(conn, &task.id)?;
            updated_tasks.push(updated_task);
        } else {
            updated_tasks.push(task);
        }
    }

    Ok(updated_tasks)
}

/// Update task status
pub fn update_task_status(
    conn: &Connection,
    task_id: &str,
    status: &str,
    exit_code: Option<i32>,
) -> Result<Option<i64>> {
    let finished_at = if status == "exited" || status == "killed" {
        Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        )
    } else {
        None
    };

    conn.execute(
        "UPDATE tasks SET status = ?1, exit_code = ?2, finished_at = ?3 WHERE id = ?4",
        (status, exit_code.map(|c| c as i64), finished_at, task_id),
    )?;

    Ok(finished_at)
}

/// Update task status by checking if process is still running
pub fn update_task_status_by_process_check(conn: &Connection, task_id: &str) -> Result<Task> {
    let mut task = get_task(conn, task_id)?;

    // Only check if task is marked as running
    if task.status == "running" && !process_exists(task.pid) {
        // Process no longer exists
        let finished_at = update_task_status(conn, task_id, "exited", None)?;
        task.status = "exited".to_string();
        task.finished_at = finished_at;
    }

    Ok(task)
}

/// Delete a task from the database
pub fn delete_task(conn: &Connection, task_id: &str) -> Result<()> {
    let rows_affected = conn.execute("DELETE FROM tasks WHERE id = ?1", [task_id])?;

    if rows_affected == 0 {
        return Err(StorageError::TaskNotFound(task_id.to_string()));
    }

    Ok(())
}

/// Clean up finished tasks older than specified days
pub fn cleanup_old_tasks(conn: &Connection, days: u64) -> Result<usize> {
    let cutoff_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        - (days * 24 * 60 * 60) as i64;

    let rows_affected = conn.execute(
        "DELETE FROM tasks WHERE status IN ('exited', 'killed') AND finished_at < ?1",
        [cutoff_time],
    )?;

    Ok(rows_affected)
}

/// Helper function to convert a row to a Task
fn row_to_task(row: &Row) -> SqliteResult<Task> {
    Ok(Task {
        id: row.get(0)?,
        pid: row.get::<_, i64>(1)? as u32,
        pgid: row.get::<_, Option<i64>>(2)?.map(|p| p as i32),
        command: row.get(3)?,
        env: row.get(4)?,
        cwd: row.get(5)?,
        status: row.get(6)?,
        exit_code: row.get::<_, Option<i64>>(7)?.map(|c| c as i32),
        started_at: row.get(8)?,
        finished_at: row.get(9)?,
        log_path: row.get(10)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_database_init() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let conn = Connection::open(db_path).unwrap();

        // Create tables manually for test
        conn.execute(
            r#"
            CREATE TABLE tasks (
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
        )
        .unwrap();

        // Test task insertion
        let task_id = "test-task-1";
        let command = vec!["echo".to_string(), "hello".to_string()];
        let log_path = temp_dir.path().join("test.log");

        insert_task(
            &conn,
            task_id,
            12345,
            Some(12345),
            &command,
            None,
            None,
            &log_path,
        )
        .unwrap();

        // Test task retrieval
        let task = get_task(&conn, task_id).unwrap();
        assert_eq!(task.id, task_id);
        assert_eq!(task.pid, 12345);
        assert_eq!(task.status, "running");

        // Test status update
        update_task_status(&conn, task_id, "exited", Some(0)).unwrap();

        let updated_task = get_task(&conn, task_id).unwrap();
        assert_eq!(updated_task.status, "exited");
        assert_eq!(updated_task.exit_code, Some(0));
        assert!(updated_task.finished_at.is_some());
    }

    #[test]
    fn test_get_tasks() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = Connection::open(db_path).unwrap();

        // Create table
        conn.execute(
            r#"
            CREATE TABLE tasks (
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
        )
        .unwrap();

        // Insert test tasks
        let log_path = temp_dir.path().join("test.log");
        let command = vec!["test".to_string()];

        insert_task(&conn, "task1", 100, None, &command, None, None, &log_path).unwrap();
        insert_task(&conn, "task2", 200, None, &command, None, None, &log_path).unwrap();

        // Update one task status
        update_task_status(&conn, "task1", "exited", Some(0)).unwrap();

        // Test getting all tasks
        let all_tasks = get_tasks(&conn, None).unwrap();
        assert_eq!(all_tasks.len(), 2);

        // Test filtering by status
        let running_tasks = get_tasks(&conn, Some("running")).unwrap();
        assert_eq!(running_tasks.len(), 1);
        assert_eq!(running_tasks[0].id, "task2");

        let exited_tasks = get_tasks(&conn, Some("exited")).unwrap();
        assert_eq!(exited_tasks.len(), 1);
        assert_eq!(exited_tasks[0].id, "task1");
    }

    #[test]
    fn test_get_tasks_with_process_check_updates_nonexistent_processes() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = Connection::open(db_path).unwrap();

        // Create table
        conn.execute(
            r#"
            CREATE TABLE tasks (
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
        )
        .unwrap();

        // Insert test tasks with PIDs that don't exist
        let log_path = temp_dir.path().join("test.log");
        let command = vec!["test".to_string()];

        // Insert task with non-existent PID (99999 should not exist)
        insert_task(&conn, "task1", 99999, None, &command, None, None, &log_path).unwrap();
        // Insert task with another non-existent PID
        insert_task(&conn, "task2", 99998, None, &command, None, None, &log_path).unwrap();

        // Verify both tasks are initially 'running'
        let task1_before = get_task(&conn, "task1").unwrap();
        let task2_before = get_task(&conn, "task2").unwrap();
        assert_eq!(task1_before.status, "running");
        assert_eq!(task2_before.status, "running");

        // Call get_tasks_with_process_check - this should update status for non-existent processes
        let tasks = get_tasks_with_process_check(&conn, None).unwrap();

        // Verify both tasks have been updated to 'exited'
        assert_eq!(tasks.len(), 2);
        for task in &tasks {
            assert_eq!(
                task.status, "exited",
                "Task {} should be marked as exited",
                task.id
            );
            assert!(
                task.finished_at.is_some(),
                "Task {} should have finished_at timestamp",
                task.id
            );
        }

        // Double-check by querying individually
        let task1_after = get_task(&conn, "task1").unwrap();
        let task2_after = get_task(&conn, "task2").unwrap();
        assert_eq!(task1_after.status, "exited");
        assert_eq!(task2_after.status, "exited");
        assert!(task1_after.finished_at.is_some());
        assert!(task2_after.finished_at.is_some());
    }
}
