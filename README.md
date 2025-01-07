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

```sh
cargo install --git https://github.com/uncenter/nixpkgs-track.git
```

## Usage

The simplest way to track a pull request can be done like so:

```
nixpkgs-track <PULL_REQUEST>
```

Where a `PULL_REQUEST` is the numerical ID of the pull request to track, such as `370713` for [github.com/NixOS/nixpkgs/pull/370713](https://togithub.com/NixOS/nixpkgs/pull/370713).

> [!TIP]
> Provide a GitHub API token with the `--token` option or set it in the `GITHUB_TOKEN` environment variable to avoid rate-limiting.

nixpkgs-track also supports saving a list of pull requests to check later on.

### `add <PULL_REQUESTS...>`

Add specified pull request(s) to the list.

### `remove [<PULL_REQUESTS...> | --all]`

Remove specified pull request(s) from the list, or remove all pull requests from the list with `--all`.

### `list`

List tracked pull requests and their metadata.

### `check`

Check each tracked pull request. Equivalent to running `nixpkgs-track <PULL_REQUEST>` for each pull request in the list.

## License

[MIT](LICENSE)
