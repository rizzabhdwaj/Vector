CREATE TABLE tasks_compleated(
    id SERIAL PRIMARY KEY,
    command TEXT UNIQUE NOT NULL,
    args TEXT[],
    worker NUMERIC ,
    outptu TEXT ,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
