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
        
        

        for let Some(ev) = self.rx.recv().await {
            eprintln!("persister received event: {ev:?}");
            // Apply the event to the database
            self.apply(&ev).await?;
        }
        //Ok(())
    }

    async fn apply(&self, ev: &Event) -> Result<(), sqlx::Error> {
        match ev {
            Event::TaskAssigned { id, worker_id } => {
                sqlx::query("UPDATE tasks SET status='running', worker_id=$2 WHERE id=$1")
                    .bind(*id as i64)
                    .bind(*worker_id)
                    .execute(&self.pool)
                    .await?;
            }
            Event::TaskFinished { id, .. } => {
                sqlx::query("UPDATE tasks SET status='completed' WHERE id=$1")
                    .bind(*id as i64)
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

