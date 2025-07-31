{
  lib,
  stdenv,
  rustPlatform,
  installShellFiles,
}:
let
  cargoHash = "sha256-Bzy77XM3DxNWjmD/3jIDfpwPPhqucj0CHj2266tXoTE=";
in
rec {
  default = rdict;
  rdict = rustPlatform.buildRustPackage {
    inherit cargoHash;

    pname = "rdict";
    version = "0.2.0";

    src = lib.cleanSource ./.;

    buildAndTestSubdir = "./rdict-cli";

    nativeBuildInputs = lib.optionals (stdenv.buildPlatform.canExecute stdenv.hostPlatform) [
      installShellFiles
    ];

    preCheck = ''
      export HOME="$(mktemp -d)"
    '';

    postInstall = lib.optionalString (stdenv.buildPlatform.canExecute stdenv.hostPlatform) ''
      installShellCompletion --cmd rdict \
        --bash <("$out/bin/rdict" --completion bash) \
        --zsh <("$out/bin/rdict" --completion zsh) \
        --fish <("$out/bin/rdict" --completion fish)
    '';

    meta = {
      license = lib.licenses.mit;
      mainProgram = "rdict";
    };
  };

  rdict-telegram = rustPlatform.buildRustPackage {
    inherit cargoHash;

    pname = "rdict-telegram";
    version = "0.2.0";

    src = lib.cleanSource ./.;

    buildAndTestSubdir = "./rdict-telegram";

    preCheck = ''
      export HOME="$(mktemp -d)"
    '';

    meta = {
      license = lib.licenses.mit;
      mainProgram = "rdict-telegram";
    };
  };
}
