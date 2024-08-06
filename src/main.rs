use clap::Parser;
use color_eyre::eyre::{Ok, Result};

use chrono::Utc;
use nixpkgs_track::{
	fetch::{branch_contains_commit, fetch_nixpkgs_pull_request},
	format_seconds_to_time_ago,
};

use tabled::{
	settings::{object::Rows, style::BorderSpanCorrection, Disable, Panel, Style},
	Table, Tabled,
};

#[derive(Tabled, Debug)]
#[tabled(rename_all = "PascalCase")]
struct BranchStatus {
	branch: String,
	#[tabled(display_with = "display_branch_status")]
	has_pull_request: bool,
}

fn display_branch_status(b: &bool) -> String {
	if *b { "✅" } else { "🚫" }.to_string()
}

#[derive(Parser)]
#[command(version, about)]
struct Cli {
	pull_request: u64,

	// GitHub token
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
		println!("Fetching branch comparisons...");

		let branches = ["master", "staging", "staging-next", "nixpkgs-unstable", "nixos-unstable-small", "nixos-unstable"];
		let mut branch_statuses: Vec<BranchStatus> = vec![];

		for branch in branches {
			let has_pull_request = branch_contains_commit(branch, &commit_sha, args.token.as_deref())?;

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
			)))
			.with(Panel::header(format!("{} ({})", pull_request.title, pull_request.html_url)))
			.with(BorderSpanCorrection);
		println!("{}", table.to_string());
	}

	Ok(())
}
