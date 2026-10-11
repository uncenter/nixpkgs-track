use std::{env, fs};

use clap::{CommandFactory, ValueEnum};
use clap_complete::{Shell, generate_to};

include!("src/cli.rs");

fn main() {
	let out_dir = env::var("OUT_DIR").unwrap();

	fs::create_dir_all(&out_dir).unwrap();

	let mut cmd = Cli::command();
	for &shell in Shell::value_variants() {
		generate_to(shell, &mut cmd, "nixpkgs-track", &out_dir).unwrap();
	}
}
