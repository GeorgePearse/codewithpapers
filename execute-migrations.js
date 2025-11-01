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

const supabaseUrl = process.env.VITE_SUPABASE_URL;
const supabaseServiceKey = process.env.SUPABASE_SERVICE_ROLE_KEY;

if (!supabaseUrl || !supabaseServiceKey) {
  console.error('❌ Missing required environment variables');
  process.exit(1);
}

const supabase = createClient(supabaseUrl, supabaseServiceKey, {
  auth: {
    autoRefreshToken: false,
    persistSession: false
  }
});

async function executeSQLFile(filepath, filename) {
  console.log(`\n📝 Executing migration: ${filename}`);
  
  const sql = await fs.readFile(filepath, 'utf-8');
  
  // Split SQL by statements (simple approach - splits by semicolon)
  // Filter out empty statements and comments
  const statements = sql
    .split(';')
    .map(s => s.trim())
    .filter(s => s && !s.startsWith('--'));
  
  let successCount = 0;
  let errorCount = 0;
  
  for (const statement of statements) {
    if (!statement || statement.match(/^\s*$/)) continue;
    
    // Skip SELECT statements as they're just for verification
    if (statement.toUpperCase().trim().startsWith('SELECT')) {
      console.log('  ⏭️  Skipping SELECT statement (verification only)');
      continue;
    }
    
    console.log(`  🔄 Executing: ${statement.substring(0, 50)}...`);
    
    try {
      // For CREATE TABLE statements, we need to execute them differently
      // Since Supabase client doesn't have a direct SQL execution method,
      // we'll output the SQL for manual execution
      console.log('  ⚠️  Direct SQL execution not available via client');
      console.log('     SQL statement needs to be run in Supabase dashboard');
      errorCount++;
    } catch (error) {
      console.log(`  ❌ Error: ${error.message}`);
      errorCount++;
    }
  }
  
  return { successCount, errorCount };
}

async function runMigrations() {
  console.log('🚀 Starting migration execution...\n');
  
  // First, let's check what migrations we have
  const migrationsDir = path.join(__dirname, 'supabase', 'migrations');
  const files = await fs.readdir(migrationsDir);
  const sqlFiles = files
    .filter(f => f.endsWith('.sql'))
    .sort();
  
  console.log(`Found ${sqlFiles.length} migration files:`);
  sqlFiles.forEach(f => console.log(`  📄 ${f}`));
  
  // Let's generate a combined SQL file that can be run in Supabase dashboard
  console.log('\n📝 Generating combined migration SQL...\n');
  
  let combinedSQL = `-- Combined Migration Script
-- Generated: ${new Date().toISOString()}
-- Run this in your Supabase SQL Editor

`;
  
  for (const file of sqlFiles) {
    const filepath = path.join(migrationsDir, file);
    const sql = await fs.readFile(filepath, 'utf-8');
    
    // Skip the connection test migration if it's just for testing
    if (file === '002_test_connection.sql') {
      console.log(`  ⏭️  Skipping ${file} (test migration)`);
      continue;
    }
    
    combinedSQL += `
-- ============================================
-- Migration: ${file}
-- ============================================

${sql}

`;
  }
  
  // Save the combined SQL
  const outputPath = path.join(__dirname, 'run-all-migrations.sql');
  await fs.writeFile(outputPath, combinedSQL);
  
  console.log('✅ Combined migration SQL saved to: run-all-migrations.sql');
  console.log('\n📌 To execute the migrations:');
  console.log('1. Go to your Supabase dashboard: https://supabase.com/dashboard');
  console.log('2. Select your project');
  console.log('3. Go to SQL Editor');
  console.log('4. Copy and paste the contents of run-all-migrations.sql');
  console.log('5. Click "Run" to execute');
  console.log('\n⚠️  Important: Review the SQL before running to ensure it\'s safe!');
  
  // Also output the SQL to console for immediate use
  console.log('\n📋 Here\'s the SQL to run:\n');
  console.log('================== COPY BELOW THIS LINE ==================\n');
  console.log(combinedSQL);
  console.log('\n================== COPY ABOVE THIS LINE ==================');
}

// Run the script
runMigrations().catch(error => {
  console.error('💥 Error:', error.message);
  process.exit(1);
});