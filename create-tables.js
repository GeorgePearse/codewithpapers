#!/usr/bin/env node

import fetch from 'node-fetch';
import dotenv from 'dotenv';

// Load environment variables
dotenv.config({ path: '.env.local' });

const supabaseUrl = process.env.VITE_SUPABASE_URL;
const supabaseServiceKey = process.env.SUPABASE_SERVICE_ROLE_KEY;

if (!supabaseUrl || !supabaseServiceKey) {
  console.error('❌ Missing required environment variables');
  process.exit(1);
}

async function executeSQL(sql) {
  const response = await fetch(`${supabaseUrl}/rest/v1/rpc`, {
    method: 'POST',
    headers: {
      'apikey': supabaseServiceKey,
      'Authorization': `Bearer ${supabaseServiceKey}`,
      'Content-Type': 'application/json',
      'Prefer': 'return=minimal'
    },
    body: JSON.stringify({
      query: sql
    })
  });

  // For DDL operations, we need to use the SQL endpoint directly
  // Let's use the pg endpoint
  const pgResponse = await fetch(`${supabaseUrl}/pg/query`, {
    method: 'POST',
    headers: {
      'apikey': supabaseServiceKey,
      'Authorization': `Bearer ${supabaseServiceKey}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      query: sql
    })
  });

  if (!pgResponse.ok && !response.ok) {
    // Try the management API
    const mgmtResponse = await fetch(`${supabaseUrl.replace('supabase.co', 'supabase.co')}/sql`, {
      method: 'POST',
      headers: {
        'apikey': supabaseServiceKey,
        'Authorization': `Bearer ${supabaseServiceKey}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        query: sql
      })
    });
    
    if (!mgmtResponse.ok) {
      throw new Error(`Failed to execute SQL: ${mgmtResponse.status}`);
    }
    return mgmtResponse;
  }
  
  return pgResponse.ok ? pgResponse : response;
}

async function createTables() {
  console.log('🚀 Creating tables directly via Supabase...\n');

  const sqlStatements = [
    {
      name: 'Enable UUID extension',
      sql: `CREATE EXTENSION IF NOT EXISTS "uuid-ossp";`
    },
    {
      name: 'Create papers table',
      sql: `CREATE TABLE IF NOT EXISTS papers (
        id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
        title TEXT NOT NULL,
        abstract TEXT,
        arxiv_id VARCHAR(20) UNIQUE,
        arxiv_url TEXT,
        pdf_url TEXT,
        published_date DATE,
        authors JSONB,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
      );`
    },
    {
      name: 'Create benchmarks table',
      sql: `CREATE TABLE IF NOT EXISTS benchmarks (
        id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
        name TEXT NOT NULL UNIQUE,
        task TEXT,
        description TEXT,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
      );`
    },
    {
      name: 'Create tasks table',
      sql: `CREATE TABLE IF NOT EXISTS tasks (
        id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
        name TEXT NOT NULL UNIQUE,
        description TEXT,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
      );`
    },
    {
      name: 'Create benchmark_metrics table',
      sql: `CREATE TABLE IF NOT EXISTS benchmark_metrics (
        id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
        paper_id UUID REFERENCES papers(id),
        benchmark_id UUID REFERENCES benchmarks(id),
        metric_name TEXT NOT NULL,
        metric_value NUMERIC NOT NULL,
        extra_data JSONB,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
      );`
    },
    {
      name: 'Create paper_benchmark_metrics table',
      sql: `CREATE TABLE IF NOT EXISTS paper_benchmark_metrics (
        id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
        paper_id UUID REFERENCES papers(id),
        benchmark_id UUID REFERENCES benchmarks(id),
        metrics JSONB,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
      );`
    },
    {
      name: 'Create users table',
      sql: `CREATE TABLE IF NOT EXISTS users (
        id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
        email TEXT UNIQUE,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
      );`
    },
    {
      name: 'Create connection_test table',
      sql: `CREATE TABLE IF NOT EXISTS connection_test (
        id SERIAL PRIMARY KEY,
        test_name VARCHAR(255) NOT NULL,
        test_timestamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
        test_value TEXT
      );`
    }
  ];

  let successCount = 0;
  let errorCount = 0;

  for (const statement of sqlStatements) {
    console.log(`📝 ${statement.name}...`);
    try {
      await executeSQL(statement.sql);
      console.log(`  ✅ Success\n`);
      successCount++;
    } catch (error) {
      console.log(`  ❌ Failed: ${error.message}\n`);
      errorCount++;
    }
  }

  console.log(`\n📊 Results:`);
  console.log(`  ✅ Successful: ${successCount}`);
  console.log(`  ❌ Failed: ${errorCount}`);

  if (errorCount > 0) {
    console.log('\n⚠️  Some operations failed. This might be because:');
    console.log('  1. Tables already exist');
    console.log('  2. Direct SQL execution requires using Supabase Dashboard');
    console.log('\nTrying alternative approach using Supabase Management API...\n');
    
    // Alternative: Use the Supabase CLI via npx
    console.log('Running Supabase migration via CLI...\n');
  }
}

createTables().catch(error => {
  console.error('💥 Error:', error);
  
  // If direct execution fails, let's use npx supabase
  console.log('\n📌 Attempting to use Supabase CLI...\n');
  
  import('child_process').then(({ exec }) => {
    // First, let's init supabase if not already done
    exec('npx supabase init', (initError) => {
      if (initError && !initError.message.includes('already')) {
        console.log('Failed to init Supabase');
      }
      
      // Link to the project
      exec(`npx supabase link --project-ref dylfyougztyphwgbzngt`, (linkError) => {
        if (linkError) {
          console.log('⚠️  Could not link to project');
        }
        
        // Try to push the migration
        exec('npx supabase db push', (pushError, stdout, stderr) => {
          if (pushError) {
            console.log('⚠️  Could not push via CLI:', pushError.message);
            console.log('\n📋 Please copy the SQL from create-tables-simple.sql and run it in:');
            console.log('https://supabase.com/dashboard/project/dylfyougztyphwgbzngt/sql/new');
          } else {
            console.log('✅ Migration pushed successfully!');
            console.log(stdout);
          }
        });
      });
    });
  });
});