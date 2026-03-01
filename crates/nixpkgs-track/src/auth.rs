use std::process::Command;

use crate::cli::Cli;

pub fn get_github_token(args: &Cli) -> Option<String> {
	args.token.clone().or_else(|| {
		Command::new("gh")
			.args(["auth", "token"])
			.output()
			.ok()
			.filter(|output| output.status.success())
			.and_then(|output| String::from_utf8(output.stdout).ok())
			.map(|s| s.trim().to_string())
	})
}
