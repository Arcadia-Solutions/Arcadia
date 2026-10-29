-- Collage owned by user_basic (id=100)
INSERT INTO collage (id, created_by_id, name, cover, description, tags, category)
VALUES (1, 100, 'Test Collage', NULL, 'A test collage', '{"test","sample"}', 'Personal');

-- Collage owned by another user for permission tests
INSERT INTO collage (id, created_by_id, name, cover, description, tags, category)
VALUES (2, 101, 'Other User Collage', NULL, 'Another test collage', '{"other"}', 'Theme');

-- Empty collage for deletion tests (no entries)
INSERT INTO collage (id, created_by_id, name, cover, description, tags, category)
VALUES (3, 100, 'Empty Collage', NULL, 'An empty collage for deletion tests', '{"empty"}', 'Personal');

-- Personal collage owned by another user for permission tests
INSERT INTO collage (id, created_by_id, name, cover, description, tags, category)
VALUES (4, 101, 'Other User Personal Collage', NULL, 'Another personal collage', '{"other"}', 'Personal');

-- Staff picks collage owned by another user for permission tests
INSERT INTO collage (id, created_by_id, name, cover, description, tags, category)
VALUES (5, 101, 'Other User Staff Picks', NULL, 'Staff picks collage', '{"other"}', 'Staff Picks');
