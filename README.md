# StarReaper

**StarReaper** is a lightweight Rust CLI that detects and removes star-farming and follow-manipulation accounts from your GitHub followers using deterministic heuristic scoring.

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

* Deterministic heuristic scoring
* Transparent scoring reasons
* Dry-run mode (audit before enforcement)
* Pagination support
* Rate-aware request pacing
* No database required
* Standalone static binary
* Cross-platform (Windows / macOS / Linux)

---

## How It Works

Execution pipeline:

```
Fetch followers
→ Fetch profile data
→ Score heuristics
→ Flag accounts above threshold
→ Optionally block
```

### Default Scoring Model

| Signal                              | Score |
| ----------------------------------- | ----- |
| Bio contains star-farming keywords  | +3    |
| Suspicious following/follower ratio | +2    |
| Zero public repositories            | +1    |
| Account younger than 90 days        | +1    |
| Zero followers + active following   | +1    |

Default block threshold: `3`

---

## Installation

### 1. Clone the repository

```bash
git clone https://github.com/yourusername/starreaper.git
cd starreaper
```

### 2. Build

```bash
cargo build --release
```

Binary will be located at:

```
target/release/starreaper
```

On Windows:

```
target\release\starreaper.exe
```

---

## Usage

### Generate a GitHub Personal Access Token

Create a **classic PAT** with:

```
user
```

No repository access required.

---

### Set token

macOS / Linux:

```bash
export GITHUB_PAT=ghp_yourtoken
```

Windows:

```powershell
set GITHUB_PAT=ghp_yourtoken
```

---

### Audit Mode (Recommended First)

```bash
starreaper --dry-run
```

Shows:

* Username
* Risk score
* Exact scoring reasons

No accounts are blocked.

---

### Enforcement Mode

```bash
starreaper --threshold 3
```

Optional parameters:

```
--threshold <number>
--limit <number>
--dry-run
```

Example:

```bash
starreaper --threshold 4 --limit 500
```

---

## Configuration

| Option        | Description                    | Default |
| ------------- | ------------------------------ | ------- |
| `--threshold` | Minimum score to trigger block | 3       |
| `--limit`     | Max followers scanned per run  | 200     |
| `--dry-run`   | Audit only, no blocking        | false   |

---

## Rate Limiting

GitHub authenticated limit: 5,000 requests/hour.

StarReaper:

* Uses pagination
* Inserts request delays
* Is intended for periodic execution (daily / weekly), not continuous polling

---

## Security

* Token is never logged
* No token persistence
* No external storage
* No third-party services

You remain fully in control.

---

## Philosophy

StarReaper does not inflate metrics.
It removes manipulation.

The goal is clean signal:

* Authentic followers
* Honest engagement
* Trustworthy reputation

---

## Roadmap (Possible Extensions)

* Whitelist support
* JSON output mode
* GitHub Action integration
* Scheduled automation
* Cross-platform reputation scoring

---

## License

MIT

---

## Disclaimer

Heuristic detection is not perfect.
Always use `--dry-run` before enforcement to review flagged accounts.

Use responsibly.
