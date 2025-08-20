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

## Running Locally

```bash
npm install
npm run dev
```

Then open http://localhost:3000 in your browser.

## Data Format

Papers are currently stored in YAML format (`benchmark_metrics.yaml`). Each paper entry includes:

- Title, authors, abstract
- Publication details (venue, date)
- Keywords and techniques
- Connections to other papers
- Implementation links
- Benchmark results

## Roadmap / TODO

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

Definitely going to want to scrape https://web.archive.org/web/20250708172035/https://paperswithcode.com/task/representation-learning to really get this moving. Got to be services you can just pay to do it for you.

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

## License

MIT
