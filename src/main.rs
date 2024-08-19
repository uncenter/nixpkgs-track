use clap::Parser;
use color_eyre::eyre::{Ok, Result};

use chrono::Utc;
use nixpkgs_track::{branch_contains_commit, fetch_nixpkgs_pull_request, utils::format_seconds_to_time_ago};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
	pull_request: u64,

	/// GitHub token
	#[clap(long, short, env = "GITHUB_TOKEN")]
	token: Option<String>,
}

fn main() -> Result<()> {
	let args = Cli::parse();
	color_eyre::install()?;

	println!("Fetching pull request data...");
	let pull_request = fetch_nixpkgs_pull_request(args.pull_request, args.token.as_deref())?;

	let Some(commit_sha) = pull_request.merge_commit_sha else {
		println!("This pull request is very old. I can't track it!");
		return Ok(());
	};

	if pull_request.merged == false {
		println!("This pull request hasn't been merged yet!")
	} else {
		println!("{} ({})", pull_request.title, pull_request.html_url);
		println!(
			"Merged {} ago ({}).",
			format_seconds_to_time_ago(
				Utc::now()
					.signed_duration_since(pull_request.merged_at.unwrap())
					.num_seconds()
					.try_into()?
			),
			pull_request
				.merged_at
				.unwrap()
				.to_rfc3339()
		);

		println!("Fetching branch comparisons...");

		let branches = ["master", "staging", "staging-next", "nixpkgs-unstable", "nixos-unstable-small", "nixos-unstable"];

		for branch in branches {
			let has_pull_request = branch_contains_commit(branch, &commit_sha, args.token.as_deref())?;

			println!("{}: {}", branch, if has_pull_request { "✅" } else { "🚫" });
		}
	}

	Ok(())
}
