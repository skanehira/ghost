use rusqlite::Connection;

use super::task::Task;
use super::task_repository::{row_to_task, update_task_status_by_process_check};
use super::task_status::TaskStatus;
use crate::app::error::Result;

/// Clean up old tasks (legacy function)
pub fn cleanup_old_tasks(conn: &Connection, days: u64) -> Result<usize> {
    let cutoff_time = crate::app::helpers::now_timestamp() - (days * 24 * 60 * 60) as i64;

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
    // First, update status for all running tasks
    let running_sql = "SELECT id FROM tasks WHERE status = 'running'";
    let mut running_stmt = conn.prepare(running_sql)?;
    let running_ids: Vec<String> = running_stmt
        .query_map([], |row| row.get(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    // Update status for each running task
    for task_id in running_ids {
        update_task_status_by_process_check(conn, &task_id)?;
    }

    // Now get cleanup candidates with filters applied
    let mut sql = "SELECT id, pid, pgid, command, env, cwd, status, exit_code, started_at, finished_at, log_path FROM tasks".to_string();
    let mut params: Vec<Box<dyn rusqlite::ToSql + '_>> = Vec::new();
    let mut conditions = Vec::new();

    // Add status filter
    if !status_filter.is_empty() {
        let status_placeholders = status_filter
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");
        conditions.push(format!("status IN ({status_placeholders})"));

        for status in status_filter {
            params.push(Box::new(status.as_str()));
        }
    }

    // Add time filter if specified
    if let Some(days) = days {
        let cutoff_time = crate::app::helpers::now_timestamp() - (days * 24 * 60 * 60) as i64;

        conditions.push("finished_at IS NOT NULL AND finished_at < ?".to_string());
        params.push(Box::new(cutoff_time));
    }

    // Build WHERE clause if we have conditions
    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
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
        if std::path::Path::new(&task.log_path).exists()
            && let Err(e) = std::fs::remove_file(&task.log_path)
        {
            eprintln!(
                "Warning: Failed to delete log file {}: {}",
                task.log_path, e
            );
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
