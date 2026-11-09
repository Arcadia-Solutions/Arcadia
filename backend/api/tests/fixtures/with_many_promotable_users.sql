-- more users than the 100 processed per batch
INSERT INTO user_classes (name, previous_user_class, required_uploaded)
VALUES ('batch_newbie', NULL, 0),
       ('batch_member', 'batch_newbie', 1000);

INSERT INTO users (id, username, email, password_hash, passkey, class_name, uploaded, registered_from_ip, css_sheet_name)
SELECT id, 'batch_user_' || id, 'batch_' || id || '@example.com', 'hash', md5(id::text), 'batch_newbie', 1000, '127.0.0.1', 'arcadia'
FROM generate_series(3000, 3249) AS id;
