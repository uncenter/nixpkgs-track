use chrono::prelude::*;
use color_eyre::eyre::Result;
use reqwest::blocking::Client;
use serde::Deserialize;

pub fn fetch_nixpkgs_pull_request(pull_request: u64, token: Option<String>) -> Result<PullRequest> {
	let url: String = format!("https://api.github.com/repos/nixos/nixpkgs/pulls/{}", pull_request);
	let mut request = Client::new()
		.get(&url)
		.header("User-Agent", "nixpkgs-track");

	if let Some(token) = token {
		request = request.bearer_auth(token);
	}

	let response = request.send()?.json::<PullRequest>()?;
	Ok(response)
}

#[derive(Clone, Debug, Deserialize)]
pub struct User {
	pub login: String,
	pub url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PullRequest {
	pub html_url: String,
	pub number: u64,
	pub title: String,
	pub user: User,
	pub created_at: DateTime<Utc>,
	pub merged_at: Option<DateTime<Utc>>,
	pub merged: bool,
	pub merge_commit_sha: Option<String>,
}
