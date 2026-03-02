# StarReaper

**GitHub Signal Purification Engine**

StarReaper is a powerful tool designed to audit your GitHub followers, identify potential bot/spam accounts using heuristic analysis, and help you maintain a clean signal-to-noise ratio in your network.

## Features

- **Heuristic Analysis**: Scores accounts based on multiple risk factors:
  - Suspicious bio keywords (e.g., "follow for follow", "star for star").
  - High following-to-follower ratios.
  - Lack of public repositories.
  - Account age (new accounts are flagged).
  - Zero followers with active following activity.
- **Interactive TUI Mode**: Review flagged accounts in a terminal UI before taking action.
- **Dry Run Mode**: Safely scan without blocking anyone.
- **Configurable Thresholds**: Adjust sensitivity to match your tolerance.
- **Safe Blocking**: No automatic blocking in TUI mode; explicit confirmation required.

## Installation

Ensure you have Rust installed (via [rustup](https://rustup.rs/)).

```bash
git clone https://github.com/yourusername/starreaper.git
cd starreaper
cargo build --release
```

## Usage

You need a GitHub Personal Access Token (PAT) with `user` (specifically `read:user` and `user:follow` to manage blocks) permissions.

### CLI Arguments

```bash
# Basic usage (defaults to CLI output mode)
cargo run -- --token YOUR_GITHUB_PAT

# Launch interactive TUI mode (Recommended)
cargo run -- --tui --token YOUR_GITHUB_PAT

# Dry run (scan only, no blocking)
cargo run -- --dry-run --token YOUR_GITHUB_PAT

# Customize scan limit and sensitivity threshold
cargo run -- --limit 500 --threshold 4 --token YOUR_GITHUB_PAT
```

Alternatively, set the `GITHUB_PAT` environment variable:

```bash
export GITHUB_PAT="your_token_here"
cargo run -- --tui
```

### Options

- `--token <TOKEN>`: GitHub Personal Access Token (env: `GITHUB_PAT`).
- `--tui`: Launch the Terminal User Interface for interactive review.
- `--dry-run`: Detect suspicious accounts but do not perform any blocking actions.
- `--threshold <THRESHOLD>`: Minimum bot score to trigger a flag (default: 3).
- `--limit <LIMIT>`: Max followers to scan per run (default: 200).

## Scoring Logic

Accounts accumulate points based on the following criteria:

| Criteria | Score Impact |
|----------|--------------|
| **Bio Keywords** (e.g., "f4f", "give me stars") | +3 |
| **Suspicious Ratio** (Following > 3x Followers) | +2 |
| **Zero Public Repos** | +1 |
| **Recent Account** (< 90 days) | +1 |
| **Zero Followers** (with > 20 following) | +1 |

- **Score 3 (Medium)**: Flagged for review.
- **Score 4-5 (High)**: Strong likelihood of being a bot/spammer.
- **Score 6+ (Critical)**: Almost certainly a bot.

## TUI Controls

- `↑` / `↓`: Navigate the list of flagged accounts.
- `Enter`: Open block confirmation for the selected account.
- `Q`: Quit the application.

## License

MIT License
