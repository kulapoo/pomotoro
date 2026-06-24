-- Reverse the completed_at addition.
--
-- SQLite >= 3.35 supports DROP COLUMN directly.
ALTER TABLE tasks DROP COLUMN completed_at;
