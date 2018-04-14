ALTER TABLE students
    ADD COLUMN instructed boolean NOT NULL DEFAULT false;

UPDATE students
    SET instructed = true WHERE year <= 2017;
