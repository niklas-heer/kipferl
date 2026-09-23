{
  description = "Bake Python CLI apps into fast, standalone binaries";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

  outputs =
    { nixpkgs, ... }:
    let
      forAllSystems = nixpkgs.lib.genAttrs [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];
    in
    {
      packages = forAllSystems (system: rec {
        kipferl = nixpkgs.legacyPackages.${system}.callPackage ./nix/package.nix { };
        default = kipferl;
      });

      formatter = forAllSystems (system: nixpkgs.legacyPackages.${system}.nixfmt);
    };
}
