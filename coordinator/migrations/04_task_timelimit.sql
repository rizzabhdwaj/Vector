-- Per-task execution time limit (seconds) carried from the request to the worker.
ALTER TABLE tasks_pending ADD COLUMN IF NOT EXISTS timelimit BIGINT NOT NULL DEFAULT 15;
ALTER TABLE tasks_compleated ADD COLUMN IF NOT EXISTS timelimit BIGINT NOT NULL DEFAULT 15;
