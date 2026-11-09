-- auto_newbie -> auto_member -> auto_elite, members need 1000 bytes uploaded and elites 5000
INSERT INTO user_classes (name, previous_user_class, required_uploaded)
VALUES ('auto_newbie', NULL, 0),
       ('auto_member', 'auto_newbie', 1000),
       ('auto_elite', 'auto_member', 5000);
-- never reached automatically even though it has no requirement
INSERT INTO user_classes (name, previous_user_class, automatic_promotion)
VALUES ('invite_only', 'auto_newbie', FALSE);
-- never left automatically even when its requirements stop being met
INSERT INTO user_classes (name, previous_user_class, required_uploaded, automatic_demotion)
VALUES ('sticky_member', 'auto_newbie', 1000, FALSE);

INSERT INTO users (id, username, email, password_hash, passkey, class_name, uploaded, warned, registered_from_ip, css_sheet_name)
VALUES (2000, 'promoted_user', 'promoted@example.com', 'hash', 'passkey2000', 'auto_newbie', 10000, FALSE, '127.0.0.1', 'arcadia'),
       (2001, 'unqualified', 'unqualified@example.com', 'hash', 'passkey2001', 'auto_newbie', 0, FALSE, '127.0.0.1', 'arcadia'),
       (2002, 'demoted_user', 'demoted@example.com', 'hash', 'passkey2002', 'auto_member', 0, FALSE, '127.0.0.1', 'arcadia'),
       (2003, 'warned_user', 'warned@example.com', 'hash', 'passkey2003', 'auto_newbie', 10000, TRUE, '127.0.0.1', 'arcadia'),
       (2004, 'sticky_user', 'sticky@example.com', 'hash', 'passkey2004', 'sticky_member', 0, FALSE, '127.0.0.1', 'arcadia');
