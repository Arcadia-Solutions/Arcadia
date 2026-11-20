-- door 910 opened by user 100 (5 bonus points) and user 199 (10 bonus points)
INSERT INTO advent_calendars (id, created_by_id, name, starts_at, button_text)
VALUES (910, 1, 'stats_calendar', NOW(), 'Open');

INSERT INTO advent_calendar_doors (id, calendar_id, position, name, duration_minutes)
VALUES (910, 910, 0, 'Day 1', 1440);

INSERT INTO advent_calendar_door_openings (user_id, door_id, upload, download_removed, freeleech_tokens, invitations, bonus_points)
VALUES (100, 910, 0, 0, 0, 0, 5), (199, 910, 0, 0, 0, 0, 10);
