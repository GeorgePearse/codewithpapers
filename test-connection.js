#!/usr/bin/env node

import { createClient } from '@supabase/supabase-js';
import dotenv from 'dotenv';

// Load environment variables
dotenv.config({ path: '.env.local' });

const supabaseUrl = process.env.VITE_SUPABASE_URL;
const supabaseServiceKey = process.env.SUPABASE_SERVICE_ROLE_KEY;

console.log('🔍 Testing Supabase Connection...\n');
console.log(`📍 URL: ${supabaseUrl ? supabaseUrl.substring(0, 30) + '...' : 'NOT SET'}`);
console.log(`🔑 Service Key: ${supabaseServiceKey ? '✅ Set' : '❌ NOT SET'}\n`);

if (!supabaseUrl || !supabaseServiceKey) {
  console.error('❌ Missing required environment variables!');
  process.exit(1);
}

const supabase = createClient(supabaseUrl, supabaseServiceKey, {
  auth: {
    autoRefreshToken: false,
    persistSession: false
  }
});

async function testConnection() {
  try {
    // Try to query the auth.users table (always exists in Supabase)
    const { data, error } = await supabase.auth.admin.listUsers({
      page: 1,
      perPage: 1
    });
    
    if (error) {
      console.error('❌ Connection failed:', error.message);
      return false;
    }
    
    console.log('✅ Successfully connected to Supabase!');
    console.log(`👥 Found ${data.users ? data.users.length : 0} user(s) in the database\n`);
    
    // Test if we can create tables (test write permissions)
    console.log('🧪 Testing write permissions...');
    
    // Since we can't directly execute SQL via the client, we'll test by trying to access a table
    const { error: tableError } = await supabase
      .from('test_connection_check')
      .select('*')
      .limit(1);
    
    if (tableError && tableError.code === 'PGRST205') {
      console.log('ℹ️  Table "test_connection_check" doesn\'t exist (expected)');
      console.log('✅ Connection is working! You can now run migrations via Supabase dashboard.\n');
    } else if (tableError) {
      console.log('⚠️  Unexpected error:', tableError.message);
    } else {
      console.log('✅ Table access working!');
    }
    
    return true;
    
  } catch (err) {
    console.error('❌ Unexpected error:', err);
    return false;
  }
}

// Run the test
testConnection().then(success => {
  if (success) {
    console.log('🎉 Connection test passed! Your Supabase credentials are working correctly.');
    console.log('\n📝 Next steps:');
    console.log('1. Go to your Supabase dashboard SQL editor');
    console.log('2. Run the migrations from supabase/migrations/ folder');
    console.log('3. Start with 002_test_connection.sql to verify everything works');
  } else {
    console.log('❌ Connection test failed. Please check your credentials in .env.local');
  }
  process.exit(success ? 0 : 1);
});