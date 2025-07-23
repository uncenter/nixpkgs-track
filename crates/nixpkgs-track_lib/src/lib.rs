use chrono::prelude::*;
use reqwest::{RequestBuilder, StatusCode};
use serde::Deserialize;

use thiserror::Error;

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

pub async fn fetch_nixpkgs_pull_request(client: impl AsRef<reqwest::Client>, pull_request: u64, token: Option<&str>) -> Result<PullRequest, NixpkgsTrackError> {
	let url = format!("{BASE_API_URL}/pulls/{pull_request}");
	let response = build_request(client, &url, token)
		.send()
		.await;

	log::debug!("fetch_nixpkgs_pull_request: {:?}", response);

	match response {
		Ok(response) => match response.status() {
			StatusCode::OK => response
				.json::<PullRequest>()
				.await
				.map_err(NixpkgsTrackError::RequestFailed),
			StatusCode::NOT_FOUND => Err(NixpkgsTrackError::PullRequestNotFound(pull_request)),
			StatusCode::FORBIDDEN => Err(NixpkgsTrackError::RateLimitExceeded),
			_ => Err(NixpkgsTrackError::RequestFailed(response.error_for_status().unwrap_err())),
		},
		Err(err) => Err(NixpkgsTrackError::RequestFailed(err)),
	}
}

pub async fn branch_contains_commit(client: impl AsRef<reqwest::Client>, branch: &str, commit: &str, token: Option<&str>) -> Result<bool, NixpkgsTrackError> {
	// `per_page=1000000&page=100`: a hack for the API, to _not_ return
	//   information about files or commits, which we do not need here;
    //   we only need to know whether it's `ahead` or `behind`
	let url = format!("{BASE_API_URL}/compare/{branch}...{commit}?per_page=1000000&page=100");
	let response = build_request(client, &url, token)
		.send()
		.await;

	log::debug!("branch_contains_commit: {:?}", response);

	match response {
		Ok(response) => match response.status() {
			StatusCode::OK => match response.json::<Comparison>().await {
				Ok(json) => Ok(json.status == "identical" || json.status == "behind"),
				Err(err) => Err(NixpkgsTrackError::RequestFailed(err)),
			},
			StatusCode::NOT_FOUND => Ok(false),
			StatusCode::FORBIDDEN => Err(NixpkgsTrackError::RateLimitExceeded),
			_ => Err(NixpkgsTrackError::RequestFailed(response.error_for_status().unwrap_err())),
		},
		Err(err) => Err(NixpkgsTrackError::RequestFailed(err)),
	}
}

#[derive(Clone, Debug, Deserialize)]
pub struct User {
	pub login: String,
	pub url: String,
}

#[non_exhaustive]
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
	/// Base branch that the pull request is merged into
	pub base: ForkBranch,
	/// Head branch that the pull request is merged from
	pub head: ForkBranch,
}

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize)]
pub struct ForkBranch {
	/// Fork and branch name in the format "owner:branch"
	pub label: String,
	/// Branch name in a given fork
	pub r#ref: String,
	pub sha: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Comparison {
	pub status: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GitHubError {
	pub message: String,
	pub documentation_url: String,
}

#[derive(Error, Debug)]
#[cfg_attr(feature = "miette", derive(miette::Diagnostic))]
pub enum NixpkgsTrackError {
	#[error("Pull request not found")]
	PullRequestNotFound(u64),
	#[error("Rate limit exceeded")]
	RateLimitExceeded,
	#[error(transparent)]
	RequestFailed(#[from] reqwest::Error),
}
