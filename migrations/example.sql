INSERT INTO days VALUES
    ('Di'), ('Do');

INSERT INTO experiments VALUES
    ('Versuch 1');

INSERT INTO events VALUES
    (1, 'Di', 'Versuch 1', '2017-04-18'),
    (2, 'Do', 'Versuch 1', '2017-04-20');

INSERT INTO groups VALUES
    (1, 1, 'Di', ''),
    (2, 2, 'Di', 'Partner nicht erschienen'),
    (3, 1, 'Do', '');

INSERT INTO students VALUES
    ('fm41abdf', 'Franz Maier', 1),
    ('aw43cldu', 'Anna Walter', 1),
    ('ph73aoxo', 'Peter Huber', 2),
    ('ms18gwhd', 'Maria Schneider', 3),
    ('az63zbwp', 'Alex Zimmer', 3);

INSERT INTO tasks VALUES
    (1, 'Versuch 1', '1a'),
    (2, 'Versuch 1', '1b'),
    (3, 'Versuch 1', '2');

INSERT INTO completions VALUES
    (1, 1, 'John Anonymous'),
    (1, 2, 'John Anonymous'),
    (2, 1, 'John Anonymous'),
    (2, 2, 'John Anonymous'),
    (2, 3, 'John Anonymous'),
    (3, 2, 'John Anonymous');
