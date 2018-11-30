CREATE TABLE tasks (
  id SERIAL PRIMARY KEY,
  posted TIMESTAMPTZ NOT NULL default current_timestamp,
  whosent VARCHAR NOT NULL,
  secret VARCHAR NOT NULL,
  completed BOOLEAN NOT NULL DEFAULT 'f',
  description VARCHAR NOT NULL
);