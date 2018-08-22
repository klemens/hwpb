ALTER TABLE tutors
    ADD FOREIGN KEY (year) REFERENCES years;

ALTER TABLE ip_whitelist
    ADD FOREIGN KEY (year) REFERENCES years;
