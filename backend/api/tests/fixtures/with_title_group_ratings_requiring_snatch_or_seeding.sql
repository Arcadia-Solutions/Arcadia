UPDATE arcadia_settings SET title_group_rating_requires_snatch_or_seeding = TRUE;
-- the basic user (id 100) snatched the torrent of title group 1, but none of title group 2
INSERT INTO torrent_activities (torrent_id, user_id, grabbed_at, completed_at)
VALUES (1, 100, NOW(), NOW());
