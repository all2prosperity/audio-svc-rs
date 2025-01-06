// @generated automatically by Diesel CLI.

diesel::table! {
    roles (role_id) {
        role_id -> Varchar,
        prompt -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    sections (section_id) {
        section_id -> Varchar,
        session_id -> Varchar,
        user_message -> Text,
        assistant_message -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    sessions (session_id) {
        session_id -> Varchar,
        xid -> Varchar,
        role_id -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    users (xid) {
        xid -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    roles,
    sections,
    sessions,
    users,
);
