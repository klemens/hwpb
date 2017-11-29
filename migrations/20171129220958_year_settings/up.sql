CREATE TABLE years (
    id smallint,
    writable boolean NOT NULL DEFAULT true,
    PRIMARY KEY(id)
);

-- collect existing years before adding foreign keys
INSERT INTO years (id)
    SELECT DISTINCT year
    FROM days
    ORDER BY year ASC;

ALTER TABLE days
    ADD FOREIGN KEY (year) REFERENCES years;
ALTER TABLE experiments
    ADD FOREIGN KEY (year) REFERENCES years;
ALTER TABLE students
    ADD FOREIGN KEY (year) REFERENCES years;
