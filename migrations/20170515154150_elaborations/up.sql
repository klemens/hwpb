-- every task has to be accepted by a tutor after it is completed
CREATE TABLE elaborations (
    group_id integer,
    experiment_id text,
    rework_required bool NOT NULL DEFAULT false,
    accepted bool NOT NULL DEFAULT false,
    accepted_by text NULL,
    PRIMARY KEY (group_id, experiment_id),
    FOREIGN KEY (group_id) REFERENCES groups,
    FOREIGN KEY (experiment_id) REFERENCES experiments
);
