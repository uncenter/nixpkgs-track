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
              pkg-config
            ];
            buildInputs = with pkgs; [
              openssl
            ];
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
