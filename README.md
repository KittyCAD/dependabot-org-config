# dependabot-org-config

A Rust CLI tool to manage and audit Dependabot configuration across all repositories in a GitHub organization.

## Features

- Scans all repositories in a GitHub organization for Dependabot configuration.
- Supports custom overrides and grouping of update rules.
- Can create or update Dependabot config files and optionally open pull requests.
- Provides options for caching, verbose output, and limiting to specific repositories.

## Usage

```sh
cargo run -- <ORG_NAME> [--ecosystems-cache <PATH>] [--dependabot-overrides <PATH>] [--create-pr] [--force-new] [--repo <REPO>] [--verbose]
```

- `<ORG_NAME>`: GitHub organization name (required)
- `--ecosystems-cache`: Optional path to cache ecosystems. This speeds up repeated runs by storing information about package ecosystems, reducing API calls to GitHub. **Note:** The cache can be slow to create on the first run, especially for large organizations.
- `--dependabot-overrides`: Optional path to a YAML file with custom Dependabot update rules. This allows you to override or supplement the default configuration for specific repositories or ecosystems.
- `--create-pr`: Create PRs for config changes
- `--force-new`: Force creation of new config
- `--repo`: Limit to specific repositories (repeatable)
- `--verbose`: Print verbose output


### Example

```
export GH_TOKEN=XXXX RUST_LOG=info 
cargo run -- KittyCAD --ecosystems-cache .ecosystems-cache.json --dependabot-overrides crates/dependabot/dependabot-overrides.toml --repo "$REPO"
rm .ecosystems-cache.json
```

Optionally append:

- `--create-pr` to create a PR
- `--force-new` to create a dependabot config if none exists

## Setup

1. Install Rust: https://rustup.rs/
2. Clone this repo and run:
   ```sh
   cargo build --release
   ```
3. Set a `GH_TOKEN` environment variable with a GitHub personal access token that has repo access.
4. Run the CLI as shown above.

**Note:** When using `--create-pr`, pull requests will be generated under the account associated with your `GH_TOKEN`.
