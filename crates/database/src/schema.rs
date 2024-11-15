// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "download_status"))]
    pub struct DownloadStatus;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::DownloadStatus;

    downloads (download_id) {
        download_id -> Uuid,
        video_id -> Uuid,
        error -> Nullable<Text>,
        status -> DownloadStatus,
        retry_count -> Int4,
        force -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    oidc_mapping (oidc_mapping_id) {
        oidc_mapping_id -> Uuid,
        #[max_length = 255]
        oidc_issuer_url -> Varchar,
        #[max_length = 255]
        oidc_issuer_id -> Varchar,
        user_id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    user_sessions (user_session_id) {
        user_session_id -> Uuid,
        #[max_length = 255]
        oidc_issuer_url -> Nullable<Varchar>,
        #[max_length = 255]
        csrf_token -> Nullable<Varchar>,
        #[max_length = 255]
        nonce -> Nullable<Varchar>,
        #[max_length = 255]
        pkce_code_verifier -> Nullable<Varchar>,
        user_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    users (user_id) {
        user_id -> Uuid,
        last_login -> Timestamptz,
        last_activity -> Timestamptz,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        handle -> Varchar,
        profile_picture_url -> Text,
        is_approved -> Bool,
        is_admin -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    videos (video_id) {
        video_id -> Uuid,
        title -> Text,
        youtube_id -> Nullable<Text>,
        url -> Text,
        file_path -> Text,
        metadata -> Nullable<Jsonb>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(downloads -> videos (video_id));
diesel::joinable!(oidc_mapping -> users (user_id));
diesel::joinable!(user_sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    downloads,
    oidc_mapping,
    user_sessions,
    users,
    videos,
);
