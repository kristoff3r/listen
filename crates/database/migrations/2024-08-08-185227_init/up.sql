-- Your SQL goes here
CREATE TABLE videos (
    id serial primary key NOT NULL,
    title text NOT NULL,
    youtube_id text,
    created_at timestamptz NOT NULL default now(),
    updated_at timestamptz NOT NULL default now()
);

-- CREATE TABLE users (
--     id serial primary key NOT NULL,
--     password character varying(128) NOT NULL,
--     last_login timestamp with time zone,
--     username character varying(150) NOT NULL,
--     first_name character varying(30) NOT NULL,
--     last_name character varying(150) NOT NULL,
--     email character varying(254) NOT NULL,
--     is_active boolean NOT NULL,
--     date_joined timestamp with time zone NOT NULL,
--     discord_id character varying(64) NOT NULL,
--     avatar character varying(1024),
--     last_changed_cell timestamp with time zone NOT NULL
-- );
