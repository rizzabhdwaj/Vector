CREATE TABLE tasks_compleated(
    id SERIAL PRIMARY KEY,
    command TEXT UNIQUE NOT NULL,
    args TEXT[],
    worker NUMERIC ,
    output TEXT ,
    timelimit BIGINT NOT NULL DEFAULT 15,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
