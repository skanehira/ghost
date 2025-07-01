use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, Result as SqliteResult, Row};
use serde::{Deserialize, Serialize};

use crate::app::{process::ProcessError, process_state};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Running,
    Exited,
    Killed,
    Unknown,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TaskStatus {
    /// Convert TaskStatus to string for database storage
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Running => "running",
            TaskStatus::Exited => "exited",
            TaskStatus::Killed => "killed",
            TaskStatus::Unknown => "unknown",
        }
    }

    /// Parse TaskStatus from string (for database retrieval)
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> TaskStatus {
        s.parse().unwrap_or(TaskStatus::Unknown)
    }
}

impl std::str::FromStr for TaskStatus {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "running" => Ok(TaskStatus::Running),
            "exited" => Ok(TaskStatus::Exited),
            "killed" => Ok(TaskStatus::Killed),
            "unknown" => Ok(TaskStatus::Unknown),
            _ => Err(format!("Unknown task status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub pid: u32,
    pub pgid: Option<i32>,
    pub command: String,     // JSON serialized Vec<String>
    pub env: Option<String>, // JSON serialized environment variables
    pub cwd: Option<String>,
    pub status: TaskStatus,
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

    #[error("Process error: {0}")]
    Process(#[from] ProcessError),
}

pub type Result<T> = std::result::Result<T, StorageError>;

/// Get the database file path
pub fn get_db_path() -> PathBuf {
    crate::app::config::get_db_path()
}

/// Initialize the database and create tables if they don't exist
pub fn init_database() -> Result<Connection> {
    let config = crate::app::config::Config::default();
    config.ensure_directories()?;

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
            status: TaskStatus::from_str(&row.get::<_, String>(6)?),
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
        if task.status == TaskStatus::Running {
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
    status: TaskStatus,
    exit_code: Option<i32>,
) -> Result<Option<i64>> {
    let finished_at = if matches!(status, TaskStatus::Exited | TaskStatus::Killed) {
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
        (
            status.as_str(),
            exit_code.map(|c| c as i64),
            finished_at,
            task_id,
        ),
    )?;

    Ok(finished_at)
}

/// Update task status by checking if process is still running
pub fn update_task_status_by_process_check(conn: &Connection, task_id: &str) -> Result<Task> {
    let mut task = get_task(conn, task_id)?;

    // Use process_state module to check and update status
    if process_state::update_task_status_if_needed(&mut task) {
        // Status was updated, persist to database
        update_task_status(conn, task_id, task.status, None)?;
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
        "DELETE FROM tasks WHERE status IN ('exited', 'killed') AND finished_at IS NOT NULL AND finished_at < ?1",
        [cutoff_time],
    )?;

    Ok(rows_affected)
}

/// Get tasks that would be cleaned up (for dry-run)
pub fn get_cleanup_candidates(
    conn: &Connection,
    days: Option<u64>,
    status_filter: &[TaskStatus],
) -> Result<Vec<Task>> {
    let mut sql = "SELECT id, pid, pgid, command, env, cwd, status, exit_code, started_at, finished_at, log_path FROM tasks WHERE 1=1".to_string();
    let mut params: Vec<Box<dyn rusqlite::ToSql + '_>> = Vec::new();

    // Add status filter
    if !status_filter.is_empty() {
        let status_placeholders = status_filter
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");
        sql.push_str(&format!(" AND status IN ({status_placeholders})"));

        for status in status_filter {
            params.push(Box::new(status.as_str()));
        }
    }

    // Add time filter if specified
    if let Some(days) = days {
        let cutoff_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            - (days * 24 * 60 * 60) as i64;

        sql.push_str(" AND finished_at IS NOT NULL AND finished_at < ?");
        params.push(Box::new(cutoff_time));
    }

    sql.push_str(" ORDER BY finished_at DESC");

    let mut stmt = conn.prepare(&sql)?;
    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let task_iter = stmt.query_map(&param_refs[..], row_to_task)?;

    let mut tasks = Vec::new();
    for task in task_iter {
        tasks.push(task?);
    }

    Ok(tasks)
}

/// Clean up tasks with more granular control
pub fn cleanup_tasks_by_criteria(
    conn: &Connection,
    days: Option<u64>,
    status_filter: &[TaskStatus],
) -> Result<usize> {
    // First, get the tasks that will be deleted (to access log files)
    let tasks_to_delete = get_cleanup_candidates(conn, days, status_filter)?;

    if tasks_to_delete.is_empty() {
        return Ok(0);
    }

    // Delete log files first
    for task in &tasks_to_delete {
        if std::path::Path::new(&task.log_path).exists() {
            if let Err(e) = std::fs::remove_file(&task.log_path) {
                eprintln!(
                    "Warning: Failed to delete log file {}: {}",
                    task.log_path, e
                );
            }
        }
    }

    // Then delete from database using task IDs
    let task_ids: Vec<_> = tasks_to_delete.iter().map(|t| &t.id).collect();
    let placeholders = task_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!("DELETE FROM tasks WHERE id IN ({placeholders})");

    let param_refs: Vec<&dyn rusqlite::ToSql> = task_ids
        .iter()
        .map(|id| *id as &dyn rusqlite::ToSql)
        .collect();
    let rows_affected = conn.execute(&sql, &param_refs[..])?;

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
        status: TaskStatus::from_str(&row.get::<_, String>(6)?),
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
        assert_eq!(task.status, TaskStatus::Running);

        // Test status update
        update_task_status(&conn, task_id, TaskStatus::Exited, Some(0)).unwrap();

        let updated_task = get_task(&conn, task_id).unwrap();
        assert_eq!(updated_task.status, TaskStatus::Exited);
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
        update_task_status(&conn, "task1", TaskStatus::Exited, Some(0)).unwrap();

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
        assert_eq!(task1_before.status, TaskStatus::Running);
        assert_eq!(task2_before.status, TaskStatus::Running);

        // Call get_tasks_with_process_check - this should update status for non-existent processes
        let tasks = get_tasks_with_process_check(&conn, None).unwrap();

        // Verify both tasks have been updated to 'exited'
        assert_eq!(tasks.len(), 2);
        for task in &tasks {
            assert_eq!(
                task.status,
                TaskStatus::Exited,
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
        assert_eq!(task1_after.status, TaskStatus::Exited);
        assert_eq!(task2_after.status, TaskStatus::Exited);
        assert!(task1_after.finished_at.is_some());
        assert!(task2_after.finished_at.is_some());
    }
}
