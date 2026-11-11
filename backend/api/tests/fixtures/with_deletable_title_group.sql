-- Title group 3 with edition groups and soft-deleted torrents (deletable)
INSERT INTO title_groups (id, name, name_aliases, created_by_id, description, original_language, country_from, covers, external_links, trailers, category, content_type, public_ratings, screenshots)
VALUES (3, 'Deletable Title Group', '{}', 100, 'A title group that can be deleted', 'English', 'US', '{}', '{}', '{}', NULL, 'music', '[]', '{}');

-- Edition groups for title_group 3
INSERT INTO edition_groups (id, title_group_id, name, created_by_id, covers, external_links)
VALUES (4, 3, 'Edition 1', 100, '{}', '{}'),
       (5, 3, 'Edition 2', 100, '{}', '{}');

-- Soft-deleted torrents for title_group 3
INSERT INTO torrents (id, edition_group_id, created_by_id, info_hash, info_dict, languages, release_name, release_group, description, file_amount_per_type, uploaded_as_anonymous, file_list, mediainfo, trumpable, staff_checked, container, size, deleted_at, deleted_by_id)
VALUES (4, 4, 100, '\x1122334455667788990011223344556677889900', '{}', '{}', 'Deleted Torrent 1', '', '', '{}', FALSE, '{}', '{}', FALSE, FALSE, 'mp3', 1000000, NOW(), 100),
       (5, 4, 100, '\x2233445566778899001122334455667788990011', '{}', '{}', 'Deleted Torrent 2', '', '', '{}', FALSE, '{}', '{}', FALSE, FALSE, 'mp3', 1000000, NOW(), 100),
       (6, 5, 100, '\x3344556677889900112233445566778899001122', '{}', '{}', 'Deleted Torrent 3', '', '', '{}', FALSE, '{}', '{}', FALSE, FALSE, 'mp3', 1000000, NOW(), 100);

-- Artist 2 affiliated with title_group 3 (to test counter decrement)
-- Starts with: title_groups_amount=1, edition_groups_amount=2, torrents_amount=0 (its torrents are soft-deleted)
INSERT INTO artists (id, name, description, pictures, created_by_id, title_groups_amount, edition_groups_amount, torrents_amount, seeders_amount, leechers_amount, snatches_amount)
VALUES (2, 'Test Artist', 'An artist for testing deletion', '{}', 100, 1, 2, 0, 0, 0, 0);

INSERT INTO affiliated_artists (id, title_group_id, artist_id, roles, nickname, created_by_id)
VALUES (2, 3, 2, '{main}', NULL, 100);

-- Title group 4 with soft-deleted torrent only (also deletable, no affiliated artists)
INSERT INTO title_groups (id, name, name_aliases, created_by_id, description, original_language, country_from, covers, external_links, trailers, category, content_type, public_ratings, screenshots)
VALUES (4, 'Title Group With Deleted Torrent', '{}', 100, 'Has only soft-deleted torrents', 'English', 'US', '{}', '{}', '{}', NULL, 'music', '[]', '{}');

INSERT INTO edition_groups (id, title_group_id, name, created_by_id, covers, external_links)
VALUES (3, 4, 'Deleted Edition', 100, '{}', '{}');

INSERT INTO torrents (id, edition_group_id, created_by_id, info_hash, info_dict, languages, release_name, release_group, description, file_amount_per_type, uploaded_as_anonymous, file_list, mediainfo, trumpable, staff_checked, container, size, deleted_at, deleted_by_id)
VALUES (3, 3, 100, '\xaabbccddeeff00112233445566778899aabbccdd', '{}', '{}', 'Deleted Torrent', '', '', '{}', FALSE, '{}', '{}', FALSE, FALSE, 'mp3', 1000000, NOW(), 100);

-- Update user 100's counters to reflect the title groups, edition groups and torrents created
-- Title group 3: 2 edition groups, 3 torrents
-- Title group 4: 1 edition group, 1 torrent
-- Total: 2 title groups, 3 edition groups, 4 torrents
UPDATE users SET title_groups = 2, edition_groups = 3, torrents = 4 WHERE id = 100;
