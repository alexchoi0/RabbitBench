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
└── crates/
    ├── driftwatch-api/   # API server library
    └── driftwatch-cli/   # CLI binary (includes serve command)
```

## Quick Start

### 1. Install the CLI

```bash
# From source
cargo install --path crates/driftwatch-cli

# Or download from releases
```

### 2. Start the Server

```bash
# Start the API server
driftwatch serve --port 4000
```

### 3. Authenticate

```bash
# Browser login (recommended)
driftwatch auth login

# Or with API token
driftwatch auth login --token dw_your_api_token
```

### 4. Create a Project

```bash
driftwatch project create --slug my-project --name "My Project"
```

### 5. Submit Benchmarks

```bash
# Pipe benchmark output to the CLI
cargo bench | driftwatch run \
  --project my-project \
  --branch main \
  --testbed ci-linux
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `driftwatch serve` | Start the API server |
| `driftwatch auth login` | Authenticate via browser or token |
| `driftwatch auth status` | Show authentication status |
| `driftwatch auth logout` | Remove stored credentials |
| `driftwatch project list` | List all projects |
| `driftwatch project create` | Create a new project |
| `driftwatch project show` | Show project details |
| `driftwatch run` | Run benchmarks and submit results |

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

## Development

```bash
# Start database
make db-up

# Run development server
make dev

# Run tests
make test

# Build release
make build
```

## Tech Stack

| Component | Technology |
|-----------|------------|
| CLI | Rust, clap, reqwest |
| API | Rust, Axum, async-graphql, Sea-ORM |
| Database | PostgreSQL |

## Inspiration

This project was inspired by [Bencher](https://bencher.dev) ([GitHub](https://github.com/bencherdev/bencher)), an excellent continuous benchmarking platform.

## License

MIT
