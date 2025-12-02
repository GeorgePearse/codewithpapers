# Paper Submissions

Submit papers to CodeWithPapers by creating a YAML file in this directory.

## Quick Start

1. Copy the example template:

   ```bash
   cp submissions/example.yaml submissions/my-paper.yaml
   ```

2. Fill in your paper details (see schema below)

3. Validate locally (optional but recommended):

   ```bash
   # If you have Rust installed:
   cargo run --manifest-path backend/Cargo.toml --bin validate_submission -- submissions/my-paper.yaml

   # Or use pre-commit hooks:
   pre-commit run validate-submissions --files submissions/my-paper.yaml
   ```

4. Submit a Pull Request

5. CI will validate your submission automatically. Once merged, the data is inserted into the database and the YAML file is deleted.

## File Naming

Name your file descriptively:

- `2301.12345-attention-is-all-you-need.yaml`
- `1706.03762-transformers.yaml`

## Schema

### Required Fields

```yaml
paper:
  title: 'Your Paper Title' # 5-500 characters
  arxiv_id: '2301.12345' # Format: YYMM.NNNNN or YYMM.NNNNNvN
```

### Optional Fields

```yaml
paper:
  abstract: |
    Your abstract here...
  arxiv_url: 'https://arxiv.org/abs/2301.12345' # Auto-generated if omitted
  pdf_url: 'https://arxiv.org/pdf/2301.12345.pdf' # Auto-generated if omitted
  published_date: '2023-01-15' # YYYY-MM-DD format
  authors:
    - 'Author One'
    - 'Author Two'

implementations:
  - github_url: 'https://github.com/org/repo'
    framework: 'pytorch' # pytorch, tensorflow, jax, keras, sklearn, other
    is_official: true
    stars: 1000 # Optional, updated automatically later

benchmark_results:
  - dataset_name: 'ImageNet'
    task: 'Image Classification'
    metric_name: 'top1_accuracy'
    metric_value: 85.6
    extra_data: # Optional additional context
      model_size: '86M params'
```

## Valid Frameworks

- `pytorch`
- `tensorflow`
- `jax`
- `keras`
- `sklearn`
- `other`

## Common Errors

### Invalid arXiv ID

```
ERROR: paper.arxiv_id: Invalid arXiv ID format
```

Use just the ID, not the full URL:

- :white_check_mark: `"2301.12345"`
- :white_check_mark: `"2301.12345v2"`
- :x: `"https://arxiv.org/abs/2301.12345"`

### Unknown Field

```
ERROR: yaml: unknown field 'titl'
```

Check for typos in field names. Use `title`, not `titl`.

### Missing Required Field

```
ERROR: paper.title: Field required
```

Ensure both `title` and `arxiv_id` are present.

## Questions?

Open an issue if you need help with your submission.
