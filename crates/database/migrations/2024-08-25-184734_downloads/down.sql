-- This file should undo anything in `up.sql`

ALTER TABLE videos
    DROP COLUMN metadata,
    DROP COLUMN status;

DROP TYPE download_status;

DROP TABLE downloads;

DROP INDEX youtube_id_unique;