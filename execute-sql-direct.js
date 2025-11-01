#!/usr/bin/env node

import fetch from 'node-fetch';
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

async function executeSQLDirect(sql) {
  const url = `${supabaseUrl}/rest/v1/rpc/exec_sql`;
  
  try {
    const response = await fetch(url, {
      method: 'POST',
      headers: {
        'apikey': supabaseServiceKey,
        'Authorization': `Bearer ${supabaseServiceKey}`,
        'Content-Type': 'application/json',
        'Prefer': 'return=representation'
      },
      body: JSON.stringify({ query: sql })
    });
    
    if (!response.ok) {
      // Try another approach - use the query endpoint
      const queryUrl = `${supabaseUrl}/rest/v1/`;
      
      // For DDL statements, we need to use a different approach
      // Let's try to create tables one by one using POST requests
      console.log('⚠️  Direct SQL execution not available via REST API');
      return { success: false, message: 'Need to use Supabase Dashboard' };
    }
    
    const result = await response.json();
    return { success: true, result };
  } catch (error) {
    return { success: false, error: error.message };
  }
}

async function runMigrationsDirect() {
  console.log('🚀 Attempting to execute migrations directly...\n');
  
  // Read the initial schema migration
  const migrationPath = path.join(__dirname, 'supabase', 'migrations', '001_initial_schema.sql');
  const sql = await fs.readFile(migrationPath, 'utf-8');
  
  // Parse and execute SQL statements one by one
  const statements = sql
    .split(';')
    .map(s => s.trim())
    .filter(s => s && !s.match(/^\s*$/) && !s.trim().startsWith('--'));
  
  console.log(`📝 Found ${statements.length} SQL statements to execute\n`);
  
  // Unfortunately, Supabase doesn't allow direct DDL execution via REST API
  // We need to use the Dashboard or the CLI with local development
  
  console.log('⚠️  Direct SQL execution via REST API is not supported for DDL statements');
  console.log('\n📌 You have two options:\n');
  console.log('Option 1: Use Supabase Dashboard (Recommended)');
  console.log('=========================================');
  console.log('1. Go to: https://supabase.com/dashboard/project/dylfyougztyphwgbzngt/sql/new');
  console.log('2. Copy the SQL from run-all-migrations.sql');
  console.log('3. Paste and click "Run"\n');
  
  console.log('Option 2: Use Supabase CLI with Docker');
  console.log('=======================================');
  console.log('1. Install Docker Desktop: https://www.docker.com/products/docker-desktop');
  console.log('2. Start Docker');
  console.log('3. Run: npx supabase start');
  console.log('4. Run: npx supabase db push\n');
  
  // Let's also create a simpler version with just the essential tables
  console.log('📝 Creating simplified migration for manual execution...\n');
  
  const simplifiedSQL = `
-- Simplified migration - Run this in Supabase SQL Editor
-- This creates the essential tables for the application

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

-- Users table (if needed)
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

-- Insert test record
INSERT INTO connection_test (test_name, test_value) 
VALUES ('Migration Test', 'Tables created successfully at ' || NOW()::TEXT);
`;
  
  // Save simplified SQL
  const simplifiedPath = path.join(__dirname, 'create-tables-simple.sql');
  await fs.writeFile(simplifiedPath, simplifiedSQL);
  
  console.log('✅ Simplified SQL saved to: create-tables-simple.sql');
  console.log('\n📋 Here\'s the simplified SQL (copy and run in Supabase Dashboard):\n');
  console.log('================== COPY BELOW THIS LINE ==================\n');
  console.log(simplifiedSQL);
  console.log('\n================== COPY ABOVE THIS LINE ==================');
}

// Run the script
runMigrationsDirect().catch(error => {
  console.error('💥 Error:', error.message);
  process.exit(1);
});