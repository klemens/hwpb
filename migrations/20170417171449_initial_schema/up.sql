-- every group (and so student) in enrolled for a specific day (class)
CREATE TABLE days (
    id text,
    PRIMARY KEY (id)
);

-- list of experiments every student has to complete
CREATE TABLE experiments (
    id text,
    PRIMARY KEY (id)
);

-- each experiment is conducted for every day on a specific date (the
-- date is not predictable from the day because of holidays)
CREATE TABLE events (
    id SERIAL,
    day_id text NOT NULL,
    experiment_id text NOT NULL,
    date date NOT NULL,
    PRIMARY KEY (id),
    UNIQUE (day_id, experiment_id),
    FOREIGN KEY (day_id) REFERENCES days,
    FOREIGN KEY (experiment_id) REFERENCES experiments
);

-- groups consist of several students that work together on the same
-- desk and are graded together
CREATE TABLE groups (
    id SERIAL,
    desk integer NOT NULL,
    day_id text NOT NULL,
    comment text NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (day_id) REFERENCES days
);

-- students, each with its group if already assigned
CREATE TABLE students (
    id text,
    name text NOT NULL,
    group_id integer NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (group_id) REFERENCES groups
);

-- experiments consist of several individual tasks
CREATE TABLE tasks (
    id SERIAL,
    experiment_id text NOT NULL,
    name text NOT NULL,
    PRIMARY KEY (id),
    UNIQUE (experiment_id, name),
    FOREIGN KEY (experiment_id) REFERENCES experiments
);

-- every task has to be accepted by a tutor after it is completed
CREATE TABLE completions (
    group_id integer,
    task_id integer,
    tutor text NULL,
    PRIMARY KEY (group_id, task_id),
    FOREIGN KEY (group_id) REFERENCES groups,
    FOREIGN KEY (task_id) REFERENCES tasks
);
