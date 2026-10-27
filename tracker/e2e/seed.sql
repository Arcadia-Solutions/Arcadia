-- Loaded after the title group, edition group and torrent fixtures of the tracker tests.
-- Expects the psql variables info_hash (hex) and size.
-- downloaded starts at 0 instead of the default 1, so that the reported amounts can be compared as is
INSERT INTO users (username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, downloaded, real_downloaded)
VALUES
    ('e2e_seeder', 'seeder@e2e.test', 'none', '127.0.0.1', '5eede75eede75eede75eede75eede75e', 'newbie', 'arcadia', 0, 0),
    ('e2e_leecher', 'leecher@e2e.test', 'none', '127.0.0.1', '1eec4e121eec4e121eec4e121eec4e12', 'newbie', 'arcadia', 0, 0),
    ('e2e_leecher2', 'leecher2@e2e.test', 'none', '127.0.0.1', '2eec4e122eec4e122eec4e122eec4e12', 'newbie', 'arcadia', 0, 0);

-- Factors other than 100 so that credited and real amounts can be told apart
UPDATE torrents
SET info_hash = decode(:'info_hash', 'hex'), size = :size, upload_factor = 200, download_factor = 50
WHERE id = 1;
