// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "download_status"))]
    pub struct DownloadStatus;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::DownloadStatus;

    downloads (id) {
        id -> Int4,
        video_id -> Int4,
        error -> Nullable<Text>,
        status -> DownloadStatus,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    videos (id) {
        id -> Int4,
        title -> Text,
        youtube_id -> Nullable<Text>,
        url -> Text,
        file_path -> Nullable<Text>,
        metadata -> Nullable<Jsonb>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(downloads -> videos (video_id));

diesel::allow_tables_to_appear_in_same_query!(
    downloads,
    videos,
);
