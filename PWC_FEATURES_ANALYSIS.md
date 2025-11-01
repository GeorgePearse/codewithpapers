# Papers with Code - Original Site Features

Analysis from archived site: https://web.archive.org/web/20240703005204/https://paperswithcode.com/

## Homepage Structure

### Navigation Bar
- **Logo**: PWC barcode-style logo
- **Search**: Global search with autocomplete
- **Main Nav Items**:
  - Browse State-of-the-Art
  - Datasets
  - Methods
  - More (Newsletter, RC2022, About, Trends, Portals, Libraries)
- **Right Side**:
  - Twitter link
  - Sign In

### Content Filters
- **Top** (Trending) - Default view
- **Latest** (New papers)
- **Greatest** (Most impactful)

### Paper Cards

Each paper card displays:

**Left Column (Image)**:
- Paper thumbnail/preview image
- Links to paper detail page

**Right Column (Content)**:
- **Title**: Large, linked to paper detail
- **Metadata Row**:
  - GitHub repository link (if available)
  - Framework badge (PyTorch, TensorFlow, JAX, etc.)
  - Publication date
- **Abstract**: Short excerpt (1-2 sentences)
- **Task Badges**: Visual badges showing ML tasks (e.g., "Representation Learning", "Image Classification")
- **SOTA Indicators**: If paper achieves state-of-the-art

## Key Features to Implement

### Phase 1: Core Display (Current)
- [x] Load papers from Postgres database
- [ ] Display paper cards in grid layout
- [ ] Show title, date, abstract
- [ ] GitHub links for implementations
- [ ] Framework badges

### Phase 2: Search & Filter
- [ ] Global search across papers
- [ ] Filter by trending/latest/greatest
- [ ] Filter by date range
- [ ] Filter by tasks/methods

### Phase 3: Browse SOTA
- [ ] Browse by task category
- [ ] Benchmark leaderboards
- [ ] Dataset pages

### Phase 4: Enhanced Features
- [ ] Paper detail pages
- [ ] Author pages
- [ ] Dataset pages
- [ ] Method pages
- [ ] User accounts (optional)

## Database Schema Mapping

Our schema → PWC display:

| Database Field | PWC Display |
|---------------|-------------|
| `papers.title` | Card title |
| `papers.abstract` | Card abstract |
| `papers.arxiv_id` | Link to arXiv |
| `papers.published_date` | Publication date |
| `papers.authors` | Author list (JSONB) |
| `implementations.github_url` | GitHub link |
| `implementations.framework` | Framework badge |
| `benchmarks` + `benchmark_results` | SOTA indicators |
| `paper_datasets` | Related datasets |

## Design Elements

### Colors
- Primary blue: #0d6efd
- Cyan/teal: #21cbce (PWC brand color)
- Gray text: #6c757d

### Typography
- Font: Lato (sans-serif)
- Headers: Bold
- Body: Regular weight

### Layout
- Container: Centered, max-width
- Cards: White background, subtle shadow
- Grid: Responsive (1 col mobile, 2-3 cols desktop)

## API Endpoints Needed

Using Supabase REST API:

```javascript
// Get recent papers
GET /rest/v1/papers?select=*,implementations(*)&order=published_date.desc&limit=20

// Search papers
GET /rest/v1/papers?title=ilike.*search*&select=*

// Get paper with all relations
GET /rest/v1/papers?id=eq.{uuid}&select=*,implementations(*),benchmark_results(benchmark(*))
```

## Implementation Plan

1. **Update React App** to fetch from Supabase instead of YAML
2. **Create Paper Card Component** matching PWC design
3. **Add Supabase Client** configuration
4. **Implement Search** and filtering
5. **Add Pagination** or infinite scroll
6. **Deploy** updated version to GitHub Pages

## Current Status

- Database: ✅ 30,000+ papers loaded and growing
- Schema: ✅ Complete with papers, implementations, benchmarks
- Frontend: 🔄 Needs rebuild to match PWC design
- API: 🔄 Need to configure Supabase client
