
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
