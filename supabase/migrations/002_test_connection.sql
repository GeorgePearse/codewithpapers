-- Test migration to verify Supabase connection
-- This creates a simple test table that can be safely dropped after verification

CREATE TABLE IF NOT EXISTS connection_test (
    id SERIAL PRIMARY KEY,
    test_name VARCHAR(255) NOT NULL,
    test_timestamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    test_value TEXT
);

-- Insert a test record
INSERT INTO connection_test (test_name, test_value) 
VALUES ('Connection Test', 'Successfully connected to Supabase at ' || NOW()::TEXT);

-- Verify the insert worked
SELECT * FROM connection_test;

-- Clean up (comment out if you want to keep the test table for verification)
-- DROP TABLE IF EXISTS connection_test;