CREATE TABLE tasks (
  id SERIAL PRIMARY KEY,
  posted TIMESTAMPTZ NOT NULL default current_timestamp,
  completed BOOLEAN NOT NULL DEFAULT 'f',
  description VARCHAR NOT NULL
);