// @generated automatically by Diesel CLI.

diesel::table! {
    users (user_id) {
        user_id -> Int4,
        name -> Varchar,
        email -> Varchar,
        password -> Varchar,
    },
    sessions (session_id) {
        session_id -> Int4,
        user_id -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        role_id -> Int4,
    },
    sections (section_id) {
        section_id -> Int4,
        session_id -> Int4,
        user_content -> Varchar,
        assistant_content -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    },
    roles (role_id) {
        role_id -> Int4,
        role_name -> Varchar,
    }
}
