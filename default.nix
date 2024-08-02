{
  lib,
  darwin,
  stdenv,
  openssl,
  pkg-config,
  rustPlatform,
  version ? "latest",
  ...
}:
rustPlatform.buildRustPackage {
  pname = "nixpkgs-track";
  inherit version;

  src = ./.;
  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "git-tracker-0.2.0" = "sha256-4ah3X7y8bBV/fKQdXJXe3nphqAtnZOpBTG8ZXaoHKcc=";
    };
  };

  buildInputs = [
    openssl
  ] ++ lib.optionals stdenv.isDarwin (with darwin.apple_sdk.frameworks; [ SystemConfiguration ]);

  nativeBuildInputs = [ pkg-config ];

  meta = {
    description = "Track pull requests across Nix channels";
    homepage = "https://github.com/uncenter/nixpkgs-track";
    license = lib.licenses.mit;
    maintainers = with lib.maintainers; [ uncenter ];
    mainProgram = "nixpkgs-track";
  };
}
