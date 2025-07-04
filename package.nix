{ lib, rustPlatform }:
let
  cargoHash = "sha256-kTmN33oFodggtIwZo1Ex4slRSfYoYVOl7JVWxouVNOI=";
in
rec {
  default = rdict;
  rdict = rustPlatform.buildRustPackage {
    inherit cargoHash;

    pname = "rdict";
    version = "0.1.0";

    src = lib.cleanSource ./.;

    buildAndTestSubdir = "./rdict-cli";

    preCheck = ''
      export HOME="$(mktemp -d)"
    '';

    meta = {
      license = lib.licenses.mit;
      mainProgram = "rdict";
    };
  };

  rdict-telegram = rustPlatform.buildRustPackage {
    inherit cargoHash;

    pname = "rdict-telegram";
    version = "0.1.0";

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
