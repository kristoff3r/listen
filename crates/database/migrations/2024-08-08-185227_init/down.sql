-- This file should undo anything in `up.sql`
DROP TABLE downloads;
DROP TABLE videos;

DROP TYPE download_status;

DROP INDEX youtube_id_unique;