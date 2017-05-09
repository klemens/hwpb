ALTER TABLE completions
    ADD COLUMN completed_at timestamp with time zone DEFAULT now();
