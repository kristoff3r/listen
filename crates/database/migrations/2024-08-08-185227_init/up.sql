-- Your SQL goes here
CREATE TABLE videos (
    video_id uuid PRIMARY KEY default gen_random_uuid(),
    title text NOT NULL,
    youtube_id text,
    url text NOT NULL,
    file_path text NOT NULL,
    metadata jsonb,

    created_at timestamptz NOT NULL default now(),
    updated_at timestamptz NOT NULL default now()
);

CREATE TYPE download_status AS ENUM ('pending', 'processing', 'complete', 'failed');

CREATE TABLE downloads (
    download_id uuid PRIMARY KEY default gen_random_uuid(),
    video_id uuid NOT NULL,
    error text,
    status download_status NOT NULL default 'pending',
    retry_count integer NOT NULL default 0,
    force boolean NOT NULL default false,

    created_at timestamptz NOT NULL default now(),
    updated_at timestamptz NOT NULL default now(),

    FOREIGN KEY (video_id) REFERENCES videos (video_id)
);

CREATE UNIQUE INDEX youtube_id_unique ON videos (youtube_id);
