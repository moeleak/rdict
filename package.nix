{
  lib,
  stdenv,
  rustPlatform,
  installShellFiles,
  writableTmpDirAsHomeHook,
}:
let
  version = "0.3.0";
  src = lib.cleanSource ./.;
  cargoHash = "sha256-DF3LxDMXkYZoZHIbt/cV98HpxKcnlS1OnxALzoiWhJA=";
in
rec {
  default = rdict;

  rdict = rustPlatform.buildRustPackage {
    inherit version src cargoHash;

    pname = "rdict";

    buildAndTestSubdir = "./rdict-cli";

    nativeBuildInputs = lib.optionals (stdenv.buildPlatform.canExecute stdenv.hostPlatform) [
      installShellFiles
      writableTmpDirAsHomeHook
    ];

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
    inherit version src cargoHash;

    pname = "rdict-telegram";

    nativeBuildInputs = lib.optionals (stdenv.buildPlatform.canExecute stdenv.hostPlatform) [
      writableTmpDirAsHomeHook
    ];

    buildAndTestSubdir = "./rdict-telegram";

    meta = {
      license = lib.licenses.mit;
      mainProgram = "rdict-telegram";
    };
  };
}
