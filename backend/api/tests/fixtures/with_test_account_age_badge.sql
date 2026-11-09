-- user 101 registered 400 days ago, the other users just registered
UPDATE users SET created_at = NOW() - INTERVAL '400 days' WHERE id = 101;

INSERT INTO user_badge_categories (id, name, created_by_id)
VALUES (950, 'account_age_category', 1);

INSERT INTO user_badges (id, name, description, image_url, category_id, badge_type, criteria, created_by_id)
VALUES (
    950,
    'one_year_old',
    'account older than a year',
    'https://example.com/year.png',
    950,
    'account_age',
    '{"type":"account_age","minimum_account_age_in_days":365}'::jsonb,
    1
);
