-- drop primary keys and referening foreign key constraints (CASCADE)
ALTER TABLE days
    DROP CONSTRAINT days_pkey CASCADE;
ALTER TABLE experiments
    DROP CONSTRAINT experiments_pkey CASCADE;
ALTER TABLE students
    DROP CONSTRAINT students_pkey CASCADE;


-- add serial ids by recreating the tables
ALTER TABLE days RENAME TO days_old;
CREATE TABLE days (
    id SERIAL,
    name text NOT NULL,
    PRIMARY KEY (id)
);
INSERT INTO days (name)
    SELECT * FROM days_old;
DROP TABLE days_old;

ALTER TABLE experiments RENAME TO experiments_old;
CREATE TABLE experiments (
    id SERIAL,
    name text NOT NULL,
    PRIMARY KEY (id)
);
INSERT INTO experiments (name)
    SELECT * FROM experiments_old;
DROP TABLE experiments_old;

ALTER TABLE students RENAME TO students_old;
CREATE TABLE students (
    id SERIAL,
    matrikel text NOT NULL,
    name text NOT NULL,
    PRIMARY KEY (id)
);
INSERT INTO students (matrikel, name)
    SELECT * FROM students_old;
DROP TABLE students_old;


-- convert text to integer keys by using a temporary column (SET DATA TYPE does
-- not support subqueries and we have to keep the order to be able to revert
-- this change easily) and re-add the foreign key constraints
ALTER TABLE events
    ADD COLUMN day_tmp integer NULL,
    ADD COLUMN experiment_tmp integer NULL;
UPDATE events SET
    day_tmp = (SELECT id FROM days
        WHERE days.name = events.day_id),
    experiment_tmp = (SELECT id FROM experiments
        WHERE experiments.name = events.experiment_id);
ALTER TABLE events
    ALTER COLUMN day_id SET DATA TYPE integer USING day_tmp,
    ALTER COLUMN experiment_id SET DATA TYPE integer USING experiment_tmp,
    DROP COLUMN day_tmp,
    DROP COLUMN experiment_tmp,
    ADD FOREIGN KEY (day_id) REFERENCES days,
    ADD FOREIGN KEY (experiment_id) REFERENCES experiments;

ALTER TABLE groups
    ADD COLUMN day_tmp integer NULL;
UPDATE groups SET
    day_tmp = (SELECT id FROM days
        WHERE days.name = groups.day_id);
ALTER TABLE groups
    ALTER COLUMN day_id SET DATA TYPE integer USING day_tmp,
    DROP COLUMN day_tmp,
    ADD FOREIGN KEY (day_id) REFERENCES days;

ALTER TABLE elaborations
    ADD COLUMN experiment_tmp integer NULL;
UPDATE elaborations SET
    experiment_tmp = (SELECT id FROM experiments
        WHERE experiments.name = elaborations.experiment_id);
ALTER TABLE elaborations
    ALTER COLUMN experiment_id SET DATA TYPE integer USING experiment_tmp,
    DROP COLUMN experiment_tmp,
    ADD FOREIGN KEY (experiment_id) REFERENCES experiments;

ALTER TABLE tasks
    ADD COLUMN experiment_tmp integer NULL;
UPDATE tasks SET
    experiment_tmp = (SELECT id FROM experiments
        WHERE experiments.name = tasks.experiment_id);
ALTER TABLE tasks
    ALTER COLUMN experiment_id SET DATA TYPE integer USING experiment_tmp,
    DROP COLUMN experiment_tmp,
    ADD FOREIGN KEY (experiment_id) REFERENCES experiments;

ALTER TABLE group_mappings
    ADD COLUMN student_tmp integer NULL;
UPDATE group_mappings SET
    student_tmp = (SELECT id FROM students
        WHERE students.matrikel = group_mappings.student_id);
ALTER TABLE group_mappings
    ALTER COLUMN student_id SET DATA TYPE integer USING student_tmp,
    DROP COLUMN student_tmp,
    ADD FOREIGN KEY (student_id) REFERENCES students;
