{
  description = "Squads overlay flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = { self, nixpkgs, ... }: let
    pkgsFor = system: nixpkgs.legacyPackages.${system};
    systems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];
    forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f (pkgsFor system) system);
  in
  {
    packages = forAllSystems (pkgs: system: {
      squads = pkgs.callPackage ./package.nix {};
      default = self.packages.${system}.squads;
    });
    overlays = {
      squads = import ./.;
      default = self.overlays.squads;
    };
  };
}
