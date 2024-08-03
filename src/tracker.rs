// Partially adapted from https://github.com/getchoo/nixpkgs-tracker-bot.

// Copyright (c) 2024 seth

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

use color_eyre::eyre::Result;
use etcetera::{choose_base_strategy, BaseStrategy};
use git2::{Branch, BranchType, Commit, Oid, Reference, Repository};
use std::fs;

pub struct NixpkgsTracker {
	repository: Repository,
}

impl NixpkgsTracker {
	pub fn new() -> Result<Self> {
		let nixpkgs_path = choose_base_strategy()
			.unwrap()
			.cache_dir()
			.join("nixpkgs-track/nixpkgs");

		let repository = if !nixpkgs_path.exists() {
			println!("Nixpkgs tracker repository doesn't exist. Cloning...");
			fs::create_dir_all(&nixpkgs_path)?;
			Repository::clone("https://github.com/NixOS/nixpkgs", &nixpkgs_path)?
		} else {
			println!("Nixpkgs tracker repository already exists. Updating...");
			let repo = Repository::open(&nixpkgs_path)?;
			repo.find_remote("origin")?
				.fetch(&["master"], None, None)?;
			repo.checkout_head(None)?;

			repo
		};

		Ok(Self { repository })
	}

	pub fn commit_by_sha(&self, sha: &str) -> Result<Commit> {
		let oid = Oid::from_str(sha)?;
		let commit = self.repository.find_commit(oid)?;

		Ok(commit)
	}

	pub fn ref_contains_commit(&self, reference: &Reference, commit: &Commit) -> Result<bool> {
		let head = reference.peel_to_commit()?;

		// Check if the parent commit is the same as the child/current commit.
		let contains_commit = head.id() == commit.id() || {
			// Then check if the child/current commit is a descendant of the parent commit on this reference.
			self.repository
				.graph_descendant_of(head.id(), commit.id())?
		};

		Ok(contains_commit)
	}

	pub fn branch_contains_sha(&self, branch_name: &str, commit_sha: &str) -> Result<bool> {
		let commit = self.commit_by_sha(commit_sha)?;
		let branch = self
			.repository
			.find_branch(branch_name, BranchType::Remote)?;
		let has_pull_request = self.ref_contains_commit(&branch.into_reference(), &commit)?;

		Ok(has_pull_request)
	}
}
