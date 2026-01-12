# dependabot-org-config

A Rust CLI tool to manage and audit Dependabot configuration across all repositories in a GitHub organization.

## Features

- Scans all repositories in a GitHub organization for Dependabot configuration.
- Supports custom overrides and grouping of update rules.
- Can create or update Dependabot config files and optionally open pull requests under your account.
- Provides options for caching, verbose output, limiting to specific repositories, and processing only repos with existing PRs.

## Usage

```sh
cargo run -- <ORG_NAME> [--ecosystems-cache <PATH>] [--dependabot-overrides <PATH>] [--create-pr] [--force-new] [--repo <REPO>] [--verbose] [--only-existing]
```

- `<ORG_NAME>`: GitHub organization name (required)
- `--ecosystems-cache`: Optional path to cache ecosystems. This speeds up repeated runs by storing information about package ecosystems, reducing API calls to GitHub. **Note:** The cache can be slow to create on the first run, especially for large organizations.
- `--dependabot-overrides`: Optional path to a TOML file with custom Dependabot update rules. This allows you to override or supplement the default configuration for specific repositories or ecosystems.
- `--create-pr`: Create PRs for config changes (pull requests will be generated under your account, as determined by your `GH_TOKEN`)
- `--force-new`: Force creation of new config
- `--repo`: Limit to specific repositories (repeatable)
- `--verbose`: Print verbose output
- `--only-existing`: Only process repositories that already have an open PR for Dependabot config

### Example

```
export GH_TOKEN=XXXX RUST_LOG=info 
cargo run -- KittyCAD --ecosystems-cache .ecosystems-cache.json --dependabot-overrides overrides-sample.toml --repo "$REPO"
rm .ecosystems-cache.json
```

Optionally append:

- `--create-pr` to create a PR
- `--force-new` to create a dependabot config if none exists
- `--only-existing` to only process repos with an existing PR

## Setup

1. Install Rust: https://rustup.rs/
2. Clone this repo and run:
   ```sh
   cargo build --release
   ```
3. Set a `GH_TOKEN` environment variable with a GitHub personal access token that has repo access. **This is required.**
4. Run the CLI as shown above.

## Dependabot Overrides

You can provide a TOML file with custom update rules for specific repositories or ecosystems using the `--dependabot-overrides` flag. This allows you to override or supplement the default configuration. See `overrides-sample.toml` for an example format.

## Ecosystem Cache

The `--ecosystems-cache` option allows you to cache detected package ecosystems for all repositories. This can significantly speed up repeated runs, but note that the cache is slow to create initially, especially for large organizations.
