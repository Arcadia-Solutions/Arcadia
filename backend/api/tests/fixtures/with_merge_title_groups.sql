-- Source title group (id=10) and target title group (id=11), both music
-- Title group with different content type (id=12) for mismatch test
INSERT INTO title_groups (id, name, name_aliases, created_at, updated_at, created_by_id, description, covers, external_links, trailers, content_type, public_ratings, screenshots)
VALUES
  (10, 'Source Title Group', '{}', NOW(), NOW(), 1, 'Source description', '{}', '{https://source-only.example.com,https://shared.example.com}', '{}', 'music', '[]', '{}'),
  (11, 'Target Title Group', '{}', NOW(), NOW(), 1, 'Target description', '{}', '{https://target-only.example.com,https://shared.example.com}', '{}', 'music', '[]', '{}'),
  (12, 'Different Content Type Group', '{}', NOW(), NOW(), 1, 'Software description', '{}', '{}', '{}', 'software', '[]', '{}');

-- Edition groups: one on source, one on target
INSERT INTO edition_groups (id, title_group_id, name, release_date, release_date_only_year_known, created_at, updated_at, created_by_id, description, distributor, covers, external_links, source, additional_information)
VALUES
  (10, 10, 'Source Edition', '2020-01-01', false, NOW(), NOW(), 1, 'Source edition desc', '', '{}', '{}', 'Web', '{}'),
  (11, 11, 'Target Edition', '2020-01-01', false, NOW(), NOW(), 1, 'Target edition desc', '', '{}', '{}', 'Web', '{}');

-- Torrents: one per edition group
INSERT INTO torrents (id, edition_group_id, created_at, updated_at, created_by_id, info_hash, info_dict, languages, release_name, release_group, description, file_amount_per_type, uploaded_as_anonymous, file_list, mediainfo, trumpable, staff_checked, container, size, audio_codec, audio_bitrate, audio_bitrate_sampling, features, subtitle_languages)
VALUES
  (10, 10, NOW(), NOW(), 1, '\xaa11223344556677889900aabbccddeeff112233', '{}', '{}', 'Source Torrent', '', 'Source torrent desc', '{}', false, '{}', '{}', false, false, 'FLAC', 100000, 'flac', 320, 'Lossless', '{}', '{}'),
  (11, 11, NOW(), NOW(), 1, '\xbb11223344556677889900aabbccddeeff112233', '{}', '{}', 'Target Torrent', '', 'Target torrent desc', '{}', false, '{}', '{}', false, false, 'FLAC', 200000, 'flac', 320, 'Lossless', '{}', '{}');

-- Artists
INSERT INTO artists (id, name, description, pictures, created_by_id, created_at, title_groups_amount, edition_groups_amount, torrents_amount, seeders_amount, leechers_amount, snatches_amount)
VALUES
  (10, 'Artist A', '', '{}', 1, NOW(), 0, 0, 0, 0, 0, 0),
  (11, 'Artist B', '', '{}', 1, NOW(), 0, 0, 0, 0, 0, 0),
  (12, 'Artist Shared', '', '{}', 1, NOW(), 0, 0, 0, 0, 0, 0);

-- Affiliated artists: Artist A only on source, Artist B only on target, Artist Shared on both (conflict)
INSERT INTO affiliated_artists (title_group_id, artist_id, roles, created_by_id, created_at)
VALUES
  (10, 10, '{main}', 1, NOW()),
  (10, 12, '{main}', 1, NOW()),
  (11, 11, '{main}', 1, NOW()),
  (11, 12, '{guest}', 1, NOW());

-- Tags
INSERT INTO title_group_tags (id, name, synonyms, created_by_id)
VALUES
  (10, 'merge_tag_a', '{}', 1),
  (11, 'merge_tag_b', '{}', 1),
  (12, 'merge_tag_shared', '{}', 1);

-- Applied tags: tag_a only on source, tag_b only on target, tag_shared on both (conflict)
INSERT INTO title_group_applied_tags (title_group_id, tag_id, created_by_id)
VALUES
  (10, 10, 1),
  (10, 12, 1),
  (11, 11, 1),
  (11, 12, 1);

-- Comments: two on source, one on target
INSERT INTO title_group_comments (id, content, created_at, updated_at, created_by_id, title_group_id, locked)
VALUES
  (10, 'Source comment 1', NOW(), NOW(), 1, 10, false),
  (11, 'Source comment 2', NOW(), NOW(), 1, 10, false),
  (12, 'Target comment', NOW(), NOW(), 1, 11, false);

-- Bookmarks: user 100 bookmarks source, user 146 bookmarks both (conflict)
INSERT INTO title_group_bookmarks (id, user_id, title_group_id, description)
VALUES
  (10, 100, 10, 'Source bookmark'),
  (11, 146, 10, 'Source bookmark by merge user'),
  (12, 146, 11, 'Target bookmark by merge user');

-- Subscriptions (title_group_torrents): user 100 on source, user 146 on both (conflict)
INSERT INTO subscriptions_title_group_torrents (user_id, title_group_id)
VALUES
  (100, 10),
  (146, 10),
  (146, 11);

-- Subscriptions (title_group_comments): user 100 on source, user 146 on both (conflict)
INSERT INTO subscriptions_title_group_comments (user_id, title_group_id)
VALUES
  (100, 10),
  (146, 10),
  (146, 11);

-- Collages and entries: collage 10 has both source and target (conflict), collage 11 has only source
INSERT INTO collage (id, created_by_id, name, cover, description, tags, category)
VALUES
  (10, 1, 'Test Merge Collage A', NULL, 'Collage with both', '{}', 'Personal'),
  (11, 1, 'Test Merge Collage B', NULL, 'Collage with source only', '{}', 'Personal');

INSERT INTO collage_entry (created_by_id, collage_id, title_group_id, note)
VALUES
  (1, 10, 10, 'source entry in collage A'),
  (1, 10, 11, 'target entry in collage A'),
  (1, 11, 10, 'source entry in collage B');

-- Torrent request on source
INSERT INTO torrent_requests (id, title_group_id, created_at, updated_at, created_by_id, description, languages, container, audio_codec, audio_channels, audio_bitrate_sampling, video_codec, features, subtitle_languages, video_resolution, source)
VALUES (10, 10, NOW(), NOW(), 1, 'Request on source', ARRAY['English']::language_enum[], ARRAY['FLAC'], ARRAY['flac']::audio_codec_enum[], ARRAY['2.0']::audio_channels_enum[], ARRAY['Lossless']::audio_bitrate_sampling_enum[], ARRAY[]::video_codec_enum[], ARRAY[]::features_enum[], ARRAY[]::language_enum[], ARRAY[]::video_resolution_enum[], ARRAY['CD']::source_enum[]);

-- Entities: entity 10 only on source, entity 11 on both (conflict, target is label)
INSERT INTO entities (id, name, description, pictures, created_by_id)
VALUES
  (10, 'Entity A', '', '{}', 1),
  (11, 'Entity Shared', '', '{}', 1);

INSERT INTO affiliated_entities (title_group_id, entity_id, roles, created_by_id)
VALUES
  (10, 10, '{producer}', 1),
  (10, 11, '{producer}', 1),
  (11, 11, '{label}', 1);

-- Notifications on source for user 100
INSERT INTO notifications_title_group_comments (title_group_id, title_group_comment_id, user_id)
VALUES (10, 10, 100);

INSERT INTO notifications_artist_title_groups (artist_id, title_group_id, user_id)
VALUES (10, 10, 100);
