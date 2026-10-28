-- Update torrents with known size and seeders for predictable formula results
-- Torrent 1: size=100MB, seeders=2
-- Torrent 2: size=200MB, seeders=1
-- only torrents checked by staff reward bonus points
UPDATE torrents SET size = 100000000, seeders = 2, staff_checked = true WHERE id = 1;
UPDATE torrents SET size = 200000000, seeders = 1, staff_checked = true WHERE id = 2;

-- Torrent activities for bonus points test
-- User 100 seeds torrent 1 (200 sec) and torrent 2 (300 sec)
-- User 101 seeds torrent 1 (400 sec) and torrent 2 (500 sec)
INSERT INTO torrent_activities (torrent_id, user_id, total_seed_time, bonus_points)
VALUES
    (1, 100, 200, 0),
    (2, 100, 300, 0),
    (1, 101, 400, 0),
    (2, 101, 500, 0);

-- Active seeder peers for bonus points calculation
INSERT INTO peers (peer_id, ip, port, agent, uploaded, downloaded, "left", seeder, torrent_id, user_id, active)
VALUES
    (E'\\xb001000000000000000000000000000000000001', '10.0.0.1', 6881, 'test', 0, 0, 0, true, 1, 100, true),
    (E'\\xb001000000000000000000000000000000000002', '10.0.0.2', 6881, 'test', 0, 0, 0, true, 2, 100, true),
    (E'\\xb001000000000000000000000000000000000003', '10.0.0.3', 6881, 'test', 0, 0, 0, true, 1, 101, true),
    (E'\\xb001000000000000000000000000000000000004', '10.0.0.4', 6881, 'test', 0, 0, 0, true, 2, 101, true);

-- User 100 seeds torrent 1 from two additional clients: the reward must not be counted multiple times
INSERT INTO peers (peer_id, ip, port, agent, uploaded, downloaded, "left", seeder, torrent_id, user_id, active)
VALUES
    (E'\\xb001000000000000000000000000000000000005', '10.0.0.5', 6881, 'test', 0, 0, 0, true, 1, 100, true),
    (E'\\xb001000000000000000000000000000000000006', '10.0.0.6', 6881, 'test', 0, 0, 0, true, 1, 100, true);
