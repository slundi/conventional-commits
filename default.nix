{
  lib,
  rustPlatform,
  fetchFromGitHub,
  installShellFiles,
  stdenv,
}:
rustPlatform.buildRustPackage rec {
  pname = "conventional-commits";
  version = "0.1.0";

  src = fetchFromGitHub {
    owner = "yourusername"; # Remplacez par votre nom d'utilisateur GitHub
    repo = "conventional-commits";
    rev = "v${version}";
    hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="; # À remplacer par le vrai hash
  };

  cargoHash = "sha256-BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB="; # À remplacer par le vrai hash

  nativeBuildInputs = [
    installShellFiles
  ];

  postInstall = ''
    # Install shell completions
    installShellCompletion --cmd commit-check \
      --bash <($out/bin/commit-check --help | grep -A 50 "SUBCOMMANDS" || true) \
      --zsh <($out/bin/commit-check --help | grep -A 50 "SUBCOMMANDS" || true) \
      --fish <($out/bin/commit-check --help | grep -A 50 "SUBCOMMANDS" || true)

    # Install man page if it exists
    if [ -f "man/commit-check.1" ]; then
      installManPage man/commit-check.1
    fi
  '';

  meta = with lib; {
    description = "A lightweight library and CLI tool for validating Conventional Commits";
    homepage = "https://github.com/yourusername/conventional-commits";
    license = licenses.mit;
    maintainers = with maintainers; []; # Ajoutez votre nom de maintainer nixpkgs ici
    mainProgram = "commit-check";
    platforms = platforms.all;
    changelog = "https://github.com/yourusername/conventional-commits/blob/v${version}/CHANGELOG.md";
  };
}
