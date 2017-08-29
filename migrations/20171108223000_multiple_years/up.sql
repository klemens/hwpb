-- Use the year 2017 for all existing rows

ALTER TABLE days
    ADD COLUMN year smallint NOT NULL DEFAULT 2017;
ALTER TABLE days
    ALTER COLUMN year DROP DEFAULT;

ALTER TABLE experiments
    ADD COLUMN year smallint NOT NULL DEFAULT 2017;
ALTER TABLE experiments
    ALTER COLUMN year DROP DEFAULT;

ALTER TABLE students
    ADD COLUMN year smallint NOT NULL DEFAULT 2017;
ALTER TABLE students
    ALTER COLUMN year DROP DEFAULT;
