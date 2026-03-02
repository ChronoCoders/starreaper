use chrono::{DateTime, Days, Utc};
use clap::Parser;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "starreaper")]
#[command(about = "StarReaper — GitHub Signal Purification Engine")]
pub struct Args {
    #[arg(long, env = "GITHUB_PAT", help = "GitHub Personal Access Token")]
    pub token: String,

    #[arg(long, default_value = "3", help = "Minimum bot score to trigger block")]
    pub threshold: u32,

    #[arg(long, help = "Dry run — detect but do not block")]
    pub dry_run: bool,

    #[arg(long, default_value = "200", help = "Max followers to scan per run")]
    pub limit: u32,

    #[arg(long, help = "Launch TUI mode after scanning")]
    pub tui: bool,
}

#[derive(Debug, Deserialize)]
pub struct Follower {
    login: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UserProfile {
    pub login: String,
    pub bio: Option<String>,
    pub followers: u32,
    pub following: u32,
    pub public_repos: u32,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct BotScore {
    pub login: String,
    pub score: u32,
    pub reasons: Vec<String>,
    pub profile: UserProfile,
}

pub mod tui;

fn build_client(token: &str) -> reqwest::Client {
    let mut headers = HeaderMap::new();

    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("token {}", token)).expect("Invalid token format"),
    );

    headers.insert(USER_AGENT, HeaderValue::from_static("starreaper/0.2.0"));

    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/vnd.github+json"),
    );

    reqwest::Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(15))
        .build()
        .expect("Failed to build HTTP client")
}

async fn fetch_followers(client: &reqwest::Client, limit: u32) -> Vec<Follower> {
    let mut results = Vec::new();
    let mut page = 1;
    let per_page = 100;

    while results.len() < limit as usize {
        let url = format!(
            "https://api.github.com/user/followers?per_page={}&page={}",
            per_page, page
        );

        let resp = match client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[!] Error fetching followers: {}", e);
                break;
            }
        };

        if !resp.status().is_success() {
            eprintln!("[!] Failed to fetch followers: {}", resp.status());
            break;
        }

        let mut batch: Vec<Follower> = match resp.json().await {
            Ok(b) => b,
            Err(_) => break,
        };

        if batch.is_empty() {
            break;
        }

        results.append(&mut batch);
        page += 1;
    }

    results.truncate(limit as usize);
    results
}

async fn fetch_profile(client: &reqwest::Client, username: &str) -> Option<UserProfile> {
    let url = format!("https://api.github.com/users/{}", username);

    let resp = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[!] Error fetching {}: {}", username, e);
            return None;
        }
    };

    if !resp.status().is_success() {
        eprintln!(
            "[!] Failed to fetch profile for {}: {}",
            username,
            resp.status()
        );
        return None;
    }

    resp.json::<UserProfile>().await.ok()
}

fn score_profile(profile: &UserProfile) -> BotScore {
    let mut score = 0u32;
    let mut reasons = Vec::new();

    if let Some(bio) = &profile.bio {
        let bio_lower = bio.to_lowercase();
        let keywords = [
            "give me stars",
            "star back",
            "follow back",
            "star my repo",
            "star for star",
            "follow for follow",
            "f4f",
            "s4s",
        ];

        for keyword in keywords {
            if bio_lower.contains(keyword) {
                score += 3;
                reasons.push(format!("bio contains '{}'", keyword));
                break;
            }
        }
    }

    if profile.followers > 0
        && profile.following >= 50
        && profile.following >= profile.followers * 3
    {
        score += 2;
        reasons.push(format!(
            "suspicious ratio ({}/{})",
            profile.following, profile.followers
        ));
    }

    if profile.public_repos == 0 {
        score += 1;
        reasons.push("zero public repositories".to_string());
    }

    if let Some(created_at) = profile.created_at {
        let cutoff = Utc::now()
            .checked_sub_days(Days::new(90))
            .unwrap_or(Utc::now());

        if created_at > cutoff {
            score += 1;
            reasons.push(format!(
                "recent account ({})",
                created_at.format("%Y-%m-%d")
            ));
        }
    }

    if profile.followers == 0 && profile.following > 20 {
        score += 1;
        reasons.push("zero followers with active following".to_string());
    }

    BotScore {
        login: profile.login.clone(),
        score,
        reasons,
        profile: profile.clone(),
    }
}

pub async fn block_user(client: &reqwest::Client, username: &str) -> bool {
    let url = format!("https://api.github.com/user/blocks/{}", username);

    match client.put(&url).send().await {
        Ok(resp) => resp.status().as_u16() == 204,
        Err(e) => {
            eprintln!("[!] Failed to block {}: {}", username, e);
            false
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("================================================");
    println!("  StarReaper v0.3.0");
    println!("  GitHub Signal Purification Engine");
    println!("================================================\n");

    if args.dry_run {
        println!("[*] DRY RUN mode — no accounts will be blocked\n");
    }

    println!("[*] Threshold: {}", args.threshold);
    println!("[*] Scanning up to {} followers\n", args.limit);

    let client = build_client(&args.token);

    println!("[*] Fetching followers...");
    let followers = fetch_followers(&client, args.limit).await;
    println!("[+] Found {} followers\n", followers.len());

    let mut flagged_results = Vec::new();
    let mut blocked_count = 0u32;
    let mut clean_count = 0u32;
    let mut flagged_count = 0u32;

    for follower in &followers {
        if !args.tui {
            print!("[*] Scanning {}... ", follower.login);
        } else {
            use std::io::Write;
            print!(".");
            let _ = std::io::stdout().flush();
        }

        let Some(profile) = fetch_profile(&client, &follower.login).await else {
            if !args.tui {
                println!("skipped");
            }
            continue;
        };

        let result = score_profile(&profile);

        if result.score >= args.threshold {
            flagged_count += 1;

            if args.tui {
                flagged_results.push(result);
            } else {
                println!("FLAGGED (score: {})", result.score);

                for reason in &result.reasons {
                    println!("    - {}", reason);
                }

                if args.dry_run {
                    println!("    [DRY RUN] Would block {}", result.login);
                } else if block_user(&client, &result.login).await {
                    println!("    [+] Blocked {}", result.login);
                    blocked_count += 1;
                } else {
                    println!("    [!] Failed to block {}", result.login);
                }
            }
        } else {
            clean_count += 1;
            if !args.tui {
                println!("clean (score: {})", result.score);
            }
        }

        tokio::time::sleep(Duration::from_millis(400)).await;
    }

    if args.tui {
        let app = tui::AppState::new(
            flagged_results,
            followers.len() as u32,
            args.threshold,
            args.dry_run,
        );
        if let Err(e) = tui::run_tui(app, client).await {
            eprintln!("TUI Error: {}", e);
        }
    } else {
        println!("\n================================================");
        println!("  Summary");
        println!("================================================");
        println!("  Scanned  : {}", followers.len());
        println!("  Clean    : {}", clean_count);
        println!("  Flagged  : {}", flagged_count);

        if !args.dry_run {
            println!("  Blocked  : {}", blocked_count);
        }

        println!("================================================");
    }
}
