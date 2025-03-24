use reqwest::Url;

fn format_unit(value: i64, unit: &str) -> Option<String> {
	if value > 0 {
		Some(format!("{} {}{}", value, unit, if value > 1 { "s" } else { "" }))
	} else {
		None
	}
}

pub fn format_seconds_to_time_ago(seconds: i64) -> String {
	let minutes = seconds / 60;
	let hours = minutes / 60;
	let days = hours / 24;

	let remaining_seconds = seconds % 60;
	let remaining_minutes = minutes % 60;
	let remaining_hours = hours % 24;

	let result: Vec<String> = [
		format_unit(days, "day"),
		format_unit(remaining_hours, "hour"),
		format_unit(remaining_minutes, "minute"),
		format_unit(remaining_seconds, "second"),
	]
	.into_iter()
	.flatten()
	.collect();

	match result.as_slice() {
		[] => "0 seconds".to_string(),
		[single] => single.clone(),
		[.., last] => format!("{} and {}", result[..result.len() - 1].join(" "), last),
	}
}

pub fn parse_pull_request_id(input: &str) -> Result<u64, String> {
	if let Ok(id) = input.parse::<u64>() {
		return Ok(id);
	}

	// Downcase URL since GitHub URLs are case-insensitive (e.g. "nixos" vs "NixOS").
	if let Ok(url) = Url::parse(input.to_lowercase().as_str()) {
		if url.domain() == Some("github.com") {
			let segments: Vec<_> = url
				.path_segments()
				.map(|s| s.collect())
				.unwrap_or_default();
			if let ["nixos", "nixpkgs", "pull", id, ..] = segments.as_slice() {
				if let Ok(id) = id.parse::<u64>() {
					return Ok(id);
				}
			}
		}
	}

	Err(format!(
		"malformed pull request ID or URL: {input}. expected format: <u64> or https://github.com/nixos/nixpkgs/pull/<u64>"
	))
}

#[cfg(test)]
mod parse_pull_request_id_tests {
	use super::*;

	#[test]
	#[should_panic(expected = "malformed pull request ID or URL")]
	fn test_parse_invalid_pull_request_id() {
		parse_pull_request_id("invalid").unwrap();
	}

	#[test]
	fn test_parse_valid_pull_request_id() {
		assert_eq!(parse_pull_request_id("370713").unwrap(), 370713);
		assert_eq!(parse_pull_request_id("https://github.com/nixos/nixpkgs/pull/370713").unwrap(), 370713);
		assert_eq!(
			parse_pull_request_id("https://github.com/NixOS/nixpkgs/pull/392611/files#diff-7b2093bb8f2a6170ffb914e178c91654f6d14c00ab17e9b4d8092fa943deb7b0").unwrap(),
			392611
		);
	}
}
