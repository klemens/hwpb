CREATE TABLE group_mappings (
    student_id text,
    group_id integer,
    PRIMARY KEY (student_id, group_id),
    FOREIGN KEY (student_id) REFERENCES students,
    FOREIGN KEY (group_id) REFERENCES groups
);

ALTER TABLE students
    DROP COLUMN group_id;
