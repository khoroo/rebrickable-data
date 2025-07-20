{
  description = "Rebrickable flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            (python313.withPackages (ps: with ps; [
              requests
              tqdm
              polars
              altair
              jupyter
              ipykernel
              ipython
            ]))
          ];
        };
      });
}
