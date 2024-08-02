use std::fs;

use chrono::prelude::*;
use clap::Parser;
use color_eyre::eyre::{Ok, Result};
use etcetera::{choose_base_strategy, BaseStrategy};
use git2::Repository;
use git_tracker::Tracker;
use reqwest::blocking::Client;
use serde::Deserialize;
use tabled::{
	settings::{object::Rows, style::BorderSpanCorrection, Disable, Panel, Style},
	Table, Tabled,
};
use yansi::{hyperlink::HyperlinkExt, Paint};

#[derive(Clone, Debug, Deserialize)]
struct User {
	login: String,
	url: String,
}

#[derive(Clone, Debug, Deserialize)]
struct PullRequest {
	html_url: String,
	number: u64,
	title: String,
	user: User,
	created_at: DateTime<Utc>,
	merged_at: Option<DateTime<Utc>>,
	merged: bool,
	merge_commit_sha: Option<String>,
}

#[derive(Tabled, Debug)]
#[tabled(rename_all = "PascalCase")]
struct BranchStatus {
	branch: String,
	#[tabled(display_with = "display_branch_status")]
	has_pull_request: bool,
}

#[derive(Parser)]
#[command(version, about)]
struct Cli {
	pull_request: u64,

	// GitHub token
	#[clap(long, short, env = "GITHUB_TOKEN")]
	token: String,
}

fn display_branch_status(b: &bool) -> String {
	if *b { "✅" } else { "🚫" }.to_string()
}

fn main() -> Result<()> {
	let args = Cli::parse();
	color_eyre::install()?;

	let nixpkgs_path = choose_base_strategy()
		.unwrap()
		.cache_dir()
		.join("nixpkgs-track/nixpkgs");

	if !nixpkgs_path.exists() {
		println!("Nixpkgs tracker repository doesn't exist. Cloning...");
		fs::create_dir_all(&nixpkgs_path)?;
		Repository::clone("https://github.com/NixOS/nixpkgs", &nixpkgs_path)?;
	} else {
		println!("Nixpkgs tracker repository already exists. Updating...");
		let repo = Repository::open(&nixpkgs_path)?;
		repo.find_remote("origin")?
			.fetch(&["master"], None, None)?;
		repo.checkout_head(None)?;
	}

	let tracker = Tracker::from_path(&nixpkgs_path.to_str().unwrap())?;

	let client = Client::new();

	let url: String = format!("https://api.github.com/repos/nixos/nixpkgs/pulls/{}", args.pull_request);
	let pull_request = client
		.get(&url)
		.header("User-Agent", "nixpkgs-track")
		.header("Authorization", format!("Bearer {}", args.token))
		.send()?
		.json::<PullRequest>()?;

	let Some(commit_sha) = pull_request.merge_commit_sha else {
		println!("This pull request is very old. I can't track it!");
		return Ok(());
	};

	if pull_request.merged == false {
		println!("This pull request hasn't been merged yet!")
	} else {
		println!("Pull request was merged {}.", {
			let minutes_ago = Utc::now()
				.signed_duration_since(pull_request.merged_at.unwrap())
				.num_minutes();

			match minutes_ago {
				0 => "less than a minute ago".to_string(),
				1 => "1 minute ago".to_string(),
				m if m < 60 => format!("{} minutes ago", m),
				h if h < 120 => "1 hour ago".to_string(),         // 60 <= h < 120
				h if h < 1440 => format!("{} hours ago", h / 60), // 2 hours to 23 hours
				d if d < 2880 => "1 day ago".to_string(),         // 1440 <= d < 2880
				d => format!("{} days ago", d / 1440),            // 2 days and more
			}
		});

		let mut branch_statuses: Vec<BranchStatus> = vec![];

		for branch in &["master", "staging", "nixpkgs-unstable", "nixos-unstable-small", "nixos-unstable"] {
			let full_branch_name = format!("origin/{}", branch);
			let has_pull_request = tracker.branch_contains_sha(&full_branch_name, &commit_sha)?;

			branch_statuses.push(BranchStatus {
				branch: branch.to_string(),
				has_pull_request,
			});
		}

		let mut table = Table::new(branch_statuses);
		table
			.with(Style::modern())
			.with(Disable::row(Rows::first()))
			.with(Panel::header(format!(
				"{}",
				format!("{} - {}", format!("#{}", pull_request.number).bold(), pull_request.title).link(pull_request.html_url)
			)))
			.with(BorderSpanCorrection);
		println!("{}", table.to_string());
	}

	Ok(())
}
