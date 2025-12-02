# CodeWithPapers

An interactive visualization tool for exploring machine learning papers and their relationships, aiming to provide a richer experience than traditional paper repositories.

## Overview

CodeWithPapers creates an interactive force-directed graph visualization of ML papers, showing connections between papers through citations, shared authors, and common techniques. The goal is to make research exploration more intuitive and to surface connections that might not be immediately apparent.

## Current Features

- **Interactive Graph Visualization**: Force-directed graph showing paper relationships
- **Real-time Search**: Filter papers by title, author, or keywords
- **Paper Details**: Click on any paper to see detailed information including:
  - Abstract
  - Authors and their affiliations
  - Publication venue and date
  - Keywords and techniques used
  - Connected papers
- **Responsive Design**: Collapsible sidebar for better space utilization

## Live Demo

The site is deployed at: **https://georgepearse.github.io/codewithpapers/**

## Running Locally

### Frontend (React/Vite)

```bash
npm install
npm run dev
```

Then open http://localhost:3000 in your browser.

### Deployment

The site automatically deploys to GitHub Pages when you push to `main`. See [DEPLOYMENT.md](DEPLOYMENT.md) for details.

### Python Scripts

This project uses [uv](https://github.com/astral-sh/uv) for fast, reliable Python dependency management.

**Install uv:**

```bash
curl -LsSf https://astral.sh/uv/install.sh | sh
```

**Install Python dependencies:**

```bash
uv sync
```

**Run Python scripts:**

```bash
# Download Papers with Code archive data
uv run python scripts/download_pwc_data.py --list

# Run other scripts
uv run python add_abstracts.py
```

## Data Format

Papers are currently stored in YAML format (`benchmark_metrics.yaml`). Each paper entry includes:

- Title, authors, abstract
- Publication details (venue, date)
- Keywords and techniques
- Connections to other papers
- Implementation links
- Benchmark results

## Roadmap / TODO

Architecture -> So what we could do, is have entries and edits submitted via GitHub, but then not stored with it, maybe it stays there for a few weeks, maybe it gets deleted as soon as ingested, tests check that it conforms to the required structure. Just chuck everything in postgres for now, it's a pretty RDBMS type problem, and can yeet anything else in JSONB very easily if needed, just give absolutely every table an overflow JSONB column and call it a day. The submissions should probably be YAML files.

Biggest concern is actually the amount it might be scraped and the hosting cost that will cause.

**Note**: I'd love to set up authentication to give others direct access to the API so that people can more easily share the DB contents. If you're interested in this or have suggestions, feel free to open an issue!

### Core Features

- [ ] **Multiple Implementation Links**
  - [ ] Official implementation (GitHub, GitLab, etc.)
  - [ ] Community replications with notes on differences
  - [ ] Framework-specific implementations (PyTorch, TensorFlow, JAX)
  - [ ] Simplified/educational implementations
  - [ ] Links to Colab/Kaggle notebooks
  - [ ] Links to the model weights etc. on hugging face.

### Training Metrics & Results

- [ ] **Training Run Visualization**
  - [ ] Loss curves (training/validation)
  - [ ] Learning rate schedules
  - [ ] Gradient norms and statistics
  - [ ] Hardware specifications and training time
  - [ ] Hyperparameter configurations
- [ ] **Benchmark Comparisons**
  - [ ] Interactive charts comparing multiple papers
  - [ ] Standardized metrics across papers
  - [ ] Confidence intervals and error bars
  - [ ] Computational efficiency metrics (FLOPs, parameters, inference time)
  - [ ] Inference speed

### Enhanced Paper Information

- [ ] **Code Quality Indicators**
  - [ ] Test coverage
  - [ ] Documentation quality score
  - [ ] Last commit/maintenance status
  - [ ] Number of open issues
  - [ ] Community engagement metrics (stars, forks, contributors)
- [ ] **Reproducibility Score**
  - [ ] Availability of pretrained models
  - [ ] Dataset accessibility
  - [ ] Environment setup complexity
  - [ ] Reported reproduction success rate

### Graph Enhancements

- [ ] **Advanced Filtering**
  - [ ] Filter by year range
  - [ ] Filter by venue/conference
  - [ ] Filter by performance metrics
  - [ ] Filter by implementation availability
- [ ] **Graph Analytics**
  - [ ] Most influential papers (PageRank)
  - [ ] Research clusters/communities
  - [ ] Trending topics over time
  - [ ] Gap analysis (under-explored connections)

### Collaboration Features

- [ ] **User Annotations**
  - [ ] Personal notes on papers ???
- [ ] **Community Features**
  - [ ] Discussion threads per paper <-- need to check that other website that does things like this.
  - [ ] Q&A section
  - [ ] Bug reports for implementations
  - [ ] Bounties for reproductions

### Analysis Tools

- [ ] **Paper Recommendations**
  - [ ] "Papers like this"

Could try to make this work, but honestly feels unlikely that it's worth it, at least not until everything else is working well https://github.com/paperswithcode/sota-extractor

Plan for the maintenance of the dataset right now, is to workout if something like GitHub can cope with the volume of data, but then sync it with postgres to run the backend on. Sounds like you can hit 100GB which is probably plenty? https://www.reddit.com/r/github/comments/xn8y97/is_there_a_limit_to_how_big_a_github_repo_can_be/ (for now anyway, for a first draft)

Definitely going to want to scrape https://web.archive.org/web/20250708172035/https://paperswithcode.com/task/representation-learning to really get this moving. Got to be services you can just pay to do it for you -> Try oxylabs??

There was definitely some other website like this https://www.connectedpapers.com/main/9397e7acd062245d37350f5c05faf56e9cfae0d6/DeepFruits:-A-Fruit-Detection-System-Using-Deep-Neural-Networks/graph but better, should check that you don't start feature creeping into this territory.

## Development

### Python Environment

This project uses `uv` for Python dependency management. The configuration is in `pyproject.toml`.

**Key commands:**

```bash
# Sync dependencies
uv sync

# Add a new dependency
uv add package-name

# Add a dev dependency
uv add --dev package-name

# Run a script
uv run python script.py

# Format code
uv run black .

# Lint code
uv run ruff check .
```

### JavaScript/Node Environment

Standard npm workflow for the React frontend:

```bash
npm install          # Install dependencies
npm run dev          # Start dev server
npm run build        # Build for production
npm run lint         # Lint code
npm run format       # Format code with prettier
```

## Contributing

This project is in early development. Contributions are welcome! Areas where help is particularly needed:

- Data collection and curation
- Frontend improvements and new visualizations
- API integrations
- Documentation and tutorials

## Vision

The ultimate goal is to create a comprehensive platform that:

1. Makes ML research more accessible and navigable
2. Provides deep insights into paper implementations and reproducibility
3. Facilitates collaboration and knowledge sharing
4. Helps researchers identify gaps and opportunities in the field

## Data Sources

### Papers with Code Archive (Hugging Face)

The project utilizes the Papers with Code archive hosted on Hugging Face, which contains the last publicly available snapshot of PWC datasets (retrieved July 28-29, 2025). All datasets are licensed under CC-BY-SA-4.0.

**Main Organization**: https://huggingface.co/pwc-archive

### Papers with Code Archive (Archive.org)

For the most recent archived snapshots of the Papers with Code website:

- **Archive.org snapshots**: https://web.archive.org/web/20241101000000*/paperswithcode.com
- **Latest SOTA snapshot**: https://web.archive.org/web/20250717073537/https://paperswithcode.com/sota

The goal is to rebuild Papers with Code using these archived datasets and web snapshots.

**Available Datasets**:

1. **[papers-with-abstracts](https://huggingface.co/datasets/pwc-archive/papers-with-abstracts)** (576k papers)
   - Complete paper metadata including titles, abstracts, authors, conference details, arXiv IDs, and associated methods
   - Primary source for paper information

2. **[links-between-paper-and-code](https://huggingface.co/datasets/pwc-archive/links-between-paper-and-code)** (300k links)
   - Connects academic papers to their GitHub repositories
   - Includes paper metadata, repository URLs, and flags for official implementations

3. **[datasets](https://huggingface.co/datasets/pwc-archive/datasets)** (15k datasets)
   - Information about ML datasets (MNIST, ImageNet, GLUE, CelebA, etc.)
   - Includes dataset descriptions, associated papers, tasks, modalities, and framework-specific data loaders

4. **[methods](https://huggingface.co/datasets/pwc-archive/methods)** (8.73k methods)
   - ML methods and techniques with metadata
   - Categorized by research areas (Computer Vision, NLP, RL, etc.)
   - Includes introduction year, source papers, and code snippets

5. **[evaluation-tables](https://huggingface.co/datasets/pwc-archive/evaluation-tables)** (2.25k tables)
   - Benchmark results and performance comparisons
   - State-of-the-art tracking across different tasks

6. **[files](https://huggingface.co/datasets/pwc-archive/files)** (62 items)
   - Additional archive files

## License

MIT
