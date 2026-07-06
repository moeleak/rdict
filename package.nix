{
  lib,
  stdenv,
  buildPackages,
  rustPlatform,
  installShellFiles,
  libiconv,
  writableTmpDirAsHomeHook,
}:
let
  version = "0.3.0";
  src = lib.cleanSource ./.;
  cargoHash = "sha256-HT1itjT7KQRmjPyhXmZjso5Le5P9/yXJYp26fPkykNE=";
  darwinHostLinkAttrs = lib.optionalAttrs stdenv.buildPlatform.isDarwin {
    RUSTFLAGS = "-L native=${buildPackages.libiconv}/lib";
    env.LIBRARY_PATH = "${buildPackages.libiconv}/lib";
  };
in
rec {
  default = rdict;

  rdict = rustPlatform.buildRustPackage ({
    inherit version src cargoHash;

    pname = "rdict";

    buildAndTestSubdir = "./rdict-cli";

    nativeBuildInputs =
      lib.optionals stdenv.buildPlatform.isDarwin [
        libiconv
      ]
      ++ lib.optionals (stdenv.buildPlatform.canExecute stdenv.hostPlatform) [
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
  } // darwinHostLinkAttrs);

  rdict-telegram = rustPlatform.buildRustPackage ({
    inherit version src cargoHash;

    pname = "rdict-telegram";

    nativeBuildInputs =
      lib.optionals stdenv.buildPlatform.isDarwin [
        libiconv
      ]
      ++ lib.optionals (stdenv.buildPlatform.canExecute stdenv.hostPlatform) [
        writableTmpDirAsHomeHook
      ];

    buildAndTestSubdir = "./rdict-telegram";

    meta = {
      license = lib.licenses.mit;
      mainProgram = "rdict-telegram";
    };
  } // darwinHostLinkAttrs);

  rdict-iced = rustPlatform.buildRustPackage ({
    inherit version src cargoHash;

    pname = "rdict-iced";

    buildAndTestSubdir = "./rdict-iced";
    doCheck = stdenv.buildPlatform.canExecute stdenv.hostPlatform;

    nativeBuildInputs = lib.optionals stdenv.buildPlatform.isDarwin [
      libiconv
    ];

    meta = {
      license = lib.licenses.mit;
      mainProgram = "rdict-iced";
    };
  } // darwinHostLinkAttrs);
}
