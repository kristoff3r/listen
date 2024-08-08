// @generated automatically by Diesel CLI.

diesel::table! {
    videos (id) {
        id -> Int4,
        title -> Text,
        youtube_id -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}
