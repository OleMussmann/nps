{
  description = "Nix Package Search";

  inputs = {
    flake-compat.url = "https://flakehub.com/f/edolstra/flake-compat/1.tar.gz";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    {
      self,
      flake-compat,
      flake-utils,
      naersk,
      nixpkgs,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk' = pkgs.callPackage naersk { };
      in
      rec {
        defaultPackage = packages.default;
        packages.default = naersk'.buildPackage {
          src = ./.;
          buildInputs = with pkgs; [ nix ];
        };

        devShells.default =
          with pkgs;
          mkShell {
            buildInputs = [
              alejandra # nix formatting
              cargo-audit # check dependencies for vulnerabilities
              cargo-edit # package management
              cargo-outdated # check for dependency updates
              cargo-release # help creating releases
              cargo # rust toolchain
              cargo-tarpaulin # code coverage
              clippy # linting
              hyperfine # benchmarking
              nixfmt-rfc-style # code formatting
              python312Packages.grip # markdown rendering
              rustfmt # code formatting
            ];
          };
      }
    );
}
