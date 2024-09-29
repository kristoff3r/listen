-- Your SQL goes here
CREATE TABLE videos (
    id serial primary key NOT NULL,
    title text NOT NULL,
    youtube_id text,
    url text NOT NULL,
    file_path text,
    metadata jsonb,
    created_at timestamptz NOT NULL default now(),
    updated_at timestamptz NOT NULL default now()
);

CREATE TYPE download_status AS ENUM ('pending', 'processing', 'complete', 'failed');

CREATE TABLE downloads (
    id serial primary key NOT NULL,
    video_id integer NOT NULL,
    error text,
    status download_status NOT NULL default 'pending',
    retry_count integer NOT NULL default 0,

    created_at timestamptz NOT NULL default now(),
    updated_at timestamptz NOT NULL default now(),

    FOREIGN KEY (video_id) REFERENCES videos (id)
);

CREATE UNIQUE INDEX youtube_id_unique ON videos (youtube_id);