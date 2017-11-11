CREATE TABLE audit_logs (
    id SERIAL PRIMARY KEY,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    year smallint NOT NULL,
    author text NOT NULL,
    affected_group integer NULL,
    change text NOT NULL
);

-- Import the existing data before dropping it using the same format
-- like when adding these through code.
INSERT INTO audit_logs (created_at, year, author, affected_group, change)
    SELECT completed_at, year, tutor, group_id,
        format('Mark task %s (#%s) of %s as completed', tasks.name, task_id, experiments.name)
    FROM completions
        INNER JOIN tasks ON task_id = tasks.id
        INNER JOIN experiments ON experiment_id = experiments.id
    WHERE tutor IS NOT NULL AND completed_at IS NOT NULL
    ORDER BY completed_at ASC;

-- Elaborations did not store the date before, so we just set the date
-- to one day after the last completion, so they at least stay logically
-- consistent.
INSERT INTO audit_logs (created_at, year, author, affected_group, change)
    SELECT (
            SELECT date_trunc('day', max(completed_at)) + interval '1 day'
            FROM completions
        ), year, accepted_by, group_id,
        format('Mark elaboration of %s (#%s) as %s', experiments.name, experiment_id,
        CASE (rework_required, accepted)
            WHEN (false, false) THEN 'submitted'
            WHEN (false,  true) THEN 'accepted'
            WHEN ( true, false) THEN 'needing rework'
            WHEN ( true,  true) THEN 'rework accepted'
        END)
    FROM elaborations
        INNER JOIN experiments ON experiment_id = experiments.id
    WHERE accepted_by IS NOT NULL;

ALTER TABLE completions
    DROP COLUMN tutor,
    DROP COLUMN completed_at;

ALTER TABLE elaborations
    DROP COLUMN accepted_by;
