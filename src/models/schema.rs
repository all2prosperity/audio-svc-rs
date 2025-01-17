// @generated automatically by Diesel CLI.

diesel::table! {
    roles (id) {
        id -> Varchar,
        created_by -> Varchar,
        is_default -> Bool,
        name -> Text,
        picture_url -> Text,
        voice_id -> Text,
        audition_url -> Text,
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
        user_id -> Varchar,
        role_id -> Varchar,
        title -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    user_role (id) {
        id -> Varchar,
        role_id -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    roles,
    sections,
    sessions,
    user_role,
    users,
);
