pub mod fetch;

pub fn format_seconds_to_time_ago(seconds: u64) -> String {
	fn format_unit(value: u64, unit: &str) -> Option<String> {
		if value > 0 {
			Some(format!("{} {}{}", value, unit, if value > 1 { "s" } else { "" }))
		} else {
			None
		}
	}

	let minutes = seconds / 60;
	let hours = minutes / 60;
	let days = hours / 24;

	let remaining_seconds = seconds % 60;
	let remaining_minutes = minutes % 60;
	let remaining_hours = hours % 24;

	let mut result: Vec<String> = [
		format_unit(days, "day"),
		format_unit(remaining_hours, "hour"),
		format_unit(remaining_minutes, "minute"),
		format_unit(remaining_seconds, "second"),
	]
	.iter()
	.filter_map(|x| x.clone())
	.collect();

	if result.is_empty() {
		"0 seconds".to_string()
	} else if result.len() == 1 {
		result[0].clone()
	} else {
		let last = result.pop().unwrap();
		format!("{} and {}", result.join(" "), last)
	}
}
