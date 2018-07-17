-- split name column into given and family name, but keep the full
-- name only in given_name for now (as splitting names correctly is
-- impossible in general)

ALTER TABLE students
    ADD COLUMN given_name text NOT NULL DEFAULT '',
    ADD COLUMN family_name text NOT NULL DEFAULT '';

UPDATE students SET
    given_name = name;

ALTER TABLE students
    ALTER COLUMN given_name DROP DEFAULT,
    ALTER COLUMN family_name DROP DEFAULT,
    DROP COLUMN name;
