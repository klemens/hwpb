-- promote (day_id, experiment_id) to be the new primary key

ALTER TABLE events
    DROP COLUMN id,
    DROP CONSTRAINT events_day_id_experiment_id_key,
    ADD PRIMARY KEY (day_id, experiment_id);
