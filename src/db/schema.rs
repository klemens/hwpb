infer_schema!("dotenv:DATABASE_URL");

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
