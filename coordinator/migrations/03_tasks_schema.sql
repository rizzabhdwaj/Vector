-- The completed table's output column was created with a typo.
ALTER TABLE tasks_compleated RENAME COLUMN outptu TO output;

-- A command can be submitted more than once, so it must not be unique.
ALTER TABLE tasks_pending DROP CONSTRAINT IF EXISTS tasks_pending_command_key;
ALTER TABLE tasks_compleated DROP CONSTRAINT IF EXISTS tasks_compleated_command_key;
