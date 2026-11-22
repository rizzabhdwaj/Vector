CREATE TABLE tasks_compleated(
    id SERIAL PRIMARY KEY,
    command TEXT UNIQUE NOT NULL,
    args TEXT[],
    worker NUMERIC ,
    output TEXT ,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
