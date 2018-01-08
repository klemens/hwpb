ALTER TABLE students
    ADD COLUMN username TEXT NULL;
UPDATE students
    SET username = matrikel;
