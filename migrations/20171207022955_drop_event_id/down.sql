--recreate the table because the dropped id column was the first one

CREATE TEMPORARY TABLE events_migration
    AS SELECT * FROM events;

DROP TABLE events;

CREATE TABLE events (
    id SERIAL,
    day_id integer NOT NULL,
    experiment_id integer NOT NULL,
    date date NOT NULL,
    PRIMARY KEY (id),
    UNIQUE (day_id, experiment_id),
    FOREIGN KEY (day_id) REFERENCES days,
    FOREIGN KEY (experiment_id) REFERENCES experiments
);

INSERT INTO events (day_id, experiment_id, date)
    SELECT * FROM events_migration;
