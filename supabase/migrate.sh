#!/bin/bash

# Simple bash script to run migrations using Supabase CLI
# Requires Supabase CLI to be installed: https://supabase.com/docs/guides/cli

set -e

echo "🚀 Running Supabase migrations..."

# Check if supabase CLI is installed
if ! command -v supabase &> /dev/null; then
    echo "❌ Supabase CLI is not installed."
    echo "📦 Install it with: brew install supabase/tap/supabase"
    echo "   Or visit: https://supabase.com/docs/guides/cli/getting-started"
    exit 1
fi

# Load environment variables
if [ -f .env.local ]; then
    export $(cat .env.local | grep -v '^#' | xargs)
fi

# Check for required environment variables
if [ -z "$VITE_SUPABASE_URL" ]; then
    echo "❌ VITE_SUPABASE_URL is not set in .env.local"
    exit 1
fi

# Extract project ID from Supabase URL
PROJECT_ID=$(echo $VITE_SUPABASE_URL | sed -n 's/https:\/\/\([^.]*\).supabase.co/\1/p')

if [ -z "$PROJECT_ID" ]; then
    echo "❌ Could not extract project ID from Supabase URL"
    exit 1
fi

echo "📌 Project ID: $PROJECT_ID"

# Link to the project (if not already linked)
echo "🔗 Linking to Supabase project..."
supabase link --project-ref $PROJECT_ID || true

# Run migrations
echo "📝 Applying migrations..."
for migration in supabase/migrations/*.sql; do
    if [ -f "$migration" ]; then
        filename=$(basename "$migration")
        echo "  → Running $filename"
        supabase db push --file "$migration"
    fi
done

echo "✅ Migrations completed successfully!"