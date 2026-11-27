-- every test user is a newbie
-- sub category 101 is hidden from newbies, except user_sub_edit (110)
INSERT INTO forum_sub_category_forbidden_classes (forum_sub_category_id, user_class_name) VALUES (101, 'newbie');
INSERT INTO forum_sub_category_allowed_users (forum_sub_category_id, user_id) VALUES (101, 110);
