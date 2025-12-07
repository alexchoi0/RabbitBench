# RabbitBench

Continuous benchmarking platform for tracking performance over time. Catch performance regressions before they hit production.

## Overview

RabbitBench provides:

- **Performance Tracking**: Submit benchmark results and track them over time
- **Regression Detection**: Automatic alerts when performance degrades beyond thresholds
- **Multi-Project Support**: Manage benchmarks across multiple projects
- **Branch Comparison**: Compare performance across different branches
- **CI Integration**: Easy integration with CI/CD pipelines via CLI

## Architecture

```
rabbitbench/
├── cli/        # Rust CLI for submitting benchmarks
├── server/     # Next.js app (dashboard + GraphQL API)
└── web/        # Static landing page
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
rabbitbench auth login

# Or with API token
rabbitbench auth login --token rb_your_api_token
```

### 3. Create a Project

```bash
rabbitbench project create --slug my-project --name "My Project"
```

### 4. Submit Benchmarks

```bash
# Pipe benchmark output to the CLI
cargo bench | rabbitbench run \
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

### Server (`/server`)

Next.js application providing:
- Web dashboard for viewing benchmarks
- GraphQL API for CLI and integrations
- User authentication via GitHub OAuth
- Project and threshold management

See [Server README](server/README.md) for details.

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

      - name: Install RabbitBench CLI
        run: cargo install rabbitbench

      - name: Submit results
        run: |
          cargo bench | rabbitbench run \
            --project ${{ github.repository }} \
            --branch ${{ github.ref_name }} \
            --testbed github-actions
        env:
          RABBITBENCH_TOKEN: ${{ secrets.RABBITBENCH_TOKEN }}
```

## Hosting

RabbitBench uses Supabase for database and authentication:

- **Database**: Supabase PostgreSQL
- **Auth**: Supabase Auth with GitHub OAuth

See [Server README](server/README.md) for setup instructions.

## Tech Stack

| Component | Technology |
|-----------|------------|
| CLI | Rust, clap, reqwest |
| Server | Next.js 15, Prisma, GraphQL Yoga, pnpm |
| Database | PostgreSQL (Supabase) |
| Auth | Supabase Auth (GitHub OAuth) |
| Styling | Tailwind CSS, shadcn/ui |

## Inspiration

This project was inspired by [Bencher](https://bencher.dev) ([GitHub](https://github.com/bencherdev/bencher)), an excellent continuous benchmarking platform.

## License

MIT
