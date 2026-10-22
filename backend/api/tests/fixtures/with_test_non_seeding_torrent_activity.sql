-- User 102 grabbed torrent 1 without completing it, and has no peer on it
INSERT INTO
  torrent_activities (torrent_id, user_id, grabbed_at, completed_at)
VALUES
  (1, 102, NOW(), NULL);
