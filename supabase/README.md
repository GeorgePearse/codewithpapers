# Supabase Database Migrations

This directory contains database migrations for the Papers with Code project.

## Setup

### Prerequisites

1. **Environment Variables**: Add to your `.env.local`:
   ```
   VITE_SUPABASE_URL=your_supabase_url
   VITE_SUPABASE_ANON_KEY=your_anon_key
   SUPABASE_SERVICE_ROLE_KEY=your_service_role_key  # For migrations only
   ```

2. **Supabase CLI** (recommended):
   ```bash
   # macOS
   brew install supabase/tap/supabase
   
   # Other platforms
   # Visit: https://supabase.com/docs/guides/cli/getting-started
   ```

## Running Migrations

### Option 1: Using Supabase CLI (Recommended)

```bash
# Run the migration script
./supabase/migrate.sh
```

### Option 2: Manual via Supabase Dashboard

1. Go to your [Supabase Dashboard](https://app.supabase.com)
2. Navigate to **SQL Editor**
3. Copy the contents of each `.sql` file in `migrations/`
4. Run them in order (001, 002, etc.)

### Option 3: Using Node.js Script (Development)

```bash
# Install dependencies if needed
npm install @supabase/supabase-js dotenv

# Run migrations
node supabase/run_migrations.js
```

**Note**: The Node.js script will show you the SQL to run but won't execute it directly due to Supabase security restrictions.

## Migration Files

- `001_initial_schema.sql` - Creates the base tables:
  - `papers` - Research papers with abstracts and metadata
  - `datasets` - Dataset information
  - `benchmarks` - Benchmark definitions
  - `implementations` - Code implementations for papers
  - `benchmark_results` - Performance metrics
  - `paper_datasets` - Many-to-many relationship

## Creating New Migrations

1. Create a new file in `migrations/` with format: `XXX_description.sql`
   - Use sequential numbering (002, 003, etc.)
   - Use descriptive names

2. Write your SQL migration

3. Run the migration using one of the methods above

## Best Practices

- Always test migrations on a development database first
- Keep migrations small and focused
- Never modify existing migration files
- Use `IF NOT EXISTS` clauses to make migrations idempotent
- Document any complex migrations with comments

## Troubleshooting

### Missing Service Role Key
The service role key is needed for programmatic migrations. Get it from:
1. Supabase Dashboard → Settings → API
2. Copy the `service_role` key (keep it secret!)

### Permission Denied
If you get permission errors, ensure you're using the service role key, not the anon key.

### Migration Already Exists
The migrations track which files have been executed. If you need to re-run a migration:
1. Delete the record from the `migrations` table
2. Run the migration again