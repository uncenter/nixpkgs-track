use clap::Parser;
use color_eyre::eyre::{Ok, Result};

use chrono::Utc;
use nixpkgs_track::{fetch::fetch_nixpkgs_pull_request, format_seconds_to_time_ago, tracker::NixpkgsTracker};

use tabled::{
	settings::{object::Rows, style::BorderSpanCorrection, Disable, Panel, Style},
	Table, Tabled,
};
use yansi::{hyperlink::HyperlinkExt, Paint};

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

	let tracker = NixpkgsTracker::new()?;
	let pull_request = fetch_nixpkgs_pull_request(args.pull_request, args.token)?;

	let Some(commit_sha) = pull_request.merge_commit_sha else {
		println!("This pull request is very old. I can't track it!");
		return Ok(());
	};

	if pull_request.merged == false {
		println!("This pull request hasn't been merged yet!")
	} else {
		println!(
			"Pull request was merged {} ago.",
			format_seconds_to_time_ago(
				Utc::now()
					.signed_duration_since(pull_request.merged_at.unwrap())
					.num_seconds()
					.try_into()?
			)
		);

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
