use std::fs;

use clap::{Parser, Subcommand};
use color_eyre::eyre::{Ok, Result};
use serde::{Deserialize, Serialize};

use chrono::Utc;
use etcetera::{choose_base_strategy, BaseStrategy};
use nixpkgs_track::{branch_contains_commit, fetch_nixpkgs_pull_request, utils::format_seconds_to_time_ago};
use yansi::hyperlink::HyperlinkExt;

static DEFAULT_BRANCHES: [&str; 6] = ["master", "staging", "staging-next", "nixpkgs-unstable", "nixos-unstable-small", "nixos-unstable"];

#[derive(Parser)]
#[command(version, about, subcommand_negates_reqs = true)]
struct Cli {
	#[clap(required(true))]
	pull_request: Option<u64>,

	#[command(subcommand)]
	command: Option<Commands>,

	/// GitHub token
	#[clap(long, short, env = "GITHUB_TOKEN")]
	token: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
	/// Add a pull request to tracking list
	Add { pull_requests: Vec<u64> },
	/// Remove a pull request from the tracking list
	Remove {
		pull_requests: Vec<u64>,

		#[clap(long)]
		all: bool,
	},
	/// List tracked pull requests
	List {
		#[clap(long)]
		json: bool,
	},
	/// Check tracked pull requests
	Check {},
}

#[derive(Serialize, Deserialize)]
struct Cache {
	pull_requests: Vec<u64>,
}

impl Cache {
	fn new() -> Self {
		return Cache { pull_requests: vec![] };
	}
}

async fn check(pull_request: u64, token: Option<&str>) -> Result<()> {
	let pull_request = fetch_nixpkgs_pull_request(pull_request, token).await?;

	let Some(commit_sha) = pull_request.merge_commit_sha else {
		println!("This pull request is very old. I can't track it!");
		return Ok(());
	};

	println!(
		"[{}] {}",
		pull_request.number,
		pull_request
			.title
			.link(pull_request.html_url)
	);

	if pull_request.merged == false {
		let created_at_ago = format_seconds_to_time_ago(
			Utc::now()
				.signed_duration_since(pull_request.created_at)
				.num_seconds()
				.try_into()?,
		);
		let created_at_date = pull_request.created_at.to_rfc3339();

		println!("This pull request hasn't been merged yet!");
		println!("Created {} ago ({}).", created_at_ago, created_at_date);
	} else {
		let merged_at_ago = format_seconds_to_time_ago(
			Utc::now()
				.signed_duration_since(pull_request.merged_at.unwrap())
				.num_seconds()
				.try_into()?,
		);
		let merged_at_date = pull_request
			.merged_at
			.unwrap()
			.to_rfc3339();
		let creation_to_merge_time = format_seconds_to_time_ago(
			pull_request
				.merged_at
				.unwrap()
				.signed_duration_since(pull_request.created_at)
				.num_seconds()
				.try_into()?,
		);

		println!("Merged {} ago ({}), {} after creation.", merged_at_ago, merged_at_date, creation_to_merge_time);

		let branches = DEFAULT_BRANCHES
			.iter()
			.map(|branch| {
				let token_clone = token.map(|t| t.to_owned());
				let branch_clone = branch.to_string();
				let commit_sha_clone = commit_sha.clone();

				tokio::spawn(async move { branch_contains_commit(&branch_clone, &commit_sha_clone, token_clone.as_deref()).await })
			})
			.collect::<Vec<_>>();

		for (i, branch) in branches.into_iter().enumerate() {
			let has_pull_request = branch.await??;

			println!("{}: {}", DEFAULT_BRANCHES[i], if has_pull_request { "✅" } else { "🚫" });
		}
	}

	Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
	let args = Cli::parse();
	color_eyre::install()?;

	let cache_dir = choose_base_strategy()?
		.cache_dir()
		.join("nixpkgs-track");
	if !cache_dir.exists() {
		fs::create_dir_all(&cache_dir)?;
	}

	let cache = cache_dir.join("cache.json");

	let mut cache_data: Cache = if cache.exists() { serde_json::from_str(&fs::read_to_string(&cache)?)? } else { Cache::new() };

	match args.command {
		Some(Commands::Add { pull_requests }) => {
			cache_data
				.pull_requests
				.extend(pull_requests);
			cache_data.pull_requests.sort_unstable();
			cache_data.pull_requests.dedup();
		}
		Some(Commands::Remove { pull_requests, all }) => {
			if all {
				cache_data.pull_requests.clear();
			} else {
				cache_data.pull_requests = cache_data
					.pull_requests
					.into_iter()
					.filter(|x| !pull_requests.contains(x))
					.collect()
			}
		}
		Some(Commands::List { json }) => {
			if cache_data.pull_requests.len() == 0 {
				println!("No pull requests saved.");
				return Ok(());
			}

			#[derive(Serialize, Deserialize)]
			struct TrackedPullRequest {
				id: u64,
				title: String,
				url: String,
			}

			impl TrackedPullRequest {
				async fn new(id: u64, token: Option<&str>) -> Result<Self> {
					let data = fetch_nixpkgs_pull_request(id, token.as_deref()).await?;

					Ok(TrackedPullRequest {
						id,
						title: data.title,
						url: data.html_url,
					})
				}
			}
			let mut pull_requests = Vec::new();

			for &pr in &cache_data.pull_requests {
				let tracked_pr = TrackedPullRequest::new(pr, args.token.as_deref()).await?;
				pull_requests.push(tracked_pr);
			}

			println!(
				"{}",
				if json {
					serde_json::to_string(&pull_requests)?
				} else {
					pull_requests
						.iter()
						.map(|tpr| format!("[{}] {}", &tpr.id, &tpr.title.link(&tpr.url)))
						.collect::<Vec<_>>()
						.join("\n")
				}
			)
		}
		Some(Commands::Check {}) => {
			if cache_data.pull_requests.len() == 0 {
				println!("No pull requests saved.");
				return Ok(());
			}

			for (i, pull_request) in cache_data
				.pull_requests
				.iter()
				.enumerate()
			{
				check(*pull_request, args.token.as_deref()).await?;
				if i != (cache_data.pull_requests.len() - 1) {
					println!();
				}
			}
		}
		None => check(args.pull_request.expect("is present"), args.token.as_deref()).await?,
	}

	fs::write(&cache, serde_json::to_string(&cache_data)?)?;

	Ok(())
}
