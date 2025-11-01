import React, { useState, useEffect } from 'react';
import { fetchPapers } from './lib/supabase';
import PaperCard from './components/PaperCard';
import './App.css';

function App() {
  const [papers, setPapers] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [filter, setFilter] = useState('latest'); // 'top', 'latest', 'greatest'
  const [searchTerm, setSearchTerm] = useState('');
  const [searchInput, setSearchInput] = useState('');
  const [activeView, setActiveView] = useState('papers'); // 'papers' or 'sota'

  useEffect(() => {
    loadPapers();
  }, [filter, searchTerm]);

  async function loadPapers() {
    setLoading(true);
    setError(null);

    try {
      const options = {
        limit: 20,
        offset: 0,
        orderBy: 'published_date',
        order: 'desc',
        search: searchTerm || null
      };

      // Adjust ordering based on filter
      if (filter === 'top') {
        options.orderBy = 'created_at';
        options.order = 'desc';
      } else if (filter === 'latest') {
        options.orderBy = 'published_date';
        options.order = 'desc';
      } else if (filter === 'greatest') {
        options.orderBy = 'published_date';
        options.order = 'asc';
      }

      const { data, error: fetchError } = await fetchPapers(options);

      if (fetchError) {
        throw fetchError;
      }

      setPapers(data || []);
    } catch (err) {
      console.error('Error loading papers:', err);
      setError(err.message || 'Failed to load papers from database');
    } finally {
      setLoading(false);
    }
  }

  function handleSearch(e) {
    e.preventDefault();
    setSearchTerm(searchInput);
  }

  return (
    <div className="app">
      {/* Header */}
      <header className="pwc-header">
        <div className="container">
          <nav className="navbar">
            <div className="navbar-brand">
              <h1 className="logo">Papers with Code</h1>
            </div>
            <div className="navbar-menu">
              <button
                className={`nav-link ${activeView === 'papers' ? 'active' : ''}`}
                onClick={() => setActiveView('papers')}
              >
                Papers
              </button>
              <button
                className={`nav-link ${activeView === 'sota' ? 'active' : ''}`}
                onClick={() => setActiveView('sota')}
              >
                Browse State-of-the-Art
              </button>
            </div>
            <div className="navbar-search">
              <form onSubmit={handleSearch}>
                <input
                  type="search"
                  placeholder="Search papers..."
                  value={searchInput}
                  onChange={(e) => setSearchInput(e.target.value)}
                  className="search-input"
                />
              </form>
            </div>
          </nav>
        </div>
      </header>

      {/* Main Content */}
      <main className="container">
        {activeView === 'papers' && (
          <>
            {/* Filter Badges */}
            <div className="content-header">
              <div className="filter-badges">
                <button
                  className={`filter-badge ${filter === 'top' ? 'active' : ''}`}
                  onClick={() => setFilter('top')}
                >
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <path d="M22 12h-4l-3 9L9 3l-3 9H2"/>
                  </svg>
                  Top
                </button>
                <button
                  className={`filter-badge ${filter === 'latest' ? 'active' : ''}`}
                  onClick={() => setFilter('latest')}
                >
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/>
                  </svg>
                  Latest
                </button>
                <button
                  className={`filter-badge ${filter === 'greatest' ? 'active' : ''}`}
                  onClick={() => setFilter('greatest')}
                >
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <path d="M6 9H4.5a2.5 2.5 0 0 1 0-5H6"/>
                    <path d="M18 9h1.5a2.5 2.5 0 0 0 0-5H18"/>
                    <path d="M4 22h16"/>
                    <path d="M10 14.66V17c0 .55-.47.98-.97 1.21C7.85 18.75 7 20.24 7 22"/>
                    <path d="M14 14.66V17c0 .55.47.98.97 1.21C16.15 18.75 17 20.24 17 22"/>
                    <path d="M18 2H6v7a6 6 0 0 0 12 0V2Z"/>
                  </svg>
                  Greatest
                </button>
              </div>

              <h2 className="content-title">
                {filter === 'top' && 'Trending Research'}
                {filter === 'latest' && 'Latest Papers'}
                {filter === 'greatest' && 'Classic Papers'}
              </h2>
            </div>

        {/* Papers List */}
        {loading && (
          <div className="loading-container">
            <div className="loading-spinner"></div>
            <p>Loading papers from database...</p>
          </div>
        )}

        {error && (
          <div className="error-container">
            <h3>Database Connection Error</h3>
            <p>{error}</p>
            <p className="error-hint">
              Configure Supabase credentials in .env.local:
              <br />
              <code>VITE_SUPABASE_URL=your-project-url</code>
              <br />
              <code>VITE_SUPABASE_ANON_KEY=your-anon-key</code>
            </p>
            <p style={{ marginTop: '12px', fontSize: '0.875rem' }}>
              See <a href="https://github.com/GeorgePearse/codewithpapers#setup" target="_blank" rel="noopener noreferrer">setup instructions</a> for details.
            </p>
          </div>
        )}

        {!loading && !error && papers.length === 0 && (
          <div className="empty-state">
            <p>No papers found{searchTerm && ` for "${searchTerm}"`}</p>
          </div>
        )}

        {!loading && !error && papers.length > 0 && (
          <div className="papers-list">
            {papers.map((paper) => (
              <PaperCard key={paper.id} paper={paper} />
            ))}
          </div>
        )}

        {!loading && !error && papers.length > 0 && (
          <div className="load-more">
            <p className="papers-count">
              Showing {papers.length} papers
            </p>
          </div>
        )}
          </>
        )}

        {/* Browse State-of-the-Art View */}
        {activeView === 'sota' && (
          <div className="sota-view">
            <div className="content-header">
              <h2 className="content-title">Browse State-of-the-Art</h2>
              <p className="content-subtitle">Explore cutting-edge results across different tasks and benchmarks</p>
            </div>

            <div className="sota-categories">
              <div className="sota-category-card">
                <div className="sota-category-icon">🖼️</div>
                <h3>Computer Vision</h3>
                <p>Image Classification, Object Detection, Segmentation, Pose Estimation</p>
                <div className="sota-stats">
                  <span>{papers.length > 0 ? Math.floor(papers.length * 0.4) : 0} papers</span>
                </div>
              </div>

              <div className="sota-category-card">
                <div className="sota-category-icon">💬</div>
                <h3>Natural Language Processing</h3>
                <p>Language Modeling, Text Classification, Question Answering, Translation</p>
                <div className="sota-stats">
                  <span>{papers.length > 0 ? Math.floor(papers.length * 0.3) : 0} papers</span>
                </div>
              </div>

              <div className="sota-category-card">
                <div className="sota-category-icon">🎯</div>
                <h3>Reinforcement Learning</h3>
                <p>Game AI, Robotics, Control, Multi-Agent Systems</p>
                <div className="sota-stats">
                  <span>{papers.length > 0 ? Math.floor(papers.length * 0.15) : 0} papers</span>
                </div>
              </div>

              <div className="sota-category-card">
                <div className="sota-category-icon">📊</div>
                <h3>Graphs & Structured Data</h3>
                <p>Graph Neural Networks, Knowledge Graphs, Recommendation Systems</p>
                <div className="sota-stats">
                  <span>{papers.length > 0 ? Math.floor(papers.length * 0.15) : 0} papers</span>
                </div>
              </div>

              <div className="sota-category-card">
                <div className="sota-category-icon">🔊</div>
                <h3>Audio & Speech</h3>
                <p>Speech Recognition, Audio Classification, Text-to-Speech</p>
                <div className="sota-stats">
                  <span>{papers.length > 0 ? Math.floor(papers.length * 0.1) : 0} papers</span>
                </div>
              </div>

              <div className="sota-category-card">
                <div className="sota-category-icon">🧠</div>
                <h3>General AI</h3>
                <p>Meta-Learning, Transfer Learning, Few-Shot Learning</p>
                <div className="sota-stats">
                  <span>{papers.length > 0 ? Math.floor(papers.length * 0.1) : 0} papers</span>
                </div>
              </div>
            </div>
          </div>
        )}
      </main>

      {/* Footer */}
      <footer className="pwc-footer">
        <div className="container">
          <p>
            Rebuilding Papers with Code • {papers.length > 0 ? `${papers.length.toLocaleString()}+ papers` : 'Loading database'} • Data from{' '}
            <a href="https://huggingface.co/datasets/pwc-archive" target="_blank" rel="noopener noreferrer">
              PWC Archive
            </a>
            {' '}via Supabase
          </p>
        </div>
      </footer>
    </div>
  );
}

export default App;
