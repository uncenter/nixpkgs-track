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
  cargoLock.lockFile = ./Cargo.lock;

  buildInputs = [
    openssl
  ] ++ lib.optionals stdenv.isDarwin (with darwin.apple_sdk.frameworks; [ SystemConfiguration ]);

  nativeBuildInputs = [ pkg-config ];

  meta = {
    description = "Track where Nixpkgs pull requests have reached";
    homepage = "https://github.com/uncenter/nixpkgs-track";
    license = lib.licenses.mit;
    maintainers = with lib.maintainers; [ uncenter ];
    mainProgram = "nixpkgs-track";
  };
}
