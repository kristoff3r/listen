-- Your SQL goes here

CREATE TYPE download_status AS ENUM ('pending', 'processing', 'complete', 'failed');

ALTER TABLE videos
    ADD COLUMN metadata jsonb,
    ADD COLUMN status download_status NOT NULL default 'pending';

CREATE TABLE downloads (
    id serial primary key NOT NULL,
    video_id integer NOT NULL,
    url text NOT NULL,
    error text,
    created_at timestamptz NOT NULL default now(),
    updated_at timestamptz NOT NULL default now(),
    FOREIGN KEY (video_id) REFERENCES videos (id)
);

CREATE UNIQUE INDEX youtube_id_unique ON videos (youtube_id);