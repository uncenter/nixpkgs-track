# nixpkgs-track

Track where Nixpkgs pull requests have reached (is that update in `nixpkgs-unstable` yet??).

```console
$ nixpkgs-track 370713
[370713] helix: 24.07 -> 25.01
Merged 3 days 23 hours 9 minutes and 3 seconds ago (2025-01-03T21:58:20+00:00), 1 hour 18 minutes and 54 seconds after creation.
master: ✅
staging: ✅
staging-next: ✅
nixpkgs-unstable: ✅
nixos-unstable-small: ✅
nixos-unstable: ✅
```

Originally created as a local and reliable CLI alternative to [Alyssa Ross](https://alyssa.is/)'s [Nixpkgs Pull Request Tracker
](https://nixpk.gs/pr-tracker.html) website and based by [getchoo/nixpkgs-tracker-bot](https://github.com/getchoo/nixpkgs-tracker-bot). However, nixpkgs-track is now primarily based on the recent [ocfox/nixpkgs-tracker](https://github.com/ocfox/nixpkgs-tracker) project, solely using GitHub's API.

## Installation

### Nix

```
nix run github:uncenter/nixpkgs-track
```

### Cargo

Install from crates.io (recommended):

```sh
cargo install nixpkgs-track
```

Or directly from the Git source:

```sh
cargo install --git https://github.com/uncenter/nixpkgs-track.git
```

## Usage

The simplest way to track a pull request is like so:

```
nixpkgs-track <PULL_REQUEST>
```

Where `PULL_REQUEST` is the numerical ID of the pull request to track, such as `370713` for [github.com/NixOS/nixpkgs/pull/370713](https://togithub.com/NixOS/nixpkgs/pull/370713). You may also provide the full GitHub pull request URL instead of just the numerical ID, with something like `nixpkgs-track https://github.com/NixOS/nixpkgs/pull/370713`.

> [!TIP]
> Provide a GitHub API token with the `--token` option or set it in the `GITHUB_TOKEN` environment variable to avoid rate-limiting.

nixpkgs-track also supports saving a list of pull requests to check later on.

### `add <PULL_REQUESTS...>`

Add specified pull request(s) to the list.

### `remove [<PULL_REQUESTS...> | --all]`

Remove specified pull request(s) from the list. Remove all pull requests from the list with `--all`.

### `list`

List tracked pull requests and their metadata.

### `check`

Check each tracked pull request. Equivalent to running `nixpkgs-track <PULL_REQUEST>` for each pull request in the list.

## Library

This crate also exports a simple library interface for other programs. This is available as [`nixpkgs-track_lib`](https://crates.io/crates/nixpkgs-track_lib) on crates.io.

The two primary functions are [`nixpkgs_track_lib::fetch_nixpkgs_pull_request`](https://docs.rs/nixpkgs-track_lib/0.2.0/nixpkgs_track_lib/fn.fetch_nixpkgs_pull_request.html) for fetching pull request data from the GitHub API, and [`nixpkgs_track_lib::branch_contains_commit`](https://docs.rs/nixpkgs-track_lib/0.2.0/nixpkgs_track_lib/fn.branch_contains_commit.html) for checking if a commit SHA (such as from a merged pull request: [`PullRequest.merge_commit_sha`](https://docs.rs/nixpkgs-track_lib/0.2.0/nixpkgs_track_lib/struct.PullRequest.html#structfield.merge_commit_sha)) is present in a specified branch on GitHub.

The implementation used for the command line interface can be found at [`crates/nixpkgs-track/src/main.rs`](crates/nixpkgs-track/src/main.rs), under the `check` function. See also [`src/commands/misc/nixpkgs.rs`](https://github.com/isabelroses/blahaj/blob/main/src/commands/misc/nixpkgs.rs) of [@isabelroses](https://github.com/isabelroses)'s [Blåhaj bot for Discord](https://github.com/isabelroses/blahaj).

## License

[MIT](LICENSE)
