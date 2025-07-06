{ lib, rustPlatform }:
rec {
  default = rdict;
  rdict = rustPlatform.buildRustPackage {
    pname = "rdict";
    version = "0.1.0";

    src = lib.cleanSource ./.;

    useFetchCargoVendor = true;
    cargoHash = "sha256-Oi1N29W7PzO4qucJ9ggTH/tLT6Tvd5Yx5+P1UwaIN4w=";

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
    pname = "rdict-telegram";
    version = "0.1.0";

    src = lib.cleanSource ./.;

    useFetchCargoVendor = true;
    cargoHash = "sha256-Oi1N29W7PzO4qucJ9ggTH/tLT6Tvd5Yx5+P1UwaIN4w=";

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
