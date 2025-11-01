import { createClient } from '@supabase/supabase-js';

// For now, we'll use the direct connection since we're reading from a public database
// In production, you'd use Supabase's hosted service with proper authentication
const SUPABASE_URL = import.meta.env.VITE_SUPABASE_URL || 'https://placeholder.supabase.co';
const SUPABASE_ANON_KEY = import.meta.env.VITE_SUPABASE_ANON_KEY || 'placeholder-key';

export const supabase = createClient(SUPABASE_URL, SUPABASE_ANON_KEY);

// Helper function to fetch papers with all related data
export async function fetchPapers({
  limit = 20,
  offset = 0,
  orderBy = 'published_date',
  order = 'desc',
  search = null
} = {}) {
  let query = supabase
    .from('papers')
    .select(`
      *,
      implementations (
        github_url,
        framework,
        is_official,
        stars
      )
    `)
    .order(orderBy, { ascending: order === 'asc' })
    .range(offset, offset + limit - 1);

  // Add search filter if provided
  if (search) {
    query = query.or(`title.ilike.%${search}%,abstract.ilike.%${search}%`);
  }

  const { data, error } = await query;

  if (error) {
    console.error('Error fetching papers:', error);
    return { data: [], error };
  }

  return { data, error: null };
}

// Helper to get a single paper with all details
export async function fetchPaper(paperId) {
  const { data, error } = await supabase
    .from('papers')
    .select(`
      *,
      implementations (*),
      benchmark_results (
        *,
        benchmark (*)
      )
    `)
    .eq('id', paperId)
    .single();

  if (error) {
    console.error('Error fetching paper:', error);
    return { data: null, error };
  }

  return { data, error: null };
}
