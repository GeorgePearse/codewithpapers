#!/usr/bin/env node

import { createClient } from '@supabase/supabase-js';
import dotenv from 'dotenv';

// Load environment variables
dotenv.config({ path: '.env.local' });

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

async function listAllTables() {
  console.log('🔍 Listing all tables created by migrations...\n');
  
  // Tables from 001_initial_schema.sql
  const migrationTables = [
    'papers',
    'datasets',
    'benchmarks',
    'implementations',
    'benchmark_results',
    'paper_datasets',
    'connection_test',
    'migrations'
  ];
  
  console.log('📊 Checking tables from migrations:\n');
  console.log('=' .repeat(60));
  
  let existingTables = [];
  let missingTables = [];
  
  for (const tableName of migrationTables) {
    try {
      const { data, error, count } = await supabase
        .from(tableName)
        .select('*', { count: 'exact', head: true });
      
      if (!error) {
        console.log(`✅ ${tableName.padEnd(25)} EXISTS (${count || 0} rows)`);
        existingTables.push(tableName);
      } else {
        if (error.message.includes('not found') || error.message.includes('does not exist')) {
          console.log(`❌ ${tableName.padEnd(25)} DOES NOT EXIST`);
          missingTables.push(tableName);
        } else {
          console.log(`⚠️  ${tableName.padEnd(25)} ${error.message}`);
        }
      }
    } catch (e) {
      console.log(`❌ ${tableName.padEnd(25)} Error: ${e.message}`);
      missingTables.push(tableName);
    }
  }
  
  console.log('=' .repeat(60));
  console.log('\n📈 Summary:');
  console.log(`   ✅ Tables created: ${existingTables.length}/${migrationTables.length}`);
  console.log(`   ❌ Tables missing: ${missingTables.length}/${migrationTables.length}`);
  
  if (existingTables.length > 0) {
    console.log('\n✅ Existing tables:');
    existingTables.forEach(t => console.log(`   - ${t}`));
  }
  
  if (missingTables.length > 0) {
    console.log('\n❌ Missing tables:');
    missingTables.forEach(t => console.log(`   - ${t}`));
    
    console.log('\n📝 To create missing tables, we need to run additional migrations.');
  }
  
  // Test inserting data into connection_test table
  if (existingTables.includes('connection_test')) {
    console.log('\n🧪 Testing connection_test table...');
    try {
      const { data, error } = await supabase
        .from('connection_test')
        .insert({
          test_name: 'Table Verification',
          test_value: `Tables verified at ${new Date().toISOString()}`
        })
        .select();
      
      if (!error && data) {
        console.log('   ✅ Successfully inserted test record');
      }
    } catch (e) {
      console.log('   ⚠️  Could not insert test record');
    }
  }
  
  console.log('\n✅ Table check complete!');
}

// Run the script
listAllTables().catch(error => {
  console.error('💥 Error:', error.message);
  process.exit(1);
});