import React, { useState, useEffect } from 'react';
import yaml from 'js-yaml';
import './App.css';

function App() {
  const [metricsData, setMetricsData] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [selectedAlgorithm, setSelectedAlgorithm] = useState(null);
  const [searchTerm, setSearchTerm] = useState('');
  const [sortConfig, setSortConfig] = useState({ key: null, direction: 'asc' });

  useEffect(() => {
    const loadYamlData = async () => {
      try {
        const response = await fetch('/benchmark_metrics.yaml');
        if (!response.ok) {
          throw new Error('Failed to fetch YAML file');
        }
        const yamlText = await response.text();
        const data = yaml.load(yamlText);
        setMetricsData(data);
        setLoading(false);
      } catch (err) {
        setError(err.message);
        setLoading(false);
      }
    };

    loadYamlData();
  }, []);

  const handleSort = (key) => {
    let direction = 'asc';
    if (sortConfig.key === key && sortConfig.direction === 'asc') {
      direction = 'desc';
    }
    setSortConfig({ key, direction });
  };

  const sortModels = (models) => {
    if (!sortConfig.key) return models;
    
    return [...models].sort((a, b) => {
      const aVal = a[sortConfig.key] || 0;
      const bVal = b[sortConfig.key] || 0;
      
      if (typeof aVal === 'number' && typeof bVal === 'number') {
        return sortConfig.direction === 'asc' ? aVal - bVal : bVal - aVal;
      }
      
      const aStr = String(aVal);
      const bStr = String(bVal);
      return sortConfig.direction === 'asc' 
        ? aStr.localeCompare(bStr)
        : bStr.localeCompare(aStr);
    });
  };

  const filteredAlgorithms = metricsData?.metrics?.mmdetection?.filter(algo => 
    algo.algorithm.toLowerCase().includes(searchTerm.toLowerCase()) ||
    algo.config.toLowerCase().includes(searchTerm.toLowerCase())
  ) || [];

  const getPaperForAlgorithm = (algorithmName) => {
    return metricsData?.papers?.find(paper => 
      paper.algorithm?.toLowerCase() === algorithmName?.toLowerCase()
    );
  };

  const getMetricKeys = (models) => {
    const keys = new Set();
    models.forEach(model => {
      Object.keys(model).forEach(key => {
        if (key !== 'model' && key !== 'style') {
          keys.add(key);
        }
      });
    });
    return Array.from(keys);
  };

  if (loading) {
    return (
      <div className="app">
        <div className="loading-container">
          <h2>Loading benchmark metrics...</h2>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="app">
        <div className="error-container">
          <h2>Error loading data</h2>
          <p>{error}</p>
        </div>
      </div>
    );
  }

  return (
    <div className="app">
      <header className="app-header">
        <h1>Benchmark Metrics Viewer</h1>
        <div className="summary">
          <div className="summary-card">
            <h3>MMDetection</h3>
            <p>Total Configs: {metricsData?.summary?.mmdetection?.total_configs || 0}</p>
            <p>Total Models: {metricsData?.summary?.mmdetection?.total_models || 0}</p>
          </div>
          <div className="summary-card">
            <h3>MMPose</h3>
            <p>Total Configs: {metricsData?.summary?.mmpose?.total_configs || 0}</p>
            <p>Total Models: {metricsData?.summary?.mmpose?.total_models || 0}</p>
          </div>
        </div>
      </header>

      <main className="main-content">
        <div className="controls">
          <input
            type="text"
            placeholder="Search algorithms..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="search-input"
          />
        </div>

        <div className="content-area">
          <div className="algorithms-list">
            <h2>Algorithms</h2>
            <div className="algorithm-cards">
              {filteredAlgorithms.map((algo, index) => (
                <div
                  key={index}
                  className={`algorithm-card ${selectedAlgorithm === algo ? 'selected' : ''}`}
                  onClick={() => setSelectedAlgorithm(algo)}
                >
                  <h3>{algo.algorithm}</h3>
                  <p className="config-name">{algo.config}</p>
                  <p className="model-count">{algo.models.length} models</p>
                </div>
              ))}
            </div>
          </div>

          {selectedAlgorithm && (
            <div className="model-details">
              <h2>{selectedAlgorithm.algorithm} Models</h2>
              
              {(() => {
                const paper = getPaperForAlgorithm(selectedAlgorithm.algorithm);
                return paper && (
                  <div className="paper-abstract">
                    <h3>{paper.title}</h3>
                    <p className="abstract-text">{paper.abstract}</p>
                  </div>
                );
              })()}
              
              <div className="table-container">
                <table className="models-table">
                  <thead>
                    <tr>
                      <th>Model</th>
                      <th>Style</th>
                      {getMetricKeys(selectedAlgorithm.models).map(key => (
                        <th 
                          key={key} 
                          onClick={() => handleSort(key)}
                          className="sortable"
                        >
                          {key.replace(/_/g, ' ')}
                          {sortConfig.key === key && (
                            <span className="sort-indicator">
                              {sortConfig.direction === 'asc' ? ' ↑' : ' ↓'}
                            </span>
                          )}
                        </th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    {sortModels(selectedAlgorithm.models).map((model, idx) => (
                      <tr key={idx}>
                        <td>{model.model}</td>
                        <td>{model.style || '-'}</td>
                        {getMetricKeys(selectedAlgorithm.models).map(key => (
                          <td key={key}>
                            {model[key] !== undefined ? (
                              typeof model[key] === 'number' 
                                ? model[key].toFixed(1)
                                : model[key]
                            ) : '-'}
                          </td>
                        ))}
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
              
              {selectedAlgorithm.models.some(m => m.box_AP) && (
                <div className="chart-container">
                  <h3>Performance Comparison</h3>
                  <div className="bar-chart">
                    {sortModels(selectedAlgorithm.models.filter(m => m.box_AP)).map((model, idx) => (
                      <div key={idx} className="bar-group">
                        <div 
                          className="bar"
                          style={{ 
                            height: `${(model.box_AP / 60) * 100}%`,
                            backgroundColor: `hsl(${200 + idx * 20}, 70%, 50%)`
                          }}
                        >
                          <span className="bar-value">{model.box_AP?.toFixed(1)}</span>
                        </div>
                        <div className="bar-label">{model.model}</div>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      </main>
    </div>
  );
}

export default App;