ALTER TABLE settings
ADD COLUMN page_background_path TEXT NOT NULL DEFAULT '';

ALTER TABLE settings
ADD COLUMN page_background_opacity REAL NOT NULL DEFAULT 0.22;
