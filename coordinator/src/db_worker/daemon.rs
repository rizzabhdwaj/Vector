use crate::scheduler::event::Event;
use sqlx::PgPool;

pub struct Persister {
    rx: tokio::sync::mpsc::Receiver<Event>,
    pool: PgPool,
}

impl Persister {
    pub fn new(rx: tokio::sync::mpsc::Receiver<Event>, pool: PgPool) -> Self {
        Self { rx, pool }
    }

    pub async fn run(mut self) -> Result<(), sqlx::Error> {
        while let Some(ev) = self.rx.recv().await {
            eprintln!("persister received event: {ev:?}");
            // A failed write must not take down the coordinator; log it and keep draining.
            if let Err(err) = self.apply(&ev).await {
                eprintln!("persister failed to apply event: {err}");
            }
        }
        Ok(())
    }

    async fn apply(&self, ev: &Event) -> Result<(), sqlx::Error> {
        match ev {
            Event::TaskRunning { id, worker_id } => {
                sqlx::query("UPDATE tasks_pending SET status='running', worker=$2 WHERE id=$1")
                    .bind(*id as i64)
                    .bind(*worker_id)
                    .execute(&self.pool)
                    .await?;
            }
            Event::TaskFinished { id, output, .. } => {
                // Move the task out of the pending table once the worker reports back.
                sqlx::query(
                    "WITH moved AS (
                         DELETE FROM tasks_pending WHERE id = $1
                         RETURNING id, command, args, worker, timelimit, created_at
                     )
                     INSERT INTO tasks_compleated (id, command, args, worker, timelimit, output, created_at)
                     SELECT id, command, args, worker, timelimit, $2, created_at FROM moved
                     ON CONFLICT (id) DO UPDATE SET
                         command = EXCLUDED.command,
                         args = EXCLUDED.args,
                         worker = EXCLUDED.worker,
                         timelimit = EXCLUDED.timelimit,
                         output = EXCLUDED.output,
                         created_at = EXCLUDED.created_at",
                )
                .bind(*id as i64)
                .bind(output.as_str())
                .execute(&self.pool)
                .await?;
            }
            Event::WorkerDead { id } => {
                sqlx::query("UPDATE workers SET status='dead' WHERE id=$1")
                    .bind(*id as i64)
                    .execute(&self.pool)
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }
}

