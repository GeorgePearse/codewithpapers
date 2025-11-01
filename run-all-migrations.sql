-- Combined Migration Script
-- Generated: 2025-08-20T21:22:33.506Z
-- Run this in your Supabase SQL Editor


-- ============================================
-- Migration: 001_initial_schema.sql
-- ============================================

-- Initial schema for Papers with Code database

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Papers table
CREATE TABLE IF NOT EXISTS papers (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    title TEXT NOT NULL,
    abstract TEXT,
    arxiv_id VARCHAR(20),
    arxiv_url TEXT,
    pdf_url TEXT,
    published_date DATE,
    authors JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(arxiv_id)
);

-- Datasets table
CREATE TABLE IF NOT EXISTS datasets (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    modalities TEXT[],
    task_categories TEXT[],
    languages TEXT[],
    size VARCHAR(50),
    homepage_url TEXT,
    github_url TEXT,
    paper_url TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Benchmarks table
CREATE TABLE IF NOT EXISTS benchmarks (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    name TEXT NOT NULL,
    dataset_id UUID REFERENCES datasets(id) ON DELETE CASCADE,
    task TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name, dataset_id)
);

-- Code implementations table
CREATE TABLE IF NOT EXISTS implementations (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    paper_id UUID REFERENCES papers(id) ON DELETE CASCADE,
    github_url TEXT NOT NULL,
    framework VARCHAR(50),
    stars INTEGER,
    is_official BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Benchmark results table
CREATE TABLE IF NOT EXISTS benchmark_results (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    paper_id UUID REFERENCES papers(id) ON DELETE CASCADE,
    benchmark_id UUID REFERENCES benchmarks(id) ON DELETE CASCADE,
    implementation_id UUID REFERENCES implementations(id) ON DELETE SET NULL,
    metric_name VARCHAR(100) NOT NULL,
    metric_value NUMERIC NOT NULL,
    extra_data JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(paper_id, benchmark_id, metric_name)
);

-- Paper-Dataset relationship table (many-to-many)
CREATE TABLE IF NOT EXISTS paper_datasets (
    paper_id UUID REFERENCES papers(id) ON DELETE CASCADE,
    dataset_id UUID REFERENCES datasets(id) ON DELETE CASCADE,
    PRIMARY KEY (paper_id, dataset_id)
);

-- Create indexes for better query performance
CREATE INDEX idx_papers_arxiv_id ON papers(arxiv_id);
CREATE INDEX idx_papers_published_date ON papers(published_date);
CREATE INDEX idx_implementations_paper_id ON implementations(paper_id);
CREATE INDEX idx_benchmark_results_paper_id ON benchmark_results(paper_id);
CREATE INDEX idx_benchmark_results_benchmark_id ON benchmark_results(benchmark_id);

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers for updated_at columns
CREATE TRIGGER update_papers_updated_at BEFORE UPDATE ON papers
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_datasets_updated_at BEFORE UPDATE ON datasets
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_benchmarks_updated_at BEFORE UPDATE ON benchmarks
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_implementations_updated_at BEFORE UPDATE ON implementations
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

