use std::path::Path;

use rusqlite::{Connection, Result as SqliteResult, Row};

use super::task::Task;
use super::task_status::TaskStatus;
use crate::app::error::Result;
use crate::app::process_state;

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
    let started_at = crate::app::helpers::now_timestamp();

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

    let task = stmt
        .query_row([task_id], row_to_task)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => crate::app::error::GhostError::TaskNotFound {
                task_id: task_id.to_string(),
            },
            _ => e.into(),
        })?;
    Ok(task)
}

/// Get all tasks, optionally filtered by status
pub fn get_tasks(conn: &Connection, status_filter: Option<&str>) -> Result<Vec<Task>> {
    let base_sql = "SELECT id, pid, pgid, command, env, cwd, status, exit_code, started_at, finished_at, log_path FROM tasks";
    let order_clause = " ORDER BY started_at DESC";

    let sql = match status_filter {
        Some(_) => format!("{base_sql} WHERE status = ?1{order_clause}"),
        None => format!("{base_sql}{order_clause}"),
    };

    let mut stmt = conn.prepare(&sql)?;

    let task_iter = match status_filter {
        Some(status) => stmt.query_map([status], row_to_task)?,
        None => stmt.query_map([], row_to_task)?,
    };

    let mut tasks = Vec::new();
    for task in task_iter {
        tasks.push(task?);
    }

    Ok(tasks)
}

/// Get all tasks with process status checking
pub fn get_tasks_with_process_check(
    conn: &Connection,
    status_filter: Option<&str>,
) -> Result<Vec<Task>> {
    let mut tasks = get_tasks(conn, status_filter)?;

    // Update status for running tasks
    for task in &mut tasks {
        if task.status == TaskStatus::Running
            && let Ok(updated_task) = update_task_status_by_process_check(conn, &task.id)
        {
            *task = updated_task;
        }
    }

    Ok(tasks)
}

/// Update task status
pub fn update_task_status(
    conn: &Connection,
    task_id: &str,
    new_status: TaskStatus,
    exit_code: Option<i32>,
) -> Result<()> {
    let finished_at = if matches!(new_status, TaskStatus::Running) {
        None
    } else {
        Some(crate::app::helpers::now_timestamp())
    };

    conn.execute(
        "UPDATE tasks SET status = ?1, exit_code = ?2, finished_at = ?3 WHERE id = ?4",
        (new_status.as_str(), exit_code, finished_at, task_id),
    )?;

    Ok(())
}

/// Update task status by checking if the process is still running
pub fn update_task_status_by_process_check(conn: &Connection, task_id: &str) -> Result<Task> {
    let task = get_task(conn, task_id)?;

    if task.status == TaskStatus::Running {
        let new_status = process_state::determine_task_status(task.pid);
        update_task_status(conn, task_id, new_status, None)?;

        // Return updated task
        get_task(conn, task_id)
    } else {
        Ok(task)
    }
}

/// Delete a task by ID
pub fn delete_task(conn: &Connection, task_id: &str) -> Result<()> {
    let rows_affected = conn.execute("DELETE FROM tasks WHERE id = ?1", [task_id])?;

    if rows_affected == 0 {
        return Err(crate::app::error::GhostError::TaskNotFound {
            task_id: task_id.to_string(),
        });
    }

    Ok(())
}

/// Helper function to convert a row to a Task
pub fn row_to_task(row: &Row) -> SqliteResult<Task> {
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
