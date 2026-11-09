UPDATE torrents SET created_at = NOW() - INTERVAL '2 days' WHERE id = 900;
