# Driftwatch CLI

Command-line tool for submitting benchmark results to Driftwatch.

## Installation

### From source

```bash
cargo install --path cli
```

### Pre-built binary

Download from [releases](https://github.com/yourusername/driftwatch/releases) and add to your PATH.

## Authentication

### Browser login (recommended)

```bash
driftwatch auth login
```

This will:
1. Open your browser to the Driftwatch login page
2. After you authenticate, redirect back to the CLI
3. Save your credentials locally

```
$ driftwatch auth login

Opening browser for authentication...

If the browser doesn't open, visit this URL:
https://driftwatch.dev/cli-auth?callback=http%3A%2F%2F127.0.0.1%3A54321%2Fcallback

Waiting for authentication...
Validating token...

Authenticated as: user@example.com
Config saved to: ~/Library/Application Support/driftwatch/config.toml
```

### API token login

If you prefer, you can create an API token in your [dashboard settings](https://driftwatch.dev/settings) and use it directly:

```bash
driftwatch auth login --token dw_your_api_token_here
```

### Environment variables

You can also set credentials via environment variables:

```bash
export DRIFTWATCH_TOKEN=dw_your_api_token_here
export DRIFTWATCH_API_URL=https://driftwatch.dev  # optional, for self-hosted
```

### Check status

```bash
driftwatch auth status
```

### Logout

```bash
driftwatch auth logout
```

## Usage

### Submit benchmark results

Pipe your benchmark output to `driftwatch run`:

```bash
cargo bench -- --save-baseline main | driftwatch run \
  --project my-project \
  --branch main \
  --testbed local
```

Supported benchmark formats:
- Criterion (Rust)
- More coming soon

### Options

```
driftwatch run [OPTIONS]

Options:
  --project <SLUG>     Project slug (required)
  --branch <NAME>      Branch name (required)
  --testbed <NAME>     Testbed name (required)
  --git-hash <HASH>    Git commit hash (auto-detected if in git repo)
  --adapter <TYPE>     Benchmark adapter [default: criterion]
```

### List projects

```bash
driftwatch project list
```

### Create a project

```bash
driftwatch project create --slug my-project --name "My Project"
```

## Self-hosted instances

For self-hosted Driftwatch instances, specify the API URL:

```bash
# During login
driftwatch auth login --api-url https://your-instance.com

# Or via environment variable
export DRIFTWATCH_API_URL=https://your-instance.com
driftwatch auth login
```

## Configuration

Config is stored at:
- macOS: `~/Library/Application Support/driftwatch/config.toml`
- Linux: `~/.config/driftwatch/config.toml`
- Windows: `%APPDATA%\driftwatch\config.toml`

Example config:
```toml
token = "dw_your_api_token"
api_url = "https://driftwatch.dev"
```
