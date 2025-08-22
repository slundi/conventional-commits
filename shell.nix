{pkgs ? import <nixpkgs> {}}:
pkgs.mkShell {
  inputsFrom = with pkgs; [
    llvmPackages.bintools
    rustc
  ];

  buildInputs = with pkgs; [
    llvmPackages.bintools
    # gcc

    # Rust toolchain
    rustc
    cargo
    rustfmt
    clippy

    # Development tools
    cargo-audit
    cargo-tarpaulin
    rust-analyzer
    clippy
    cargo-crev
    # cargo-deb # build deb
    # cargo-deps # dependency graph

    # Shell completions
    installShellFiles

    # Nix tools
    nixpkgs-fmt
    nix-prefetch-github
  ];

  packages = with pkgs; [
    pkg-config
    llvmPackages.bintools
    rustc
  ];

  shellHook = ''
    echo "🔨 Nix development shell for conventional-commits"
    echo ""
    echo "Available commands:"
    echo "  cargo build                 - Build the project"
    echo "  cargo test                  - Run tests"
    echo "  cargo clippy                - Run linter"
    echo "  cargo fmt                   - Format code"
    echo "  make test                   - Run comprehensive tests"
    echo "  nix-build                   - Build with Nix"
    echo "  nix-prefetch-github <user> <repo>  - Get GitHub hash"
    echo ""
    echo "To build the Nix package:"
    echo "  nix-build default.nix"
    echo ""
    echo "To test installation:"
    echo "  nix-env -f default.nix -i"
    echo ""
  '';

  # Certain Rust tools won't work without this
  # This can also be fixed by using oxalica/rust-overlay and specifying the rust-src extension
  # See https://discourse.nixos.org/t/rust-src-not-found-and-other-misadventures-of-developing-rust-on-nixos/11570/3?u=samuela. for more details.
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
