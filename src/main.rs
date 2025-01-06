use core::result;
use std::fs;
use std::sync::Arc;

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
use std::fmt::Write;
async fn check(client: Arc<reqwest::Client>, pull_request: u64, token: Option<&str>) -> Result<String> {
	let mut output = String::new();

	let pull_request = fetch_nixpkgs_pull_request(client.clone(), pull_request, token).await?;

	let Some(commit_sha) = pull_request.merge_commit_sha else {
		writeln!(output, "This pull request is very old. I can't track it!")?;
		return Ok(output);
	};

	writeln!(output,
		"[{}] {}",
		pull_request.number,
		pull_request
			.title
			.link(pull_request.html_url)
	)?;

	if pull_request.merged == false {
		let created_at_ago = format_seconds_to_time_ago(
			Utc::now()
				.signed_duration_since(pull_request.created_at)
				.num_seconds()
				.try_into()?,
		);
		let created_at_date = pull_request.created_at.to_rfc3339();

		writeln!(output, "This pull request hasn't been merged yet!")?;
		writeln!(output, "Created {} ago ({}).", created_at_ago, created_at_date)?;
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

		writeln!(output, "Merged {} ago ({}), {} after creation.", merged_at_ago, merged_at_date, creation_to_merge_time)?;
		let mut branches = tokio::task::JoinSet::new();
		for (i, branch) in DEFAULT_BRANCHES.iter().enumerate() {
			let token_clone = token.map(|t| t.to_owned());
			let branch_clone = branch.to_string();
			let commit_sha_clone = commit_sha.clone();
			let client_clone = client.clone();

			branches.spawn(async move {
				let result = branch_contains_commit(client_clone, &branch_clone, &commit_sha_clone, token_clone.as_deref())
					.await
					.map(|has_pr| format!("{}: {}", branch_clone, if has_pr { "✅" } else { "🚫" }));
				(i, result)
			});
		}

		let mut results = branches.join_all().await;
		results.sort_by_key(|r| r.0);
		for (_, result) in results {
			match result {
				core::result::Result::Ok(result) => {
					writeln!(output, "{}", result)?;
				},
				Err(err) => {
					writeln!(output, "Error: {}", err)?;
				}

			}
		}
	}

	Ok(output)
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
				async fn new(client: impl AsRef<reqwest::Client>, id: u64, token: Option<&str>) -> Result<Self> {
					let data = fetch_nixpkgs_pull_request(client, id, token.as_deref()).await?;

					Ok(TrackedPullRequest {
						id,
						title: data.title,
						url: data.html_url,
					})
				}
			}
			let mut pull_requests = tokio::task::JoinSet::new();
			let client = Arc::new(reqwest::Client::new());

			for &pr in &cache_data.pull_requests {
				let token = args.token.clone();
				let client = client.clone();
				pull_requests.spawn(async move {
					let tracked_pr = TrackedPullRequest::new(client, pr, token.as_deref()).await?;
					Ok(tracked_pr)
				});
			}
			let pull_requests = pull_requests.join_all().await.into_iter().filter_map(|x| x.ok()).collect::<Vec<_>>();

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
			let mut set = tokio::task::JoinSet::new();
			let client = Arc::new(reqwest::Client::new());

			for (i, pull_request) in cache_data
				.pull_requests
				.iter()
				.cloned()
				.enumerate()
			{
				let client = client.clone();
				let token = args.token.clone();
				let len = cache_data.pull_requests.len();
				set.spawn(async move {
					let output = check(client, pull_request, token.as_deref()).await.ok();
					if let Some(output) = output {
						println!("{}", output);
					}
					if i != (len - 1) {
						println!();
					}
				});
			}
			set.join_all().await;
		}
		None => println!("{}", check(Arc::new(reqwest::Client::new()), args.pull_request.expect("is present"), args.token.as_deref()).await?),
	}

	fs::write(&cache, serde_json::to_string(&cache_data)?)?;

	Ok(())
}
