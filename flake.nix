{
  description = "mille — Architecture Checker, Rust-based multi-language architecture linter";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        mille = pkgs.rustPlatform.buildRustPackage {
          pname = "mille";
          version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;

          src = self;

          cargoLock.lockFile = ./Cargo.lock;

          meta = with pkgs.lib; {
            description = "Architecture Checker — Rust-based multi-language architecture linter";
            homepage = "https://github.com/makinzm/mille";
            license = licenses.mit;
            maintainers = [ ];
            mainProgram = "mille";
          };
        };
      in
      {
        packages.mille = mille;
        packages.default = mille;

        apps.mille = flake-utils.lib.mkApp { drv = mille; };
        apps.default = flake-utils.lib.mkApp { drv = mille; };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [ mille ];
        };
      }
    );
}
