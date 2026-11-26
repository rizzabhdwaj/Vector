use crate::task_queue::queue::Task;
use sqlx::PgPool;
use sqlx::Row;
use sqlx::postgres::PgPoolOptions;

pub async fn init_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
}

pub async fn load_pending_tasks(pool: &PgPool) -> Result<Vec<Task>, sqlx::Error> {
    let rows =
        sqlx::query("SELECT id, command, args, timelimit, status FROM tasks_pending ORDER BY id")
            .fetch_all(pool)
            .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let id: i32 = row.get("id");
            let command: String = row.get("command");
            let args: Option<Vec<String>> = row.get("args");
            let timelimit: i64 = row.get("timelimit");
            let status: Option<String> = row.get("status");

            Task::from_pending(
                id as u32,
                command,
                args.unwrap_or_default(),
                timelimit as u32,
                status.as_deref().unwrap_or("queued"),
            )
        })
        .collect())
}
