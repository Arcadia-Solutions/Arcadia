INSERT INTO user_badge_categories (id, name, created_by_id)
VALUES (960, 'ratings_category', 1);

-- user 100 rated title group 1 ("Love Me Do") 80 and title group 2 40 (with_test_title_group_ratings)
INSERT INTO user_badges (id, name, description, image_url, category_id, badge_type, criteria, created_by_id)
VALUES
(
    960,
    'high_rater',
    'rated a title group 70 or more',
    'https://example.com/high.png',
    960,
    'title_group_ratings',
    '{"type":"title_group_ratings","search":{"title_group_include_empty_groups":true,"page":1,"page_size":1,"order_by_column":"torrent_created_at","order_by_direction":"desc"},"minimum_rating":70,"maximum_rating":100,"minimum_rating_amount":1}'::jsonb,
    1
),
(
    961,
    'two_love_ratings',
    'rated 2 title groups with "Love" in the name',
    'https://example.com/love.png',
    960,
    'title_group_ratings',
    '{"type":"title_group_ratings","search":{"title_group_name":"Love","title_group_include_empty_groups":true,"page":1,"page_size":1,"order_by_column":"torrent_created_at","order_by_direction":"desc"},"minimum_rating":0,"maximum_rating":100,"minimum_rating_amount":2}'::jsonb,
    1
);
