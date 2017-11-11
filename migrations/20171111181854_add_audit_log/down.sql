-- These just restore the schema but not the data, which would require
-- parsing the audit log text description.
ALTER TABLE elaborations
    ADD COLUMN accepted_by text NULL;

ALTER TABLE completions
    ADD COLUMN tutor text NULL,
    ADD COLUMN completed_at timestamp with time zone DEFAULT now();

DROP TABLE audit_logs;
