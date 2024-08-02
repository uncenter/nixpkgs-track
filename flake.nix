{
  description = "Track pull requests across Nix channels";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs, ... }:
    let
      forAllSystems =
        function:
        nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed (
          system: function nixpkgs.legacyPackages.${system}
        );
      version = self.shortRev or "dirty";
    in
    {
      packages = forAllSystems (pkgs: rec {
        default = pkgs.callPackage ./default.nix { inherit version; };
        nixpkgs-track = default;
      });

      devShells = forAllSystems (pkgs: {
        default = pkgs.callPackage ./shell.nix { };
      });
    };
}
