// This can be used during development for generating the table
// definitions from the database schema
//infer_schema!("dotenv:DATABASE_URL");

// These table definitions can be generated using `diesel print-schema`
table! {
    completions (group_id, task_id) {
        group_id -> Int4,
        task_id -> Int4,
        tutor -> Nullable<Text>,
        completed_at -> Nullable<Timestamptz>,
    }
}

table! {
    days (id) {
        id -> Text,
    }
}

table! {
    elaborations (group_id, experiment_id) {
        group_id -> Int4,
        experiment_id -> Text,
        rework_required -> Bool,
        accepted -> Bool,
        accepted_by -> Nullable<Text>,
    }
}

table! {
    events (id) {
        id -> Int4,
        day_id -> Text,
        experiment_id -> Text,
        date -> Date,
    }
}

table! {
    experiments (id) {
        id -> Text,
    }
}

table! {
    group_mappings (student_id, group_id) {
        student_id -> Text,
        group_id -> Int4,
    }
}

table! {
    groups (id) {
        id -> Int4,
        desk -> Int4,
        day_id -> Text,
        comment -> Text,
    }
}

table! {
    students (id) {
        id -> Text,
        name -> Text,
    }
}

table! {
    tasks (id) {
        id -> Int4,
        experiment_id -> Text,
        name -> Text,
    }
}

joinable!(events -> days (day_id));
joinable!(events -> experiments (experiment_id));
joinable!(groups -> days (day_id));
joinable!(tasks -> experiments (experiment_id));
joinable!(elaborations -> groups (group_id));
joinable!(elaborations -> experiments (experiment_id));
joinable!(completions -> groups (group_id));
joinable!(completions -> tasks (task_id));
joinable!(group_mappings -> students (student_id));
joinable!(group_mappings -> groups (group_id));


// This is a hack until arbitrary joins or "through associations" are allowed.
// Also see https://github.com/diesel-rs/diesel/issues/938
macro_rules! allow_join {
    ($left:ident ( $left_key:ident ) <=> $right:ident ( $right_key:ident )) => {
        allow_join!($left, $right, $left_key, $right_key);
        allow_join!($right, $left, $left_key, $right_key);
    };
    ($left:ident, $right:ident, $left_key:ident, $right_key:ident) => {
        joinable_inner!(
            left_table_ty = $left::table,
            right_table_ty = $right::table,
            right_table_expr = $right::table,
            foreign_key = $right::$right_key,
            primary_key_ty = $left::$left_key,
            primary_key_expr = $left::$left_key,
        );
    };
}

allow_join!(completions (group_id) <=> group_mappings (group_id));
allow_join!(elaborations (group_id) <=> group_mappings (group_id));
