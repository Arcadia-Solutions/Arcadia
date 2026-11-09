UPDATE arcadia_settings SET bonus_points_per_endpoint = '[
  {"method": "GET", "path_prefix": "/api/users/me", "probability": 100, "amount": 7},
  {"method": "GET", "path_prefix": "/api/user-badges", "probability": 0, "amount": 1000}
]';
