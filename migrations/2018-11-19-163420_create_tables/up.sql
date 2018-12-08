CREATE TABLE tasks (
  id SERIAL PRIMARY KEY,
  rootnum INT NOT NULL DEFAULT 1,
  replnum INT NOT NULL DEFAULT 1,
  posted TIMESTAMPTZ NOT NULL default current_timestamp,
  whosent VARCHAR NOT NULL,
  attached VARCHAR DEFAULT NULL,
  editable BOOLEAN NOT NULL DEFAULT 'f',
  description VARCHAR NOT NULL
);

CREATE TABLE secrets (
  id SERIAL PRIMARY KEY,
  secret VARCHAR NOT NULL,
  taskid INT NOT NULL,
  foreign key (taskid) references tasks(id) on delete cascade
);