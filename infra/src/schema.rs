// @generated automatically by Diesel CLI.

diesel::table! {
    timers (id) {
        id -> Text,
        active_task_id -> Nullable<Text>,
        current_phase -> Text,
        remaining_seconds -> Integer,
        is_running -> Bool,
        state -> Text,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    tasks (id) {
        id -> Text,
        name -> Text,
        description -> Nullable<Text>,
        sessions -> Integer,
        current_sessions -> Integer,
        status -> Text,
        tags -> Nullable<Text>,
        config -> Text,
        is_default -> Bool,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    config (id) {
        id -> Integer,
        config_data -> Text,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    session_history (id) {
        id -> Text,
        task_id -> Text,
        session_type -> Text,
        duration_seconds -> Integer,
        completed_at -> Text,
    }
}

diesel::joinable!(session_history -> tasks (task_id));

diesel::allow_tables_to_appear_in_same_query!(
    timers,
    tasks,
    config,
    session_history,
);
