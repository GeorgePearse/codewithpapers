#!/usr/bin/env node

import { createClient } from '@supabase/supabase-js';
import fs from 'fs/promises';
import path from 'path';
import { fileURLToPath } from 'url';
import dotenv from 'dotenv';

// Load environment variables
dotenv.config({ path: '.env.local' });

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Supabase client configuration
const supabaseUrl = process.env.VITE_SUPABASE_URL;
const supabaseServiceKey = process.env.SUPABASE_SERVICE_ROLE_KEY;

if (!supabaseUrl || !supabaseServiceKey) {
  console.error('❌ Missing required environment variables:');
  if (!supabaseUrl) console.error('  - VITE_SUPABASE_URL');
  if (!supabaseServiceKey) console.error('  - SUPABASE_SERVICE_ROLE_KEY (needed for migrations)');
  console.error('\nPlease add these to your .env.local file');
  process.exit(1);
}

const supabase = createClient(supabaseUrl, supabaseServiceKey, {
  auth: {
    autoRefreshToken: false,
    persistSession: false
  }
});

// Function to execute raw SQL using the Supabase REST API
async function executeSQL(sql) {
  const response = await fetch(`${supabaseUrl}/rest/v1/rpc/exec_sql`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'apikey': supabaseServiceKey,
      'Authorization': `Bearer ${supabaseServiceKey}`
    },
    body: JSON.stringify({ query: sql })
  });

  if (!response.ok) {
    // If RPC doesn't exist, try the SQL endpoint directly
    const sqlResponse = await fetch(`${supabaseUrl}/pg/sql`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'apikey': supabaseServiceKey,
        'Authorization': `Bearer ${supabaseServiceKey}`
      },
      body: JSON.stringify({ query: sql })
    });

    if (!sqlResponse.ok) {
      const error = await sqlResponse.text();
      throw new Error(`SQL execution failed: ${error}`);
    }

    return await sqlResponse.json();
  }

  return await response.json();
}

// Migrations tracking table
const MIGRATIONS_TABLE = `
CREATE TABLE IF NOT EXISTS migrations (
  id SERIAL PRIMARY KEY,
  filename VARCHAR(255) NOT NULL UNIQUE,
  executed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);`;

async function ensureMigrationsTable() {
  try {
    console.log('📝 Ensuring migrations table exists...');
    await executeSQL(MIGRATIONS_TABLE);
    console.log('✅ Migrations table ready\n');
  } catch (error) {
    // Table might already exist, try to query it
    const { error: queryError } = await supabase
      .from('migrations')
      .select('id')
      .limit(1);
    
    if (queryError && queryError.code === 'PGRST205') {
      console.error('❌ Could not create migrations table. You may need to create it manually:');
      console.log(MIGRATIONS_TABLE);
      throw new Error('Migrations table setup failed');
    }
    // If no error or different error, table exists
    console.log('✅ Migrations table already exists\n');
  }
}

async function getExecutedMigrations() {
  const { data, error } = await supabase
    .from('migrations')
    .select('filename')
    .order('executed_at', { ascending: true });

  if (error && error.code === 'PGRST205') {
    // Table doesn't exist yet
    return [];
  }
  
  if (error) {
    console.error('❌ Error fetching migrations:', error);
    return [];
  }

  return data.map(m => m.filename);
}

async function executeMigration(filepath, filename) {
  console.log(`🔄 Running migration: ${filename}`);
  
  try {
    const sql = await fs.readFile(filepath, 'utf-8');
    
    // Split SQL into individual statements (basic split on semicolon)
    // This handles most cases but may need refinement for complex SQL
    const statements = sql
      .split(/;\s*$/m)
      .map(s => s.trim())
      .filter(s => s.length > 0 && !s.startsWith('--'));
    
    console.log(`📋 Executing ${statements.length} SQL statement(s)...`);
    
    // Execute each statement
    for (let i = 0; i < statements.length; i++) {
      const statement = statements[i];
      if (statement.trim()) {
        try {
          await executeSQL(statement + ';');
          console.log(`  ✓ Statement ${i + 1}/${statements.length} executed`);
        } catch (error) {
          console.error(`  ✗ Statement ${i + 1} failed:`, error.message);
          console.log(`    SQL: ${statement.substring(0, 100)}...`);
          throw error;
        }
      }
    }
    
    // Record the migration as executed
    const { error: insertError } = await supabase
      .from('migrations')
      .insert({ filename });
    
    if (insertError) {
      throw insertError;
    }
    
    console.log(`✅ Migration ${filename} completed successfully\n`);
    
  } catch (error) {
    console.error(`❌ Error executing migration ${filename}:`, error.message);
    throw error;
  }
}

async function runMigrations() {
  console.log('🚀 Starting database migrations...\n');
  
  try {
    // Ensure migrations table exists
    await ensureMigrationsTable();
    
    // Get list of executed migrations
    const executedMigrations = await getExecutedMigrations();
    console.log(`📊 Found ${executedMigrations.length} executed migrations`);
    if (executedMigrations.length > 0) {
      executedMigrations.forEach(m => console.log(`  - ${m}`));
    }
    console.log('');
    
    // Get all migration files
    const migrationsDir = path.join(__dirname, 'migrations');
    const files = await fs.readdir(migrationsDir);
    const sqlFiles = files
      .filter(f => f.endsWith('.sql'))
      .sort(); // Ensure migrations run in order
    
    console.log(`📁 Found ${sqlFiles.length} migration files total\n`);
    
    // Execute pending migrations
    let pendingCount = 0;
    const pending = [];
    
    for (const file of sqlFiles) {
      if (!executedMigrations.includes(file)) {
        pending.push(file);
      }
    }
    
    if (pending.length === 0) {
      console.log('✨ All migrations are up to date!');
      return;
    }
    
    console.log(`📋 ${pending.length} pending migration(s) to run:`);
    pending.forEach(p => console.log(`  - ${p}`));
    console.log('');
    
    for (const file of pending) {
      const filepath = path.join(migrationsDir, file);
      await executeMigration(filepath, file);
      pendingCount++;
    }
    
    console.log(`🎉 Successfully executed ${pendingCount} migration(s)`);
    
  } catch (error) {
    console.error('\n💥 Migration process failed:', error.message);
    console.log('\n📌 Troubleshooting:');
    console.log('1. Check your Supabase dashboard to see if tables were partially created');
    console.log('2. You can manually run migrations via the SQL editor in Supabase dashboard');
    console.log('3. Ensure your service role key has sufficient permissions');
    throw error;
  }
}

// Run migrations
runMigrations().catch(error => {
  process.exit(1);
});