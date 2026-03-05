# StarReaper

**GitHub Signal Purification Engine**

StarReaper is a lightweight Rust CLI and TUI tool that detects and removes star-farming and follow-manipulation accounts from your GitHub followers using deterministic heuristic scoring.

It helps maintain signal integrity by automatically identifying suspicious follower behavior and optionally blocking flagged accounts.

---

## Why StarReaper?

Artificial engagement distorts trust signals on GitHub.
Common patterns include:
* “Star for star” or “follow back” bios
* Extremely skewed following/follower ratios
* Recently created accounts
* Zero public repositories
* High outbound following with no inbound credibility

StarReaper evaluates these signals and assigns a risk score.
You decide the enforcement threshold.
This is a hygiene tool — not a growth tool.

---

## Features

- **Heuristic Analysis**: Scores accounts based on multiple risk factors.
- **Interactive TUI Mode**: Review flagged accounts in a terminal UI before taking action.
- **Dry Run Mode**: Safely scan without blocking anyone.
- **Configurable Thresholds**: Adjust sensitivity to match your tolerance.
- **Safe Blocking**: No automatic blocking in TUI mode; explicit confirmation required.
- **Rate-aware request pacing**: Respects GitHub API limits.
- **Privacy First**: No database required, token is never logged, no third-party services.

---

## Installation

Ensure you have Rust installed (via [rustup](https://rustup.rs/)).

```bash
git clone https://github.com/ChronoCoders/starreaper.git
cd starreaper
cargo build --release
```

Binary will be located at `target/release/starreaper`.

---

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
# Linux / macOS
export GITHUB_PAT="your_token_here"

# Windows PowerShell
$env:GITHUB_PAT="your_token_here"
```

Then run:
```bash
cargo run -- --tui
```

### Options

| Option        | Description                    | Default |
| ------------- | ------------------------------ | ------- |
| `--token`     | GitHub Personal Access Token   | `env: GITHUB_PAT` |
| `--tui`       | Launch TUI mode                | `false` |
| `--threshold` | Minimum score to trigger block | `3`     |
| `--limit`     | Max followers scanned per run  | `200`   |
| `--dry-run`   | Audit only, no blocking        | `false` |

---

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

---

## TUI Controls

- `↑` / `↓`: Navigate the list of flagged accounts.
- `Enter`: Open block confirmation for the selected account.
- `Q`: Quit the application.

---

## Security

- Token is never logged.
- No token persistence.
- No external storage.
- No third-party services.

You remain fully in control.

---

## License

MIT

---

## Disclaimer

Heuristic detection is not perfect.
Always use `--dry-run` or the TUI mode to review flagged accounts before enforcement. Use responsibly.
