-- recreate the table because we cannot insert the name column
-- in the middle

CREATE TEMPORARY TABLE students_migration
    AS SELECT * FROM students;

DROP TABLE students CASCADE;

CREATE TABLE students (
    id SERIAL,
    matrikel text NOT NULL,
    name text NOT NULL,
    year smallint NOT NULL,
    username text,
    instructed boolean DEFAULT false NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (year) REFERENCES years
);

INSERT INTO students (id, matrikel, name, year, username, instructed)
    SELECT
        id, matrikel, trim(given_name || ' ' || family_name),
        year, username, instructed
    FROM students_migration;

SELECT setval(
    'students_id_seq',
    COALESCE((SELECT MAX(id) + 1 FROM students), 1),
    false
);

ALTER TABLE group_mappings
    ADD FOREIGN KEY (student_id) REFERENCES students;
