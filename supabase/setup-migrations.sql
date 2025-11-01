-- Run this SQL in your Supabase dashboard to set up the migrations system
-- Go to: SQL Editor > New Query > Paste this code > Run

-- 1. Create migrations tracking table
CREATE TABLE IF NOT EXISTS migrations (
  id SERIAL PRIMARY KEY,
  filename VARCHAR(255) NOT NULL UNIQUE,
  executed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 2. Create a function to help execute migrations (optional but useful)
CREATE OR REPLACE FUNCTION mark_migration_complete(migration_filename VARCHAR)
RETURNS void AS $$
BEGIN
  INSERT INTO migrations (filename) 
  VALUES (migration_filename)
  ON CONFLICT (filename) DO NOTHING;
END;
$$ LANGUAGE plpgsql;

-- 3. Check what's been set up
SELECT 'Migrations table created successfully' as status;
SELECT * FROM migrations ORDER BY executed_at;
