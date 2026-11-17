-- Master group
INSERT INTO master_groups (id, name, created_at, updated_at, created_by_id)
VALUES (1, 'Beatles Discography', NOW(), NOW(), 1);

-- Link title_group 1 to the master group and to series 1
UPDATE title_groups SET master_group_id = 1, series_id = 1 WHERE id = 1;

-- Second title group in the same master group
INSERT INTO title_groups (
    id, master_group_id, name, name_aliases, created_by_id,
    description, content_type, covers, external_links, trailers, screenshots, original_release_date
)
VALUES (
    3, 1, 'Please Please Me', '{}', 1,
    'The debut album by The Beatles',
    'music', '{}', '{}', '{}', '{}',
    '1963-03-22'
);

-- Second artist
INSERT INTO artists (id, name, description, pictures, created_by_id, created_at)
VALUES (2, 'George Martin', 'English record producer', '{}', 1, NOW());

-- Two affiliated artists on title_group 1
INSERT INTO affiliated_artists (id, title_group_id, artist_id, roles, nickname, created_by_id, created_at)
VALUES (2, 1, 2, '{guest}', 'Sir George', 1, NOW());

-- Torrent report on torrent 1
INSERT INTO torrent_reports (id, reported_by_id, reported_torrent_id, description)
VALUES (1, 100, 1, 'Possible trump: better source available');

-- Torrent activity: user 100 snatched torrent 1
INSERT INTO torrent_activities (torrent_id, user_id, grabbed_at, completed_at)
VALUES (1, 100, NOW() - INTERVAL '2 days', NOW() - INTERVAL '1 day');

-- Subscriptions for user 100 on title_group 1
INSERT INTO subscriptions_title_group_torrents (user_id, title_group_id) VALUES (100, 1);
INSERT INTO subscriptions_title_group_comments (user_id, title_group_id) VALUES (100, 1);
