# Supabase Setup

## Current Status

The app is configured to use Supabase for database access, but requires configuration.

## Setup Instructions

1. **Create a Supabase project** at https://supabase.com

2. **Get your credentials** from Project Settings → API

3. **Create .env.local** with:
   ```bash
   VITE_SUPABASE_URL=https://your-project.supabase.co
   VITE_SUPABASE_ANON_KEY=your-anon-key-here
   ```

4. **Import the database schema** from `supabase/migrations/001_initial_schema.sql`

5. **Load data** (see DATABASE_SETUP.md)

## Alternative: Keep Using YAML

If you prefer to use the YAML files instead of a database, you can revert to the old App:

```bash
mv src/App-old.jsx src/App.jsx
```

This uses the static YAML files in `public/` which work great for smaller datasets.
