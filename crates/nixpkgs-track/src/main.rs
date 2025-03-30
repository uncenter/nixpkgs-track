use std::fmt::Write;
use std::fs;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use dialoguer::MultiSelect;
use miette::{Context, IntoDiagnostic};
use serde::{Deserialize, Serialize};

use chrono::Utc;
use yansi::hyperlink::HyperlinkExt;

use nixpkgs_track::utils::{format_seconds_to_time_ago, parse_pull_request_id};
use nixpkgs_track_lib::{branch_contains_commit, fetch_nixpkgs_pull_request, NixpkgsTrackError};

static DEFAULT_BRANCHES: [&str; 6] = ["master", "staging", "staging-next", "nixpkgs-unstable", "nixos-unstable-small", "nixos-unstable"];

#[derive(Parser)]
#[command(version, about, subcommand_negates_reqs = true)]
struct Cli {
	/// Numerical ID of the pull request to check (e.g. 1234) or a GitHub URL to a pull request (e.g. https://github.com/nixos/nixpkgs/pull/1234).
	#[clap(required(true), value_parser = parse_pull_request_id)]
	pull_request: Option<u64>,

	#[command(subcommand)]
	command: Option<Commands>,

	/// GitHub token
	#[clap(long, short, env = "GITHUB_TOKEN")]
	token: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
	/// Add pull request(s) to track list
	Add {
		#[clap(required(true), value_parser = parse_pull_request_id)]
		pull_requests: Vec<u64>,
	},
	/// Remove pull request(s) from track list
	Remove {
		#[clap(required_unless_present("all"), value_parser = parse_pull_request_id)]
		pull_requests: Vec<u64>,

		#[clap(long, conflicts_with = "pull_requests")]
		all: bool,

		#[clap(short, long, exclusive = true)]
		interactive: bool,
	},
	/// List tracked pull requests
	List {
		#[clap(long)]
		json: bool,
	},
	/// Check tracked pull requests
	Check {},
}

async fn check(client: Arc<reqwest::Client>, pull_request: u64, token: Option<&str>) -> miette::Result<String, CheckError> {
	let mut output = String::new();

	let pull_request = fetch_nixpkgs_pull_request(client.clone(), pull_request, token).await?;

	let Some(commit_sha) = pull_request.merge_commit_sha else {
		writeln!(output, "This pull request is very old. I can't track it!")?;
		return Ok(output);
	};

	writeln!(
		output,
		"[{}] {}",
		pull_request.number,
		pull_request
			.title
			.link(pull_request.html_url)
	)?;

	if pull_request.merged {
		let merged_at_ago = format_seconds_to_time_ago(
			Utc::now()
				.signed_duration_since(pull_request.merged_at.unwrap())
				.num_seconds(),
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
				.num_seconds(),
		);

		writeln!(output, "Merged {merged_at_ago} ago ({merged_at_date}), {creation_to_merge_time} after creation.")?;

		let mut branches = tokio::task::JoinSet::new();
		for (i, branch) in DEFAULT_BRANCHES.iter().enumerate() {
			let token_clone = token.map(ToOwned::to_owned);
			let branch_clone = (*branch).to_string();
			let commit_sha_clone = commit_sha.clone();
			let client_clone = client.clone();

			branches.spawn(async move {
				let result = branch_contains_commit(client_clone, &branch_clone, &commit_sha_clone, token_clone.as_deref()).await;
				(i, result)
			});
		}

		let mut results = branches.join_all().await;
		results.sort_by_key(|r| r.0);

		for (i, result) in results {
			let has_pull_request = result?;
			writeln!(output, "{}: {}", DEFAULT_BRANCHES[i], if has_pull_request { "✅" } else { "🚫" })?;
		}
	} else {
		let created_at_ago = format_seconds_to_time_ago(
			Utc::now()
				.signed_duration_since(pull_request.created_at)
				.num_seconds(),
		);
		let created_at_date = pull_request.created_at.to_rfc3339();

		writeln!(output, "This pull request hasn't been merged yet!")?;
		writeln!(output, "Created {created_at_ago} ago ({created_at_date}).")?;
	}

	Ok(output)
}

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> miette::Result<()> {
	env_logger::init();

	let args = Cli::parse();

	let cache_dir = user_dirs::cache_dir()
		.into_diagnostic()?
		.join("nixpkgs-track");
	if !cache_dir.exists() {
		fs::create_dir_all(&cache_dir).map_err(CacheFsError)?;
	}

	let cache = cache_dir.join("cache.json");

	let mut cache_data: Cache = if cache.exists() {
		serde_json::from_str(&fs::read_to_string(&cache).map_err(CacheFsError)?)
			.into_diagnostic()
			.context("Failed to parse cache file")?
	} else {
		Cache::new()
	};

	match args.command {
		Some(Commands::Add { pull_requests }) => {
			cache_data
				.pull_requests
				.extend(pull_requests);
			cache_data.pull_requests.sort_unstable();
			cache_data.pull_requests.dedup();
		}
		Some(Commands::Remove { pull_requests, all, interactive }) => {
			if interactive {
				let pull_requests = fetch_cached_pull_requests(&cache_data.pull_requests, args.token).await?;

				let selection = MultiSelect::new()
					.with_prompt("Select pull requests to remove")
					.report(false)
					.items(&pull_requests)
					.interact_opt()
					.unwrap();

				if let Some(selection) = selection {
					let selected_pull_requests: Vec<u64> = selection
						.iter()
						.map(|&i| pull_requests[i].id)
						.collect();

					cache_data
						.pull_requests
						.retain(|x| !selected_pull_requests.contains(x));

					println!("Selected pull requests ({}) removed.", selection.len());
				} else {
					println!("No pull requests selected.");
					return Ok(());
				}
			} else if all {
				println!("All pull requests ({} total) removed.", cache_data.pull_requests.len());
				cache_data.pull_requests.clear();
			} else {
				cache_data
					.pull_requests
					.retain(|x| !pull_requests.contains(x));
			}
		}
		Some(Commands::List { json }) => {
			let pull_requests = fetch_cached_pull_requests(&cache_data.pull_requests, args.token).await?;

			println!(
				"{}",
				if json {
					serde_json::to_string(&pull_requests)
						.into_diagnostic()
						.context("Failed to serialize pull requests to JSON")?
				} else {
					pull_requests
						.iter()
						.map(ToString::to_string)
						.collect::<Vec<_>>()
						.join("\n")
				}
			);
		}
		Some(Commands::Check {}) => {
			if cache_data.pull_requests.is_empty() {
				println!("No pull requests saved.");
				return Ok(());
			}
			let mut set = tokio::task::JoinSet::new();
			let client = Arc::new(reqwest::Client::new());

			for pull_request in cache_data.pull_requests.iter().copied() {
				let client = client.clone();
				let token = args.token.clone();

				set.spawn(async move { check(client, pull_request, token.as_deref()).await });
			}

			let results: Result<Vec<String>, CheckError> = set
				.join_all()
				.await
				.into_iter()
				.collect();
			print!("{}", results?.join("\n"));
		}
		None => print!("{}", check(Arc::new(reqwest::Client::new()), args.pull_request.expect("is present"), args.token.as_deref()).await?),
	}

	fs::write(
		&cache,
		serde_json::to_string(&cache_data)
			.into_diagnostic()
			.context("Failed to serialize cache file")?,
	)
	.map_err(CacheFsError)?;

	Ok(())
}

#[derive(Serialize, Deserialize)]
struct Cache {
	pull_requests: Vec<u64>,
}

impl Cache {
	fn new() -> Self {
		Cache { pull_requests: vec![] }
	}
}

#[derive(Serialize, Deserialize)]
struct TrackedPullRequest {
	id: u64,
	title: String,
	url: String,
}

impl TrackedPullRequest {
	async fn new(client: impl AsRef<reqwest::Client>, id: u64, token: Option<&str>) -> Result<Self, NixpkgsTrackError> {
		let data = fetch_nixpkgs_pull_request(client, id, token).await?;

		Ok(TrackedPullRequest {
			id,
			title: data.title,
			url: data.html_url,
		})
	}
}

impl ToString for TrackedPullRequest {
	fn to_string(&self) -> String {
		format!("[{}] {}", &self.id, &self.title.link(&self.url))
	}
}

async fn fetch_cached_pull_requests(pull_requests: &[u64], token: Option<String>) -> miette::Result<Vec<TrackedPullRequest>, NixpkgsTrackError> {
	if pull_requests.is_empty() {
		println!("No pull requests saved.");
		return Ok(vec![]);
	}

	let mut set = tokio::task::JoinSet::new();
	let client = Arc::new(reqwest::Client::new());

	for &pr in pull_requests {
		let token = token.clone();
		let client = client.clone();
		set.spawn(async move {
			let tracked_pr = TrackedPullRequest::new(client, pr, token.as_deref()).await?;
			Ok(tracked_pr)
		});
	}
	let pull_requests: Result<Vec<TrackedPullRequest>, NixpkgsTrackError> = set
		.join_all()
		.await
		.into_iter()
		.collect();

	pull_requests
}

#[derive(thiserror::Error, Debug, miette::Diagnostic)]
#[error("An error occurred while reading or writing the cache file.")]
pub struct CacheFsError(#[from] std::io::Error);

#[derive(thiserror::Error, Debug, miette::Diagnostic)]
pub enum CheckError {
	#[error("Failed to fetch the pull request.")]
	#[diagnostic(help("Is the GitHub authentication token, set by the --token flag or GITHUB_TOKEN environment variable, correct?"))]
	RequestFailed(#[source] reqwest::Error),
	#[error("Pull request {0} not found.")]
	#[diagnostic(help("Are you sure the pull request exists?"))]
	PullRequestNotFound(u64),
	#[error("GitHub rate limit was exceeded.")]
	#[diagnostic(help("You can provide a GitHub token with the --token flag or GITHUB_TOKEN environment variable."))]
	RateLimitExceeded,
	#[error("An error occurred while formatting the output.")]
	FormatFailed(#[from] std::fmt::Error),
}

impl From<NixpkgsTrackError> for CheckError {
	fn from(err: NixpkgsTrackError) -> Self {
		match err {
			NixpkgsTrackError::RequestFailed(err) => CheckError::RequestFailed(err),
			NixpkgsTrackError::PullRequestNotFound(pull_request) => CheckError::PullRequestNotFound(pull_request),
			NixpkgsTrackError::RateLimitExceeded => CheckError::RateLimitExceeded,
		}
	}
}
