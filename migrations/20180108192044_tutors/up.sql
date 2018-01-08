CREATE TABLE tutors (
    id SERIAL PRIMARY KEY,
    username text NOT NULL,
    year smallint NOT NULL,
    is_admin boolean NOT NULL
);
