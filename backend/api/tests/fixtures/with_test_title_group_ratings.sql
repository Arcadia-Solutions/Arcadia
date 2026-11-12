-- title group 1 is rated 80 and title group 2 is rated 40 by the basic user (id 100)
INSERT INTO title_group_ratings (title_group_id, user_id, rating, comment)
VALUES (1, 100, 80, 'Great'), (2, 100, 40, NULL);
UPDATE title_groups SET user_rating_average = 80, user_rating_count = 1 WHERE id = 1;
UPDATE title_groups SET user_rating_average = 40, user_rating_count = 1 WHERE id = 2;
UPDATE users SET title_group_ratings = 2 WHERE id = 100;
