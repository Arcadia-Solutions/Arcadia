-- Two title groups (id=20, id=21) to be linked as similar
INSERT INTO title_groups (id, name, name_aliases, created_at, updated_at, created_by_id, description, original_release_date, covers, external_links, trailers, content_type, screenshots)
VALUES
  (20, 'First Similar Title Group', '{}', NOW(), NOW(), 1, 'First description', '2001-01-01', '{https://cover-20-a.example.com,https://cover-20-b.example.com}', '{}', '{}', 'movie', '{}'),
  (21, 'Second Similar Title Group', '{}', NOW(), NOW(), 1, 'Second description', '2002-02-02', '{https://cover-21-a.example.com}', '{}', '{}', 'movie', '{}');
