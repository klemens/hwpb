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
    events (day_id, experiment_id) {
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
    ip_whitelist (id) {
        id -> Int4,
        ipnet -> Inet,
        year -> Int2,
    }
}

table! {
    students (id) {
        id -> Int4,
        matrikel -> Text,
        name -> Text,
        year -> Int2,
        username -> Nullable<Text>,
        instructed -> Bool,
    }
}

table! {
    tasks (id) {
        id -> Int4,
        experiment_id -> Int4,
        name -> Text,
    }
}

table! {
    tutors (id) {
        id -> Int4,
        username -> Text,
        year -> Int2,
        is_admin -> Bool,
    }
}

table! {
    years (id) {
        id -> Int2,
        writable -> Bool,
    }
}

joinable!(completions -> groups (group_id));
joinable!(completions -> tasks (task_id));
joinable!(days -> years (year));
joinable!(elaborations -> experiments (experiment_id));
joinable!(elaborations -> groups (group_id));
joinable!(events -> days (day_id));
joinable!(events -> experiments (experiment_id));
joinable!(experiments -> years (year));
joinable!(group_mappings -> groups (group_id));
joinable!(group_mappings -> students (student_id));
joinable!(groups -> days (day_id));
joinable!(students -> years (year));
joinable!(tasks -> experiments (experiment_id));

allow_tables_to_appear_in_same_query!(
    audit_logs,
    completions,
    days,
    elaborations,
    events,
    experiments,
    group_mappings,
    groups,
    ip_whitelist,
    students,
    tasks,
    tutors,
    years,
);
