// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "download_status"))]
    pub struct DownloadStatus;
}

diesel::table! {
    downloads (id) {
        id -> Int4,
        video_id -> Int4,
        url -> Text,
        error -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::DownloadStatus;

    videos (id) {
        id -> Int4,
        title -> Text,
        youtube_id -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        metadata -> Nullable<Jsonb>,
        status -> DownloadStatus,
    }
}

diesel::joinable!(downloads -> videos (video_id));

diesel::allow_tables_to_appear_in_same_query!(downloads, videos,);
