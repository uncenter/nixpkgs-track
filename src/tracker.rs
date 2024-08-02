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

	pub fn branch_by_name(&self, name: &str) -> Result<Branch> {
		Ok(self
			.repository
			.find_branch(name, BranchType::Remote)?)
	}

	pub fn ref_contains_commit(&self, reference: &Reference, commit: &Commit) -> Result<bool> {
		let head = reference.peel_to_commit()?;

		// NOTE: we have to check this as `Repository::graph_descendant_of()` (like the name says)
		// only finds *descendants* of it's parent commit, and will not tell us if the parent commit
		// *is* the child commit. i have no idea why i didn't think of this, but that's why this
		// comment is here now
		let is_head = head.id() == commit.id();

		let has_commit = self
			.repository
			.graph_descendant_of(head.id(), commit.id())?;

		Ok(has_commit || is_head)
	}

	pub fn branch_contains_sha(&self, branch_name: &str, commit_sha: &str) -> Result<bool> {
		let commit = self.commit_by_sha(commit_sha)?;
		let branch = self.branch_by_name(branch_name)?;
		let has_pr = self.ref_contains_commit(&branch.into_reference(), &commit)?;

		Ok(has_pr)
	}
}
