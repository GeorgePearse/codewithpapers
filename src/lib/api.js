// API client for the Rust backend
const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:8000';

// Helper function to fetch papers with all related data
// Supports new Tantivy search params: q, date_from, date_to
export async function fetchPapers({
  limit = 20,
  offset = 0,
  orderBy = 'published_date',
  order = 'desc',
  search = null,
  q = null,
  dateFrom = null,
  dateTo = null,
} = {}) {
  try {
    const params = new URLSearchParams({
      limit: limit.toString(),
      offset: offset.toString(),
      order_by: orderBy,
      order: order,
    });

    // Use q for Tantivy full-text search, fall back to search for legacy
    const searchQuery = q || search;
    if (searchQuery) {
      params.append('q', searchQuery);
    }

    // Date range filters for faceted search
    if (dateFrom) {
      params.append('date_from', dateFrom);
    }
    if (dateTo) {
      params.append('date_to', dateTo);
    }

    const response = await fetch(`${API_URL}/api/papers?${params}`);

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(errorData.error || `HTTP ${response.status}`);
    }

    const responseData = await response.json();

    // Handle new response format { papers, total_hits, facets }
    const papers = responseData.papers || responseData;
    const totalHits = responseData.total_hits || papers.length;
    const facets = responseData.facets || null;

    return {
      data: papers,
      totalHits,
      facets,
      error: null,
    };
  } catch (error) {
    console.error('Error fetching papers:', error);
    return { data: [], totalHits: 0, facets: null, error };
  }
}

// Helper to get a single paper with all details
export async function fetchPaper(paperId) {
  try {
    const response = await fetch(`${API_URL}/api/papers/${paperId}`);

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(errorData.error || `HTTP ${response.status}`);
    }

    const data = await response.json();
    return { data, error: null };
  } catch (error) {
    console.error('Error fetching paper:', error);
    return { data: null, error };
  }
}

// Fetch datasets
export async function fetchDatasets({
  limit = 20,
  offset = 0,
  search = null,
} = {}) {
  try {
    const params = new URLSearchParams({
      limit: limit.toString(),
      offset: offset.toString(),
    });

    if (search) {
      params.append('search', search);
    }

    const response = await fetch(`${API_URL}/api/datasets?${params}`);

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(errorData.error || `HTTP ${response.status}`);
    }

    const data = await response.json();
    return { data, error: null };
  } catch (error) {
    console.error('Error fetching datasets:', error);
    return { data: [], error };
  }
}

// Fetch benchmarks
export async function fetchBenchmarks({
  limit = 20,
  offset = 0,
  search = null,
} = {}) {
  try {
    const params = new URLSearchParams({
      limit: limit.toString(),
      offset: offset.toString(),
    });

    if (search) {
      params.append('search', search);
    }

    const response = await fetch(`${API_URL}/api/benchmarks?${params}`);

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(errorData.error || `HTTP ${response.status}`);
    }

    const data = await response.json();
    return { data, error: null };
  } catch (error) {
    console.error('Error fetching benchmarks:', error);
    return { data: [], error };
  }
}

// Fetch stats
export async function fetchStats() {
  try {
    const response = await fetch(`${API_URL}/api/stats`);

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(errorData.error || `HTTP ${response.status}`);
    }

    const data = await response.json();
    return { data, error: null };
  } catch (error) {
    console.error('Error fetching stats:', error);
    return { data: null, error };
  }
}

// Health check
export async function checkHealth() {
  try {
    const response = await fetch(`${API_URL}/api/health`);
    return response.ok;
  } catch {
    return false;
  }
}
