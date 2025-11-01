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
  if (!supabaseServiceKey) console.error('  - SUPABASE_SERVICE_ROLE_KEY');
  console.error('\nPlease add these to your .env.local file');
  process.exit(1);
}

const supabase = createClient(supabaseUrl, supabaseServiceKey, {
  auth: {
    autoRefreshToken: false,
    persistSession: false
  }
});

async function checkMigrationsTable() {
  const { data, error } = await supabase
    .from('migrations')
    .select('id')
    .limit(1);
  
  if (error && error.code === 'PGRST205') {
    return false;
  }
  return true;
}

async function getExecutedMigrations() {
  const { data, error } = await supabase
    .from('migrations')
    .select('filename')
    .order('executed_at', { ascending: true });

  if (error) {
    if (error.code === 'PGRST205') {
      return [];
    }
    console.error('❌ Error fetching migrations:', error);
    return [];
  }

  return data.map(m => m.filename);
}

async function markMigrationComplete(filename) {
  const { error } = await supabase
    .from('migrations')
    .insert({ filename });
  
  if (error) {
    throw error;
  }
}

async function generateMigrationScript() {
  console.log('📝 Generating SQL script for pending migrations...\n');
  
  // Check if migrations table exists
  const tableExists = await checkMigrationsTable();
  
  if (!tableExists) {
    console.log('⚠️  Migrations table not found!');
    console.log('👉 First, run the setup script in your Supabase dashboard:');
    console.log('   File: supabase/setup-migrations.sql\n');
    return;
  }
  
  // Get executed migrations
  const executedMigrations = await getExecutedMigrations();
  console.log(`📊 Found ${executedMigrations.length} executed migrations`);
  if (executedMigrations.length > 0) {
    executedMigrations.forEach(m => console.log(`  ✓ ${m}`));
  }
  
  // Get all migration files
  const migrationsDir = path.join(__dirname, 'migrations');
  const files = await fs.readdir(migrationsDir);
  const sqlFiles = files
    .filter(f => f.endsWith('.sql'))
    .sort();
  
  // Find pending migrations
  const pending = sqlFiles.filter(f => !executedMigrations.includes(f));
  
  if (pending.length === 0) {
    console.log('\n✨ All migrations are up to date!');
    return;
  }
  
  console.log(`\n📋 ${pending.length} pending migration(s):`);
  pending.forEach(p => console.log(`  ○ ${p}`));
  
  // Generate combined SQL script
  let combinedSQL = `-- Generated migration script
-- Created: ${new Date().toISOString()}
-- Pending migrations: ${pending.join(', ')}

BEGIN; -- Start transaction

`;
  
  for (const file of pending) {
    const filepath = path.join(migrationsDir, file);
    const sql = await fs.readFile(filepath, 'utf-8');
    
    combinedSQL += `
-- ============================================
-- Migration: ${file}
-- ============================================

${sql}

-- Mark migration as complete
INSERT INTO migrations (filename) VALUES ('${file}');

`;
  }
  
  combinedSQL += `
COMMIT; -- Commit transaction

-- If any errors occur, the entire transaction will be rolled back
`;
  
  // Save the combined script
  const outputPath = path.join(__dirname, 'pending-migrations.sql');
  await fs.writeFile(outputPath, combinedSQL);
  
  console.log('\n✅ Migration script generated: supabase/pending-migrations.sql');
  console.log('\n📌 Next steps:');
  console.log('1. Go to your Supabase dashboard');
  console.log('2. Open the SQL Editor');
  console.log('3. Copy and paste the content from: supabase/pending-migrations.sql');
  console.log('4. Review the SQL and click "Run"');
  console.log('5. After successful execution, run this script again to verify');
  
  // Also try to mark them as complete if we can
  console.log('\n🔄 Attempting to mark migrations as complete...');
  let successCount = 0;
  for (const file of pending) {
    try {
      await markMigrationComplete(file);
      console.log(`  ✓ Marked ${file} as pending execution`);
      successCount++;
    } catch (error) {
      console.log(`  ⚠️  Could not mark ${file}: Run the SQL manually`);
    }
  }
  
  if (successCount === pending.length) {
    console.log('\n✅ All migrations marked! Remember to run the SQL in Supabase dashboard.');
  }
}

// Run the script
generateMigrationScript().catch(error => {
  console.error('💥 Error:', error.message);
  process.exit(1);
});