pub mod utils;

use chrono::prelude::*;
use color_eyre::eyre::Result;
use reqwest::{
	blocking::{Client, RequestBuilder},
	StatusCode,
};
use serde::Deserialize;

const BASE_API_URL: &str = "https://api.github.com/repos/nixos/nixpkgs";

fn build_request(url: &str, token: Option<&str>) -> RequestBuilder {
	let mut request = Client::new()
		.get(url)
		.header("User-Agent", "nixpkgs-track");

	if let Some(token) = token {
		request = request.bearer_auth(token);
	}

	request
}

pub fn fetch_nixpkgs_pull_request(pull_request: u64, token: Option<&str>) -> Result<PullRequest> {
	let url = format!("{}/pulls/{}", BASE_API_URL, pull_request);
	let response = build_request(&url, token)
		.send()?
		.json::<PullRequest>()?;
	Ok(response)
}

pub fn branch_contains_commit(branch: &str, commit: &str, token: Option<&str>) -> Result<bool> {
	let url = format!("{}/compare/{}...{}", BASE_API_URL, branch, commit);

	let response = build_request(&url, token).send()?;
	Ok(match response.status() {
		StatusCode::NOT_FOUND => false,
		_ => {
			let json = response.json::<Comparison>()?;
			if json.status == "identical" || json.status == "behind" {
				true
			} else {
				false
			}
		}
	})
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

#[derive(Clone, Debug, Deserialize)]
pub struct Comparison {
	pub status: String,
}
