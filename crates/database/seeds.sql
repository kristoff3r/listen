BEGIN;

delete from videos;

insert into videos (title, youtube_id) values
    ( 'Kjell Höglund Maskinerna är våra vänner', 'sZHL2EEXdiY' ),
    ( 'Disco Snails | Vulfmon & Zachary Barker', 'oAZBKlVSLkU' );

COMMIT;