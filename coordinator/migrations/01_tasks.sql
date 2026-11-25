CREATE TABLE tasks_pending(
    id SERIAL PRIMARY KEY,
    command TEXT UNIQUE NOT NULL,
    args TEXT[],
    worker NUMERIC ,
    status TEXT , --- Queued / Running / Crashed 
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
