# When adding new tools to this template, always check the official registry to
# find the correct attribute name:
# * Search Tool: NixOS Package Search
# * Usage: If you find ripgrep, simply add pkgs.ripgrep to the buildInputs in your flake.nix.
{
  description = "Rust Template with automatic pre-commit hooks";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        extensions = with pkgs.vscode-extensions; [
          rust-lang.rust-analyzer
          tamasfe.even-better-toml
          jnoortheen.nix-ide
          mkhl.direnv
          vadimcn.vscode-lldb
          redhat.vscode-yaml
          # Note: If swellaby is not in your nixpkgs channel,
          # you may need to use a community overlay or skip it here.
          # swellaby.vscode-rust-test-adapter
        ];
        # Create a custom VSCodium with these extensions
        custom-codium = pkgs.vscode-with-extensions.override {
          vscode = pkgs.vscodium;
          vscodeExtensions = extensions;
        };

        # Define the rust toolchain from your toml
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
      in
      {
        # The Package definition
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = manifest.name;
          version = manifest.version;
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
          ];

          postInstall = ''
            install -Dm644 commit-check.example.toml \
              $out/share/doc/commit-check/commit-check.example.toml
          '';
        };

        # Development Shell
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            cargo-dist
            cargo-nextest
            cargo-cross
            cargo-machete
            cargo-audit
            cargo-deny
            cargo-outdated # Reports dependencies with newer releases available
            cargo-llvm-cov # Coverage instrumentation via LLVM
            prek # pre-commit
            gitleaks # The compiled secret scanner or trufflehog or ripgrep
            just # Command runner
            bacon # Background checker (cargo check/clippy/test on save)
            git-cliff # Changelog generator from conventional commits
            mold # Fast linker (referenced in .cargo/config.toml)
            dprint # Formatter for JSON, TOML, Markdown
            nixfmt-rfc-style # Nix formatter (RFC 166)
            nil # Nix language server (referenced in .vscode/settings.json)
            custom-codium
          ];

          # This runs when the shell starts
          shellHook = ''
            echo "🦀 Rust Dev Shell Loaded"
            echo "💡 Tip: Run 'codium .' to start coding with all extensions pre-installed."
            # Automatically install hooks if .git exists
            if [ -d .git ] && command -v prek >/dev/null; then
              prek install
            fi
          '';
        };

        # App definition (allows 'nix run')
        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
        };
      }
    );
}
