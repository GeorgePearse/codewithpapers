import React, { useState } from 'react';

function DatasetsView({ datasetsData }) {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedDataset, setSelectedDataset] = useState(null);
  const [categoryFilter, setCategoryFilter] = useState('all');

  // Get unique categories from all datasets
  const getAllCategories = () => {
    const categories = new Set();
    datasetsData?.datasets?.forEach(dataset => {
      dataset.categories?.forEach(cat => categories.add(cat));
    });
    return Array.from(categories).sort();
  };

  // Filter datasets based on search and category
  const filteredDatasets = datasetsData?.datasets?.filter(dataset => {
    const matchesSearch = dataset.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
                          dataset.description.toLowerCase().includes(searchTerm.toLowerCase());
    const matchesCategory = categoryFilter === 'all' || 
                           dataset.categories?.includes(categoryFilter);
    return matchesSearch && matchesCategory;
  }) || [];

  // Format number with commas
  const formatNumber = (num) => {
    return num?.toLocaleString() || '0';
  };

  return (
    <div className="datasets-view">
      <div className="controls">
        <input
          type="text"
          placeholder="Search datasets..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          className="search-input"
        />
        <select 
          className="category-filter"
          value={categoryFilter}
          onChange={(e) => setCategoryFilter(e.target.value)}
        >
          <option value="all">All Categories</option>
          {getAllCategories().map(cat => (
            <option key={cat} value={cat}>
              {cat.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase())}
            </option>
          ))}
        </select>
      </div>

      <div className="datasets-grid">
        {filteredDatasets.map((dataset, index) => (
          <div
            key={index}
            className={`dataset-card ${selectedDataset === dataset ? 'selected' : ''}`}
            onClick={() => setSelectedDataset(selectedDataset === dataset ? null : dataset)}
          >
            <h3>{dataset.name}</h3>
            <p className="dataset-description">{dataset.description}</p>
            
            <div className="dataset-stats">
              <div className="stat">
                <span className="stat-label">Papers:</span>
                <span className="stat-value">{formatNumber(dataset.papers_count)}</span>
              </div>
              <div className="stat">
                <span className="stat-label">Benchmarks:</span>
                <span className="stat-value">{formatNumber(dataset.benchmarks_count)}</span>
              </div>
            </div>

            <div className="dataset-categories">
              {dataset.categories?.slice(0, 3).map((cat, idx) => (
                <span key={idx} className="category-tag">
                  {cat.replace(/_/g, ' ')}
                </span>
              ))}
              {dataset.categories?.length > 3 && (
                <span className="category-tag more">+{dataset.categories.length - 3}</span>
              )}
            </div>

            {dataset.url && (
              <a 
                href={dataset.url} 
                target="_blank" 
                rel="noopener noreferrer"
                className="dataset-link"
                onClick={(e) => e.stopPropagation()}
              >
                View Dataset →
              </a>
            )}
          </div>
        ))}
      </div>

      {selectedDataset && (
        <div className="dataset-modal-overlay" onClick={() => setSelectedDataset(null)}>
          <div className="dataset-modal" onClick={(e) => e.stopPropagation()}>
            <button className="close-button" onClick={() => setSelectedDataset(null)}>×</button>
            
            <h2>{selectedDataset.name}</h2>
            <p className="modal-description">{selectedDataset.description}</p>
            
            <div className="modal-stats">
              <div className="stat-card">
                <h4>Papers Using This Dataset</h4>
                <p className="stat-number">{formatNumber(selectedDataset.papers_count)}</p>
              </div>
              <div className="stat-card">
                <h4>Benchmarks</h4>
                <p className="stat-number">{formatNumber(selectedDataset.benchmarks_count)}</p>
              </div>
            </div>

            <div className="modal-categories">
              <h4>Categories</h4>
              <div className="categories-list">
                {selectedDataset.categories?.map((cat, idx) => (
                  <span key={idx} className="category-tag large">
                    {cat.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase())}
                  </span>
                ))}
              </div>
            </div>

            {selectedDataset.url && (
              <div className="modal-actions">
                <a 
                  href={selectedDataset.url} 
                  target="_blank" 
                  rel="noopener noreferrer"
                  className="primary-button"
                >
                  Visit Dataset Website
                </a>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

export default DatasetsView;