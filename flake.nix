{
  description = "Rebrickable flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; overlays = [rust-overlay.overlays.default]; };
        toolchain = pkgs.rust-bin.fromRustupToolchainFile ./toolchain.toml;
        pythonWithPackages = pkgs.python313.withPackages (ps: with ps; [
          requests
          ipython
          tqdm
          polars
          altair
          matplotlib
          jupyter
          ipykernel
          ipython
          scipy
        ]);
      in {
        devShells.default = pkgs.mkShell {
          RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
          PYTHONPATH = "${pythonWithPackages}/bin/python";
          PYTHON_PATH = "${pythonWithPackages}/bin/python";
          packages = with pkgs; [
            toolchain
            rust-analyzer-unwrapped
            clippy
            pythonWithPackages
          ];
        };
      });
}
