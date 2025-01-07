pub mod utils;

use chrono::prelude::*;
use color_eyre::eyre::{bail, Result};
use reqwest::{RequestBuilder, StatusCode};
use serde::Deserialize;

const BASE_API_URL: &str = "https://api.github.com/repos/nixos/nixpkgs";

fn build_request(client: impl AsRef<reqwest::Client>, url: &str, token: Option<&str>) -> RequestBuilder {
	let mut request = client
		.as_ref()
		.get(url)
		.header("User-Agent", "nixpkgs-track");

	if let Some(token) = token {
		request = request.bearer_auth(token);
	}

	request
}

pub async fn fetch_nixpkgs_pull_request(client: impl AsRef<reqwest::Client>, pull_request: u64, token: Option<&str>) -> Result<PullRequest> {
	let url = format!("{BASE_API_URL}/pulls/{pull_request}");
	let response = build_request(client, &url, token)
		.send()
		.await?;

	if response.status() == StatusCode::NOT_FOUND {
		bail!("Pull request {} not found", pull_request);
	} else if response.status() == StatusCode::FORBIDDEN {
		let error = response
			.json::<GitHub403Forbidden>()
			.await?;
		bail!("Error: {}", error.message);
	} else {
		Ok(response.json::<PullRequest>().await?)
	}
}

pub async fn branch_contains_commit(client: impl AsRef<reqwest::Client>, branch: &str, commit: &str, token: Option<&str>) -> Result<bool> {
	let url = format!("{BASE_API_URL}/compare/{branch}...{commit}");

	let response = build_request(client, &url, token)
		.send()
		.await?;
	Ok(if response.status() == StatusCode::NOT_FOUND {
		false
	} else {
		let json = response.json::<Comparison>().await?;
		json.status == "identical" || json.status == "behind"
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

#[derive(Clone, Debug, Deserialize)]
pub struct GitHub403Forbidden {
	pub message: String,
	pub documentation_url: String,
}
