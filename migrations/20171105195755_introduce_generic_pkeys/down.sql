-- drop primary keys and referening foreign key constraints (CASCADE)
ALTER TABLE days
    DROP CONSTRAINT days_pkey CASCADE;
ALTER TABLE experiments
    DROP CONSTRAINT experiments_pkey CASCADE;
ALTER TABLE students
    DROP CONSTRAINT students_pkey CASCADE;


-- convert integer to text keys by using a temporary column (SET DATA TYPE does
-- not support subqueries)
ALTER TABLE events
    ADD COLUMN day_tmp text NULL,
    ADD COLUMN experiment_tmp text NULL;
UPDATE events SET
    day_tmp = (SELECT name FROM days
        WHERE days.id = events.day_id),
    experiment_tmp = (SELECT name FROM experiments
        WHERE experiments.id = events.experiment_id);
ALTER TABLE events
    ALTER COLUMN day_id SET DATA TYPE text USING day_tmp,
    ALTER COLUMN experiment_id SET DATA TYPE text USING experiment_tmp,
    DROP COLUMN day_tmp,
    DROP COLUMN experiment_tmp;

ALTER TABLE groups
    ADD COLUMN day_tmp text NULL;
UPDATE groups SET
    day_tmp = (SELECT name FROM days
        WHERE days.id = groups.day_id);
ALTER TABLE groups
    ALTER COLUMN day_id SET DATA TYPE text USING day_tmp,
    DROP COLUMN day_tmp;

ALTER TABLE elaborations
    ADD COLUMN experiment_tmp text NULL;
UPDATE elaborations SET
    experiment_tmp = (SELECT name FROM experiments
        WHERE experiments.id = elaborations.experiment_id);
ALTER TABLE elaborations
    ALTER COLUMN experiment_id SET DATA TYPE text USING experiment_tmp,
    DROP COLUMN experiment_tmp;

ALTER TABLE tasks
    ADD COLUMN experiment_tmp text NULL;
UPDATE tasks SET
    experiment_tmp = (SELECT name FROM experiments
        WHERE experiments.id = tasks.experiment_id);
ALTER TABLE tasks
    ALTER COLUMN experiment_id SET DATA TYPE text USING experiment_tmp,
    DROP COLUMN experiment_tmp;

ALTER TABLE group_mappings
    ADD COLUMN student_tmp text NULL;
UPDATE group_mappings SET
    student_tmp = (SELECT matrikel FROM students
        WHERE students.id = group_mappings.student_id);
ALTER TABLE group_mappings
    ALTER COLUMN student_id SET DATA TYPE text USING student_tmp,
    DROP COLUMN student_tmp;


-- remove integer key columns, rename old columns to id and re-add their primary
-- key constraints
ALTER TABLE days
    DROP COLUMN id;
ALTER TABLE days
    RENAME COLUMN name to id;
ALTER TABLE days
    ADD PRIMARY KEY (id);

ALTER TABLE experiments
    DROP COLUMN id;
ALTER TABLE experiments
    RENAME COLUMN name to id;
ALTER TABLE experiments
    ADD PRIMARY KEY (id);

ALTER TABLE students
    DROP COLUMN id;
ALTER TABLE students
    RENAME COLUMN matrikel to id;
ALTER TABLE students
    ADD PRIMARY KEY (id);


-- re-add the foreing key constraints
ALTER TABLE events
    ADD FOREIGN KEY (day_id) REFERENCES days,
    ADD FOREIGN KEY (experiment_id) REFERENCES experiments;

ALTER TABLE groups
    ADD FOREIGN KEY (day_id) REFERENCES days;

ALTER TABLE elaborations
    ADD FOREIGN KEY (experiment_id) REFERENCES experiments;

ALTER TABLE tasks
    ADD FOREIGN KEY (experiment_id) REFERENCES experiments;

ALTER TABLE group_mappings
    ADD FOREIGN KEY (student_id) REFERENCES students;
