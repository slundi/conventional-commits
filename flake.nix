{
  description = "A lightweight library and CLI tool for validating Conventional Commits";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [(import rust-overlay)];
      pkgs = import nixpkgs {
        inherit system overlays;
      };

      # Define Rust toolchain
      rustToolchain = pkgs.rust-bin.stable.latest.default.override {
        extensions = ["rust-src" "clippy" "rustfmt"];
      };

      # Build the package
      conventionnal-commits = pkgs.rustPlatform.buildRustPackage rec {
        pname = "conventionnal-commits";
        version = "0.1.0";

        src = ./.;

        cargoLock = {
          lockFile = ./Cargo.lock;
        };

        nativeBuildInputs = with pkgs; [
          installShellFiles
        ];

        # Tests require the binary to be built first
        checkPhase = ''
          runHook preCheck
          cargo test --lib --verbose
          runHook postCheck
        '';

        postInstall = ''
          # Install shell completions if clap supports it
          $out/bin/commit-check --help > /dev/null || true

          # Create man page directory structure
          mkdir -p $out/share/man/man1
        '';

        meta = with pkgs.lib; {
          description = "A lightweight library and CLI tool for validating Conventional Commits";
          homepage = "https://github.com/yourusername/conventionnal-commits";
          license = licenses.mit;
          maintainers = []; # Add your maintainer info here
          mainProgram = "commit-check";
          platforms = platforms.all;
        };
      };
    in {
      # Default package
      packages.default = conventionnal-commits;
      packages.conventionnal-commits = conventionnal-commits;

      # Development shell
      devShells.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          rustToolchain
          cargo-audit
          cargo-tarpaulin
          rust-analyzer
          nixpkgs-fmt
          nix-prefetch-github
          # For generating completion scripts
          installShellFiles
        ];

        shellHook = ''
          echo "🚀 Development environment for conventionnal-commits"
          echo "Available commands:"
          echo "  cargo build    - Build the project"
          echo "  cargo test     - Run tests"
          echo "  cargo clippy   - Run linter"
          echo "  cargo fmt      - Format code"
          echo "  make test      - Run comprehensive tests"
          echo "  nix build      - Build with Nix"
        '';
      };

      # Apps
      apps.default = flake-utils.lib.mkApp {
        drv = conventionnal-commits;
        name = "commit-check";
      };

      apps.commit-check = flake-utils.lib.mkApp {
        drv = conventionnal-commits;
        name = "commit-check";
      };

      # Checks for CI
      checks = {
        build = conventionnal-commits;
        format =
          pkgs.runCommand "check-format"
          {
            buildInputs = [rustToolchain];
          }
          ''
            cd ${./.}
            cargo fmt -- --check
            touch $out
          '';

        clippy =
          pkgs.runCommand "clippy-check"
          {
            buildInputs = [rustToolchain];
          }
          ''
            cd ${./.}
            cargo clippy -- -D warnings
            touch $out
          '';
      };
    });
}
