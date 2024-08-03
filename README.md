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

> [!NOTE]
> The initial run will take at least few minutes for results as nixpkgs-track requires a full clone of the Nixpkgs repository to query. Depending on your internet connection/stability, cloning may take up to 15 minutes. After the initial run and clone, subsequent invocations should be considerably faster (5-10 seconds) as nixpkgs-track only pulls to update the cached repository before querying.

## License

[MIT](LICENSE)
