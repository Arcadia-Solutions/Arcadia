-- calendar 900 with 2 doors, user 100 opened both, user 101 opened one
INSERT INTO advent_calendars (id, created_by_id, name, starts_at, button_text)
VALUES (900, 1, 'badge_calendar', NOW(), 'Open');

INSERT INTO advent_calendar_doors (id, calendar_id, position, name, duration_minutes)
VALUES (900, 900, 0, 'Day 1', 1440), (901, 900, 1, 'Day 2', 1440);

INSERT INTO advent_calendar_door_openings (user_id, door_id, upload, download_removed, freeleech_tokens, invitations, bonus_points)
VALUES (100, 900, 0, 0, 0, 0, 0), (100, 901, 0, 0, 0, 0, 0), (101, 900, 0, 0, 0, 0, 0);

INSERT INTO user_badge_categories (id, name, created_by_id)
VALUES (900, 'advent_calendar_category', 1);

INSERT INTO user_badges (id, name, description, image_url, category_id, badge_type, criteria, created_by_id)
VALUES (
    900,
    'all_doors_opened',
    'opened every door of the calendar',
    'https://example.com/calendar.png',
    900,
    'advent_calendar_doors',
    '{"type":"advent_calendar_doors","calendar_id":900,"minimum_opened_door_amount":2}'::jsonb,
    1
), (
    901,
    'day_2_opened',
    'opened the door named Day 2',
    'https://example.com/day2.png',
    900,
    'advent_calendar_doors',
    '{"type":"advent_calendar_doors","calendar_id":900,"minimum_opened_door_amount":1,"door_names":["Day 2"]}'::jsonb,
    1
);
