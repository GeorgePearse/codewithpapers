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

async function createAllTables() {
  console.log('🚀 Creating tables using Supabase client...\n');
  
  // Since we can't execute DDL directly through the client,
  // let's check what tables exist and create test data
  
  const tablesToCreate = [
    'papers',
    'benchmarks',
    'tasks',
    'benchmark_metrics',
    'paper_benchmark_metrics',
    'users',
    'connection_test'
  ];
  
  console.log('📊 Checking table existence and trying to initialize...\n');
  
  for (const tableName of tablesToCreate) {
    console.log(`Checking ${tableName}...`);
    
    try {
      // Try to select from the table
      const { data, error } = await supabase
        .from(tableName)
        .select('*')
        .limit(1);
      
      if (error) {
        if (error.message.includes('not found')) {
          console.log(`  ❌ Table '${tableName}' does not exist`);
          
          // Try to create it by inserting (this will fail but give us info)
          const { error: insertError } = await supabase
            .from(tableName)
            .insert({});
            
          if (insertError) {
            console.log(`     Need to create via SQL`);
          }
        } else {
          console.log(`  ⚠️  ${error.message}`);
        }
      } else {
        console.log(`  ✅ Table '${tableName}' exists`);
      }
    } catch (e) {
      console.log(`  ❌ Error: ${e.message}`);
    }
  }
  
  console.log('\n📝 Since we cannot create tables via the JS client, here\'s what to do:\n');
  console.log('Option 1: Use the SQL Editor in Supabase Dashboard');
  console.log('================================================');
  console.log('1. Open: https://supabase.com/dashboard/project/dylfyougztyphwgbzngt/sql/new');
  console.log('2. Copy the SQL below and paste it in the editor');
  console.log('3. Click "Run"\n');
  
  const createTableSQL = `
-- Create all required tables for CodeWithPapers

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Papers table
CREATE TABLE IF NOT EXISTS papers (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    title TEXT NOT NULL,
    abstract TEXT,
    arxiv_id VARCHAR(20) UNIQUE,
    arxiv_url TEXT,
    pdf_url TEXT,
    published_date DATE,
    authors JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Benchmarks table
CREATE TABLE IF NOT EXISTS benchmarks (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    task TEXT,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Tasks table
CREATE TABLE IF NOT EXISTS tasks (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Benchmark metrics table
CREATE TABLE IF NOT EXISTS benchmark_metrics (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    paper_id UUID REFERENCES papers(id),
    benchmark_id UUID REFERENCES benchmarks(id),
    metric_name TEXT NOT NULL,
    metric_value NUMERIC NOT NULL,
    extra_data JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Paper-benchmark-metrics relationship
CREATE TABLE IF NOT EXISTS paper_benchmark_metrics (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    paper_id UUID REFERENCES papers(id),
    benchmark_id UUID REFERENCES benchmarks(id),
    metrics JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    email TEXT UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Connection test table
CREATE TABLE IF NOT EXISTS connection_test (
    id SERIAL PRIMARY KEY,
    test_name VARCHAR(255) NOT NULL,
    test_timestamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    test_value TEXT
);

-- Insert test record to verify
INSERT INTO connection_test (test_name, test_value) 
VALUES ('Table Creation', 'All tables created successfully at ' || NOW()::TEXT)
ON CONFLICT DO NOTHING;

-- Verify creation
SELECT 'Tables created successfully!' as status;
`;

  console.log('================== COPY SQL BELOW ==================\n');
  console.log(createTableSQL);
  console.log('\n================== END SQL ==================\n');
  
  console.log('Option 2: Get your database password and use Supabase CLI');
  console.log('==========================================================');
  console.log('1. Go to: https://supabase.com/dashboard/project/dylfyougztyphwgbzngt/settings/database');
  console.log('2. Find or reset your database password');
  console.log('3. Run: npx supabase link --project-ref dylfyougztyphwgbzngt');
  console.log('4. Enter the database password when prompted');
  console.log('5. Run: npx supabase db push');
}

// Run the script
createAllTables().catch(error => {
  console.error('💥 Error:', error.message);
  process.exit(1);
});