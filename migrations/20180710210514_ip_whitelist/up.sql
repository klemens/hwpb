CREATE TABLE ip_whitelist (
    id SERIAL PRIMARY KEY,
    ipnet inet NOT NULL,
    year smallint NOT NULL
);
