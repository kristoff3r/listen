BEGIN;

delete from videos;

insert into videos (title, youtube_id, url, file_path) values
    ( 'Shatner Of The Mount by Fall On Your Sword', 'HU2ftCitvyQ', '', 'HU2ftCitvyQ.mp4'),
    ( 'Kjell Höglund Maskinerna är våra vänner', 'sZHL2EEXdiY', '', 'sZHL2EEXdiY.mp4'),
    ( 'Disco Snails | Vulfmon & Zachary Barker', 'oAZBKlVSLkU', '', 'oAZBKlVSLkU.mp4');

COMMIT;
