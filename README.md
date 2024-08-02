# nixpkgs-track

Track where Nixpkgs pull requests have reached (is that update in `nixpkgs-unstable` yet??).

A local, reliable CLI alternative to [Alyssa Ross](https://alyssa.is/)'s great but fickle [Nixpkgs Pull Request Tracker
](https://nixpk.gs/pr-tracker.html) website. Also inspired by and [partially adapted](./src/tracker.rs) from [getchoo/nixpkgs-tracker-bot](https://github.com/getchoo/nixpkgs-tracker-bot).

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
nixpkgs-track [OPTIONS] <PULL_REQUEST>
```

## License

[MIT](LICENSE)
