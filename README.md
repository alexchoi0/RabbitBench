# Driftwatch

Continuous benchmarking platform for tracking performance over time. Catch performance regressions before they hit production.

## Overview

Driftwatch provides:

- **Performance Tracking**: Submit benchmark results and track them over time
- **Regression Detection**: Automatic alerts when performance degrades beyond thresholds
- **Multi-Project Support**: Manage benchmarks across multiple projects
- **Branch Comparison**: Compare performance across different branches
- **CI Integration**: Easy integration with CI/CD pipelines via CLI

## Architecture

```
driftwatch/
├── cli/        # Rust CLI for submitting benchmarks
└── api/        # Rust HTTP API server
```

## Quick Start

### 1. Install the CLI

```bash
# From source
cargo install --path cli

# Or download from releases
```

### 2. Authenticate

```bash
# Browser login (recommended)
driftwatch auth login

# Or with API token
driftwatch auth login --token dw_your_api_token
```

### 3. Create a Project

```bash
driftwatch project create --slug my-project --name "My Project"
```

### 4. Submit Benchmarks

```bash
# Pipe benchmark output to the CLI
cargo bench | driftwatch run \
  --project my-project \
  --branch main \
  --testbed ci-linux
```

## Components

### CLI (`/cli`)

Rust command-line tool for:
- Authentication (browser OAuth or API token)
- Project management
- Submitting benchmark results
- Parsing Criterion output (more adapters coming)

See [CLI README](cli/README.md) for details.

### API (`/api`)

Rust HTTP API server providing:
- REST API for CLI and integrations
- User authentication
- Project and threshold management

See [API README](api/README.md) for details.

## CI Integration

### GitHub Actions

```yaml
name: Benchmarks

on:
  push:
    branches: [main]
  pull_request:

jobs:
  bench:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run benchmarks
        run: cargo bench -- --save-baseline main

      - name: Install Driftwatch CLI
        run: cargo install driftwatch

      - name: Submit results
        run: |
          cargo bench | driftwatch run \
            --project ${{ github.repository }} \
            --branch ${{ github.ref_name }} \
            --testbed github-actions
        env:
          DRIFTWATCH_TOKEN: ${{ secrets.DRIFTWATCH_TOKEN }}
```

## Tech Stack

| Component | Technology |
|-----------|------------|
| CLI | Rust, clap, reqwest |
| API | Rust, Axum, SQLx |
| Database | PostgreSQL |

## Inspiration

This project was inspired by [Bencher](https://bencher.dev) ([GitHub](https://github.com/bencherdev/bencher)), an excellent continuous benchmarking platform.

## License

MIT
