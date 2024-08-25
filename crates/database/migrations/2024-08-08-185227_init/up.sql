-- Your SQL goes here
CREATE TABLE videos (
    id serial primary key NOT NULL,
    title text NOT NULL,
    youtube_id text,
    created_at timestamptz NOT NULL default now(),
    updated_at timestamptz NOT NULL default now()
);