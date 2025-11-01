import React, { useState, useEffect } from 'react';
import yaml from 'js-yaml';
import PapersView from './PapersView';
import DatasetsView from './DatasetsView';
import './App.css';

function App() {
  const [metricsData, setMetricsData] = useState(null);
  const [datasetsData, setDatasetsData] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [selectedAlgorithm, setSelectedAlgorithm] = useState(null);
  const [searchTerm, setSearchTerm] = useState('');
  const [sortConfig, setSortConfig] = useState({ key: null, direction: 'asc' });
  const [activeTab, setActiveTab] = useState('papers');

  useEffect(() => {
    const loadYamlData = async () => {
      try {
        // Load both YAML files - use BASE_URL for GitHub Pages compatibility
        const baseUrl = import.meta.env.BASE_URL;
        const [metricsResponse, datasetsResponse] = await Promise.all([
          fetch(`${baseUrl}benchmark_metrics.yaml`),
          fetch(`${baseUrl}datasets.yaml`)
        ]);
        
        if (!metricsResponse.ok || !datasetsResponse.ok) {
          throw new Error('Failed to fetch YAML files');
        }
        
        const [metricsText, datasetsText] = await Promise.all([
          metricsResponse.text(),
          datasetsResponse.text()
        ]);
        
        const metricsData = yaml.load(metricsText);
        const datasetsData = yaml.load(datasetsText);
        
        setMetricsData(metricsData);
        setDatasetsData(datasetsData);
        setLoading(false);
      } catch (err) {
        setError(err.message);
        setLoading(false);
      }
    };

    loadYamlData();
  }, []);


  const filteredAlgorithms = metricsData?.metrics?.mmdetection?.filter(algo => 
    algo.algorithm.toLowerCase().includes(searchTerm.toLowerCase()) ||
    algo.config.toLowerCase().includes(searchTerm.toLowerCase())
  ) || [];

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
        <h1>Papers with Code Browser</h1>
        <nav className="nav-tabs">
          <button 
            className={`tab-button ${activeTab === 'papers' ? 'active' : ''}`}
            onClick={() => setActiveTab('papers')}
          >
            Papers
          </button>
          <button 
            className={`tab-button ${activeTab === 'datasets' ? 'active' : ''}`}
            onClick={() => setActiveTab('datasets')}
          >
            Datasets
          </button>
        </nav>
        {activeTab === 'papers' && (
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
        )}
        {activeTab === 'datasets' && (
          <div className="summary">
            <div className="summary-card">
              <h3>Total Datasets</h3>
              <p>{datasetsData?.datasets?.length || 0} datasets available</p>
            </div>
          </div>
        )}
      </header>

      <main className="main-content">
        {activeTab === 'papers' && (
          <PapersView
            metricsData={metricsData}
            searchTerm={searchTerm}
            setSearchTerm={setSearchTerm}
            filteredAlgorithms={filteredAlgorithms}
            selectedAlgorithm={selectedAlgorithm}
            setSelectedAlgorithm={setSelectedAlgorithm}
            sortConfig={sortConfig}
            setSortConfig={setSortConfig}
          />
        )}
        {activeTab === 'datasets' && (
          <DatasetsView datasetsData={datasetsData} />
        )}
      </main>
    </div>
  );
}

export default App;