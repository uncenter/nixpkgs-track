use clap::{Parser, Subcommand};

use crate::utils::parse_pull_request_id;

#[derive(Parser)]
#[command(version, about, subcommand_negates_reqs = true)]
pub struct Cli {
	/// Numerical ID of the pull request to check (e.g. 1234) or a GitHub URL to a pull request (e.g. https://github.com/nixos/nixpkgs/pull/1234).
	#[clap(required(true), value_parser = parse_pull_request_id)]
	pub pull_request: Option<u64>,

	#[command(subcommand)]
	pub command: Option<Commands>,

	/// GitHub token
	#[clap(long, short, env = "GITHUB_TOKEN")]
	pub token: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
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
