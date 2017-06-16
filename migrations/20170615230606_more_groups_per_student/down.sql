ALTER TABLE students
    ADD COLUMN group_id integer NULL,
    ADD FOREIGN KEY (group_id) REFERENCES groups;

DROP TABLE group_mappings;
