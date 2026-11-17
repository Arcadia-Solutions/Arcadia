INSERT INTO
  title_groups (
    id, name, name_aliases, created_by_id, description, original_language,
    covers, external_links, trailers, category, content_type, screenshots
  )
VALUES
  (
    50, 'Short Link Title Group', '{}', 1, 'External link without an identifier suffix', 'English',
    '{}', '{https://example.com/1}', '{}', 'Single', 'music', '{}'
  ),
  (
    51, 'Long Link Title Group', '{}', 1, 'External link whose identifier starts with the short one', 'English',
    '{}', '{https://example.com/123}', '{}', 'Single', 'music', '{}'
  ),
  (
    52, 'Trailing Slash Link Title Group', '{}', 1, 'External link stored with a trailing slash', 'English',
    '{}', '{https://example.com/456/}', '{}', 'Single', 'music', '{}'
  );
