INSERT INTO title_group_tags (id, name, synonyms, created_by_id)
VALUES (2, 'drama', '{}', 1);

INSERT INTO title_group_applied_tags (title_group_id, tag_id, created_by_id)
VALUES (1, 2, 1);

INSERT INTO title_groups (id, name, name_aliases, created_by_id, description, original_language, country_from, covers, external_links, trailers, category, content_type, public_ratings, screenshots)
VALUES (3, 'Title Group Created By User 100', '{}', 100, '', 'English', 'US', '{}', '{}', '{}', NULL, 'music', '[]', '{}');

INSERT INTO title_group_applied_tags (title_group_id, tag_id, created_by_id)
VALUES (3, 2, 100);
