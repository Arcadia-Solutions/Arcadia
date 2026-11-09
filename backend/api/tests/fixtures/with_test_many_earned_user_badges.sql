-- 10 badges earned by user 100, badge 709 being the most recent one
INSERT INTO user_badge_categories (id, name, created_by_id)
VALUES (701, 'many_badges_category', 100);

INSERT INTO user_badges (id, name, description, image_url, category_id, badge_type, created_by_id)
SELECT 700 + n, 'many_badges_' || n, 'badge ' || n, 'https://example.com/' || n || '.png', 701, 'manual', 100
FROM generate_series(0, 9) AS n;

INSERT INTO user_earned_badges (user_id, badge_id, awarded_at)
SELECT 100, 700 + n, NOW() - (10 - n) * INTERVAL '1 day'
FROM generate_series(0, 9) AS n;
