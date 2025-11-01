#!/usr/bin/env node

import { createClient } from '@supabase/supabase-js';
import dotenv from 'dotenv';

// Load environment variables
dotenv.config({ path: '.env.local' });

const supabaseUrl = process.env.VITE_SUPABASE_URL;
const supabaseServiceKey = process.env.SUPABASE_SERVICE_ROLE_KEY;

if (!supabaseUrl || !supabaseServiceKey) {
  console.error('❌ Missing required environment variables:');
  if (!supabaseUrl) console.error('  - VITE_SUPABASE_URL');
  if (!supabaseServiceKey) console.error('  - SUPABASE_SERVICE_ROLE_KEY');
  process.exit(1);
}

const supabase = createClient(supabaseUrl, supabaseServiceKey, {
  auth: {
    autoRefreshToken: false,
    persistSession: false
  }
});

async function listTables() {
  console.log('🔍 Listing all tables in the database...\n');
  
  // Query to get all tables in the public schema
  const { data, error } = await supabase.rpc('get_tables_list', {}, {
    count: 'exact'
  }).single();
  
  // If the RPC doesn't exist, try a direct query
  if (error) {
    console.log('⚠️  RPC function not found, trying direct query...\n');
    
    // Use a raw SQL query to get tables
    const query = `
      SELECT 
        schemaname,
        tablename,
        tableowner
      FROM pg_tables 
      WHERE schemaname = 'public'
      ORDER BY tablename;
    `;
    
    // Execute raw SQL through Supabase
    const { data: tables, error: queryError } = await supabase.rpc('exec_sql', { query });
    
    if (queryError) {
      // If that doesn't work either, let's try another approach
      console.log('📊 Attempting to list tables by checking known table names...\n');
      
      const knownTables = [
        'users',
        'papers',
        'benchmarks',
        'tasks',
        'benchmark_metrics',
        'paper_benchmark_metrics',
        'migrations',
        'connection_test'
      ];
      
      console.log('Checking for existence of known tables:\n');
      
      for (const tableName of knownTables) {
        try {
          const { data, error } = await supabase
            .from(tableName)
            .select('*')
            .limit(0);
          
          if (!error) {
            console.log(`  ✅ ${tableName} - EXISTS`);
          } else if (error.code === '42P01') {
            console.log(`  ❌ ${tableName} - DOES NOT EXIST`);
          } else {
            console.log(`  ⚠️  ${tableName} - ${error.message}`);
          }
        } catch (e) {
          console.log(`  ❌ ${tableName} - Error checking table`);
        }
      }
      
      return;
    }
    
    if (tables && tables.length > 0) {
      console.log(`Found ${tables.length} table(s) in the public schema:\n`);
      tables.forEach(table => {
        console.log(`  📋 ${table.tablename}`);
        console.log(`     Schema: ${table.schemaname}`);
        console.log(`     Owner: ${table.tableowner}\n`);
      });
    } else {
      console.log('⚠️  No tables found in the public schema');
    }
    return;
  }
  
  // Process successful RPC result
  if (data && data.length > 0) {
    console.log(`Found ${data.length} table(s):\n`);
    data.forEach(table => {
      console.log(`  📋 ${table.name}`);
    });
  } else {
    console.log('⚠️  No tables found');
  }
}

// Also try to get table count information
async function getTableInfo() {
  console.log('\n📈 Getting row counts for accessible tables...\n');
  
  const tablesToCheck = [
    'users',
    'papers',
    'benchmarks', 
    'tasks',
    'benchmark_metrics',
    'paper_benchmark_metrics',
    'migrations',
    'connection_test'
  ];
  
  for (const table of tablesToCheck) {
    try {
      const { count, error } = await supabase
        .from(table)
        .select('*', { count: 'exact', head: true });
      
      if (!error) {
        console.log(`  📊 ${table}: ${count || 0} rows`);
      }
    } catch (e) {
      // Table doesn't exist, skip
    }
  }
}

// Run the script
(async () => {
  try {
    await listTables();
    await getTableInfo();
    console.log('\n✅ Table listing complete!');
  } catch (error) {
    console.error('💥 Error:', error.message);
    process.exit(1);
  }
})();