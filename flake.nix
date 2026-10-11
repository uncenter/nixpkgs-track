{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    naersk.url = "github:nix-community/naersk";
  };

  outputs =
    {
      self,
      naersk,
      nixpkgs,
    }:
    let
      forEachSystem = f: builtins.mapAttrs f nixpkgs.legacyPackages;
    in
    {
      packages = forEachSystem (
        system: pkgs:
        let
          naersk' = pkgs.callPackage naersk { };
        in
        {
          nixpkgs-track = naersk'.buildPackage {
            pname = "nixpkgs-track";
            src = ./.;
            nativeBuildInputs = with pkgs; [
              installShellFiles
              pkg-config
            ];
            buildInputs = with pkgs; [
              openssl
            ];
            postInstall = ''
              installShellCompletion \
                target/release/build/nixpkgs-track-*/out/nixpkgs-track.{bash,fish} \
                --zsh target/release/build/nixpkgs-track-*/out/_nixpkgs-track
            '';
          };
          default = self.packages.${system}.nixpkgs-track;
        }
      );

      devShell = forEachSystem (
        system: pkgs:
        pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            clippy
            rustfmt
            rust-analyzer
            cargo
            rustc
          ];
          inputsFrom = [ self.packages.${system}.nixpkgs-track ];
          env = {
            OPENSSL_NO_VENDOR = 1;
            RUST_SRC_PATH = toString pkgs.rustPlatform.rustLibSrc;
          };
        }
      );
    };
}
