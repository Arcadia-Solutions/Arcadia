-- Torrent request created by user_basic (100), filled by user_edit_art (101) with the torrent of
-- user 1, who each received half of the bounty
INSERT INTO torrent_requests (id, title_group_id, created_by_id, description, filled_by_user_id, filled_by_torrent_id, filled_at)
VALUES (5, 1, 100, 'Request created by the basic user and filled with a torrent', 101, 1, NOW());

INSERT INTO torrent_request_votes (id, torrent_request_id, created_by_id, bounty_upload, bounty_bonus_points)
VALUES (6, 5, 100, 1000, 50);

UPDATE users SET uploaded = uploaded + 500, bonus_points = bonus_points + 25 WHERE id IN (1, 101);
UPDATE users SET requests_filled = 1 WHERE id = 101;

SELECT setval('torrent_requests_id_seq', 5);
SELECT setval('torrent_request_votes_id_seq', 6);
