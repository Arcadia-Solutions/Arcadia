-- One catalog of each kind the title group stats are computed for: an artist, a series and a
-- collage. Every title group below is attached to some of them, and title group 24 to none of them,
-- so it must never be counted.
INSERT INTO artists (id, name, description, pictures, created_by_id, created_at)
VALUES (20, 'Stats Artist', 'An artist used for the title group stats', '{}', 100, NOW());

INSERT INTO series (id, name, description, tags, covers, banners, created_by_id, created_at, updated_at)
VALUES (20, 'Stats Series', 'A series used for the title group stats', '{}', '{}', '{}', 100, NOW(), NOW());

INSERT INTO collage (id, created_by_id, name, cover, description, tags, category)
VALUES (20, 100, 'Stats Collage', NULL, 'A collage used for the title group stats', '{}', 'Personal');

INSERT INTO title_groups (
    id, master_group_id, name, name_aliases, created_at, updated_at, created_by_id,
    description, platform, original_language, original_release_date,
    original_release_date_only_year_known, tagline, country_from,
    covers, external_links, trailers, category, content_type,
    series_id, screenshots
) VALUES
(
    20, NULL, 'Stats Series Movie', '{}', '2025-01-01 00:00:00', '2025-01-01 00:00:00', 100,
    'In the series and affiliated to the artist', NULL, 'English', '2020-01-01',
    FALSE, NULL, 'US',
    '{}', '{}', '{}', 'FeatureFilm', 'movie', 20, '{}'
),
(
    21, NULL, 'Stats Series Album', '{}', '2025-01-01 00:00:00', '2025-01-01 00:00:00', 100,
    'In the series and affiliated to the artist', NULL, 'English', '2021-06-01',
    FALSE, NULL, 'US',
    '{}', '{}', '{}', 'Album', 'music', 20, '{}'
),
(
    22, NULL, 'Stats Collage Movie', '{}', '2025-01-01 00:00:00', '2025-01-01 00:00:00', 100,
    'In the collage and affiliated to the artist, without release date', NULL, 'English', NULL,
    FALSE, NULL, 'US',
    '{}', '{}', '{}', 'FeatureFilm', 'movie', NULL, '{}'
),
(
    23, NULL, 'Stats Collage Book', '{}', '2025-01-01 00:00:00', '2025-01-01 00:00:00', 100,
    'Only in the collage', NULL, 'English', '1999-01-01',
    FALSE, NULL, 'US',
    '{}', '{}', '{}', 'Book', 'book', NULL, '{}'
),
(
    24, NULL, 'Stats Unrelated Movie', '{}', '2025-01-01 00:00:00', '2025-01-01 00:00:00', 100,
    'Attached to no catalog', NULL, 'English', '2018-01-01',
    FALSE, NULL, 'US',
    '{}', '{}', '{}', 'FeatureFilm', 'movie', NULL, '{}'
);

-- Title group 20 has two Blu-Ray edition groups on purpose: the source of a title group must be
-- counted only once however many edition groups share it.
INSERT INTO edition_groups (
    id, title_group_id, name, release_date, release_date_only_year_known,
    created_at, updated_at, created_by_id, description, distributor,
    covers, external_links, source, additional_information
) VALUES
(20, 20, 'Stats Series Movie Blu-Ray', '2020-01-01', FALSE, '2025-01-01 00:00:00', '2025-01-01 00:00:00', 100, '', '', '{}', '{}', 'Blu-Ray', '{}'),
(21, 20, 'Stats Series Movie Blu-Ray Collector', '2020-01-01', FALSE, '2025-01-01 00:00:00', '2025-01-01 00:00:00', 100, '', '', '{}', '{}', 'Blu-Ray', '{}'),
(22, 20, 'Stats Series Movie Vinyl', '2020-01-01', FALSE, '2025-01-01 00:00:00', '2025-01-01 00:00:00', 100, '', '', '{}', '{}', 'Vinyl', '{}'),
(23, 21, 'Stats Series Album CD', '2021-06-01', FALSE, '2025-01-01 00:00:00', '2025-01-01 00:00:00', 100, '', '', '{}', '{}', 'CD', '{}'),
(24, 22, 'Stats Collage Movie Web', NULL, FALSE, '2025-01-01 00:00:00', '2025-01-01 00:00:00', 100, '', '', '{}', '{}', 'Web', '{}'),
-- A source of NULL is never counted
(25, 23, 'Stats Collage Book Unknown Source', NULL, FALSE, '2025-01-01 00:00:00', '2025-01-01 00:00:00', 100, '', '', '{}', '{}', NULL, '{}');

INSERT INTO affiliated_artists (title_group_id, artist_id, roles, created_by_id)
VALUES (20, 20, '{main}', 100),
       (21, 20, '{main}', 100),
       (22, 20, '{director}', 100);

INSERT INTO collage_entry (collage_id, title_group_id, created_by_id)
VALUES (20, 22, 100),
       (20, 23, 100);
