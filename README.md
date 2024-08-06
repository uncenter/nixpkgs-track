# nixpkgs-track

Track where Nixpkgs pull requests have reached (is that update in `nixpkgs-unstable` yet??).

Originally created as a local and reliable CLI alternative to [Alyssa Ross](https://alyssa.is/)'s [Nixpkgs Pull Request Tracker
](https://nixpk.gs/pr-tracker.html) website and based by [getchoo/nixpkgs-tracker-bot](https://github.com/getchoo/nixpkgs-tracker-bot). However, nixpkgs-track is now based on the wonderful new [ocfox/nixpkgs-tracker](https://github.com/ocfox/nixpkgs-tracker) project, solely using GitHub's API.

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

```
nixpkgs-track <PULL_REQUEST> [--token <TOKEN>]
```

## License

[MIT](LICENSE)
