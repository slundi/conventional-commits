{
  lib,
  rustPlatform,
  fetchFromGitHub,
}:
rustPlatform.buildRustPackage rec {
  pname = "conventional-commits";
  version = "1.0.2";

  src = fetchFromGitHub {
    owner = "slundi";
    repo = "conventional-commits";
    rev = "v${version}";
    hash = "sha256-vnm/brSS875gj7pvVTDYM+6fotXVqJbB1tgYXeNHQV4=";
  };

  # v1.0.2 was tagged without Cargo.lock, so it cannot be vendored.
  # Bump `version`/`hash` to the first tag that ships Cargo.lock, build once,
  # and paste the hash Nix reports here.
  cargoHash = lib.fakeHash;

  postInstall = ''
    install -Dm644 commit-check.example.toml \
      $out/share/doc/commit-check/commit-check.example.toml
  '';

  meta = {
    description = "A lightweight library and CLI tool for validating Conventional Commits";
    homepage = "https://codeberg.org/slundi/conventional-commits";
    changelog = "https://codeberg.org/slundi/conventional-commits/src/tag/v${version}/Changelog.md";
    license = lib.licenses.mit;
    maintainers = [];
    mainProgram = "commit-check";
    platforms = lib.platforms.all;
  };
}
