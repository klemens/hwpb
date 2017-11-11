// These table definitions can be generated using `diesel print-schema`
table! {
    audit_logs (id) {
        id -> Int4,
        created_at -> Timestamptz,
        year -> Int2,
        author -> Text,
        affected_group -> Nullable<Int4>,
        change -> Text,
    }
}

table! {
    completions (group_id, task_id) {
        group_id -> Int4,
        task_id -> Int4,
    }
}

table! {
    days (id) {
        id -> Int4,
        name -> Text,
        year -> Int2,
    }
}

table! {
    elaborations (group_id, experiment_id) {
        group_id -> Int4,
        experiment_id -> Int4,
        rework_required -> Bool,
        accepted -> Bool,
    }
}

table! {
    events (id) {
        id -> Int4,
        day_id -> Int4,
        experiment_id -> Int4,
        date -> Date,
    }
}

table! {
    experiments (id) {
        id -> Int4,
        name -> Text,
        year -> Int2,
    }
}

table! {
    group_mappings (student_id, group_id) {
        student_id -> Int4,
        group_id -> Int4,
    }
}

table! {
    groups (id) {
        id -> Int4,
        desk -> Int4,
        day_id -> Int4,
        comment -> Text,
    }
}

table! {
    students (id) {
        id -> Int4,
        matrikel -> Text,
        name -> Text,
        year -> Int2,
    }
}

table! {
    tasks (id) {
        id -> Int4,
        experiment_id -> Int4,
        name -> Text,
    }
}

joinable!(elaborations -> groups (group_id));
joinable!(events -> days (day_id));
joinable!(events -> experiments (experiment_id));
joinable!(groups -> days (day_id));
joinable!(elaborations -> experiments (experiment_id));
joinable!(tasks -> experiments (experiment_id));
joinable!(completions -> groups (group_id));
joinable!(completions -> tasks (task_id));
joinable!(group_mappings -> students (student_id));
joinable!(group_mappings -> groups (group_id));

// These often occur in the same query, although not directly related
enable_multi_table_joins!(days, experiments);

// Additional non-primary-key joins, for which the on-clause must be
// specified separately
enable_multi_table_joins!(completions, group_mappings); // group_id
enable_multi_table_joins!(elaborations, group_mappings); // group_id
enable_multi_table_joins!(completions, students); // student_id
enable_multi_table_joins!(elaborations, students); // student_id
